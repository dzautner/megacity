//! SVC-015: Parks Multi-Tier System
//!
//! Differentiates park tiers functionally:
//! - **SmallPark**: +5 happiness, +3 land value bonus
//! - **Playground**: +5 happiness for families (citizens with children)
//! - **LargePark**: +10 happiness, +8 land value, pollution reduction
//! - **SportsField**: +5 happiness, exercise/health bonus
//! - **Plaza**: +3 happiness, commercial boost (land value for commercial zones)
//!
//! Also tracks city-wide park acreage vs the NRPA standard of 10 acres
//! per 1,000 population. A park deficit applies a happiness penalty.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// NRPA standard: 10 acres of parkland per 1,000 population.
const NRPA_ACRES_PER_1000_POP: f32 = 10.0;

/// Each park cell is CELL_SIZE x CELL_SIZE feet; convert to acres.
/// 1 acre = 43,560 sqft. Each cell = 16*16 = 256 sqft in game units.
/// We use a simplified game-scale conversion: 1 park cell ≈ 0.1 acres.
const ACRES_PER_PARK_CELL: f32 = 0.1;

/// Maximum happiness penalty from park deficit (city-wide).
const MAX_DEFICIT_PENALTY: f32 = 8.0;

/// Per-tier happiness bonuses applied to cells within range.
const SMALL_PARK_HAPPINESS: f32 = 5.0;
const PLAYGROUND_HAPPINESS: f32 = 5.0;
const LARGE_PARK_HAPPINESS: f32 = 10.0;
const SPORTS_FIELD_HAPPINESS: f32 = 5.0;
const PLAZA_HAPPINESS: f32 = 3.0;

/// Per-tier land value bonuses.
const SMALL_PARK_LAND_VALUE: f32 = 3.0;
const LARGE_PARK_LAND_VALUE: f32 = 8.0;
const PLAZA_COMMERCIAL_BOOST: f32 = 5.0;

/// Health bonus from SportsField (added to nearby citizens' health factor).
const SPORTS_FIELD_HEALTH_BONUS: f32 = 3.0;

/// Pollution reduction radius and intensity for LargePark.
const LARGE_PARK_POLLUTION_REDUCTION: u8 = 8;

// ---------------------------------------------------------------------------
// Per-cell park effects grid
// ---------------------------------------------------------------------------

/// Precomputed per-cell park effects, updated each slow tick.
#[derive(Resource, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct ParkEffectsGrid {
    /// Happiness bonus per cell from differentiated park tiers.
    pub happiness_bonus: Vec<f32>,
    /// Land value bonus per cell from nearby parks.
    pub land_value_bonus: Vec<f32>,
    /// Health bonus per cell from SportsField proximity.
    pub health_bonus: Vec<f32>,
    /// Pollution reduction per cell from LargePark proximity.
    pub pollution_reduction: Vec<u8>,
    /// Whether the cell has playground coverage (family happiness).
    pub has_playground: Vec<bool>,
    /// Whether the cell has plaza commercial boost.
    pub has_plaza_boost: Vec<bool>,
}

impl Default for ParkEffectsGrid {
    fn default() -> Self {
        let n = GRID_WIDTH * GRID_HEIGHT;
        Self {
            happiness_bonus: vec![0.0; n],
            land_value_bonus: vec![0.0; n],
            health_bonus: vec![0.0; n],
            pollution_reduction: vec![0; n],
            has_playground: vec![false; n],
            has_plaza_boost: vec![false; n],
        }
    }
}

impl ParkEffectsGrid {
    #[inline]
    pub fn idx(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    pub fn happiness_at(&self, x: usize, y: usize) -> f32 {
        self.happiness_bonus[Self::idx(x, y)]
    }

    pub fn land_value_at(&self, x: usize, y: usize) -> f32 {
        self.land_value_bonus[Self::idx(x, y)]
    }

    pub fn health_at(&self, x: usize, y: usize) -> f32 {
        self.health_bonus[Self::idx(x, y)]
    }

    pub fn pollution_reduction_at(&self, x: usize, y: usize) -> u8 {
        self.pollution_reduction[Self::idx(x, y)]
    }

    fn clear(&mut self) {
        self.happiness_bonus.fill(0.0);
        self.land_value_bonus.fill(0.0);
        self.health_bonus.fill(0.0);
        self.pollution_reduction.fill(0);
        self.has_playground.fill(false);
        self.has_plaza_boost.fill(false);
    }
}

// ---------------------------------------------------------------------------
// City-wide park statistics
// ---------------------------------------------------------------------------

/// City-wide park supply/demand tracking for NRPA standard compliance.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct ParksState {
    /// Total park acreage in the city.
    pub total_park_acres: f32,
    /// Target park acreage based on population.
    pub target_park_acres: f32,
    /// Ratio of actual to target (1.0 = meeting NRPA standard).
    pub coverage_ratio: f32,
    /// City-wide happiness penalty from park deficit (0.0 if meeting standard).
    pub deficit_penalty: f32,
    /// Count of each park type.
    pub small_park_count: u32,
    pub large_park_count: u32,
    pub playground_count: u32,
    pub sports_field_count: u32,
    pub plaza_count: u32,
}

impl crate::Saveable for ParksState {
    const SAVE_KEY: &'static str = "parks_system";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Park tier metadata
// ---------------------------------------------------------------------------

/// Return the number of park cells (footprint area) for a park service type.
fn park_cell_count(service_type: ServiceType) -> u32 {
    let (w, h) = ServiceBuilding::footprint(service_type);
    (w * h) as u32
}

/// Effect radius in cells for each park tier.
fn park_effect_radius(service_type: ServiceType) -> i32 {
    let radius_world = ServiceBuilding::coverage_radius(service_type);
    (radius_world / CELL_SIZE).ceil() as i32
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Recompute per-cell park effects and city-wide park statistics.
#[allow(clippy::too_many_arguments)]
pub fn update_park_effects(
    slow_timer: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    mut effects: ResMut<ParkEffectsGrid>,
    mut state: ResMut<ParksState>,
    stats: Res<crate::stats::CityStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    effects.clear();

    // Reset counts
    state.small_park_count = 0;
    state.large_park_count = 0;
    state.playground_count = 0;
    state.sports_field_count = 0;
    state.plaza_count = 0;

    let mut total_park_cells: u32 = 0;

    for service in &services {
        let st = service.service_type;
        if !ServiceBuilding::is_park(st) && st != ServiceType::Stadium {
            continue;
        }

        // Count park cells for acreage
        let cells = park_cell_count(st);
        total_park_cells += cells;

        // Track counts
        match st {
            ServiceType::SmallPark => state.small_park_count += 1,
            ServiceType::LargePark => state.large_park_count += 1,
            ServiceType::Playground => state.playground_count += 1,
            ServiceType::SportsField => state.sports_field_count += 1,
            ServiceType::Plaza => state.plaza_count += 1,
            _ => {}
        }

        let radius = park_effect_radius(st);
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = (radius as f32 * CELL_SIZE) * (radius as f32 * CELL_SIZE);

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let cx = sx + dx;
                let cy = sy + dy;
                if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                    continue;
                }
                let wx_diff = dx as f32 * CELL_SIZE;
                let wy_diff = dy as f32 * CELL_SIZE;
                if wx_diff * wx_diff + wy_diff * wy_diff > r2 {
                    continue;
                }
                let idx = ParkEffectsGrid::idx(cx as usize, cy as usize);
                apply_tier_effects(st, idx, &mut effects);
            }
        }
    }

    // Compute city-wide acreage and deficit
    state.total_park_acres = total_park_cells as f32 * ACRES_PER_PARK_CELL;
    let population = stats.population.max(1) as f32;
    state.target_park_acres = (population / 1000.0) * NRPA_ACRES_PER_1000_POP;

    state.coverage_ratio = if state.target_park_acres > 0.0 {
        (state.total_park_acres / state.target_park_acres).min(2.0)
    } else {
        1.0
    };

    // Park deficit penalty: linearly scales from 0 (at 100% coverage) to MAX_DEFICIT_PENALTY (at 0%)
    state.deficit_penalty = if state.coverage_ratio < 1.0 {
        (1.0 - state.coverage_ratio) * MAX_DEFICIT_PENALTY
    } else {
        0.0
    };
}

/// Apply tier-specific effects to a single cell index.
fn apply_tier_effects(service_type: ServiceType, idx: usize, effects: &mut ParkEffectsGrid) {
    match service_type {
        ServiceType::SmallPark => {
            effects.happiness_bonus[idx] = effects.happiness_bonus[idx].max(SMALL_PARK_HAPPINESS);
            effects.land_value_bonus[idx] =
                effects.land_value_bonus[idx].max(SMALL_PARK_LAND_VALUE);
        }
        ServiceType::Playground => {
            // Playground gives happiness to all, but the family bonus is tracked separately
            effects.happiness_bonus[idx] =
                effects.happiness_bonus[idx].max(PLAYGROUND_HAPPINESS);
            effects.has_playground[idx] = true;
        }
        ServiceType::LargePark => {
            effects.happiness_bonus[idx] = effects.happiness_bonus[idx].max(LARGE_PARK_HAPPINESS);
            effects.land_value_bonus[idx] =
                effects.land_value_bonus[idx].max(LARGE_PARK_LAND_VALUE);
            effects.pollution_reduction[idx] =
                effects.pollution_reduction[idx].max(LARGE_PARK_POLLUTION_REDUCTION);
        }
        ServiceType::SportsField => {
            effects.happiness_bonus[idx] =
                effects.happiness_bonus[idx].max(SPORTS_FIELD_HAPPINESS);
            effects.health_bonus[idx] =
                effects.health_bonus[idx].max(SPORTS_FIELD_HEALTH_BONUS);
        }
        ServiceType::Plaza => {
            effects.happiness_bonus[idx] = effects.happiness_bonus[idx].max(PLAZA_HAPPINESS);
            effects.has_plaza_boost[idx] = true;
            effects.land_value_bonus[idx] =
                effects.land_value_bonus[idx].max(PLAZA_COMMERCIAL_BOOST);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ParksSystemPlugin;

impl Plugin for ParksSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParkEffectsGrid>();
        app.init_resource::<ParksState>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ParksState>();

        app.add_systems(
            FixedUpdate,
            update_park_effects
                .after(crate::stats::update_stats)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_park_effect_defaults() {
        let grid = ParkEffectsGrid::default();
        assert_eq!(grid.happiness_at(10, 10), 0.0);
        assert_eq!(grid.land_value_at(10, 10), 0.0);
        assert_eq!(grid.health_at(10, 10), 0.0);
        assert_eq!(grid.pollution_reduction_at(10, 10), 0);
    }

    #[test]
    fn test_apply_small_park_effects() {
        let mut effects = ParkEffectsGrid::default();
        let idx = ParkEffectsGrid::idx(10, 10);
        apply_tier_effects(ServiceType::SmallPark, idx, &mut effects);
        assert!((effects.happiness_bonus[idx] - SMALL_PARK_HAPPINESS).abs() < f32::EPSILON);
        assert!((effects.land_value_bonus[idx] - SMALL_PARK_LAND_VALUE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_large_park_effects() {
        let mut effects = ParkEffectsGrid::default();
        let idx = ParkEffectsGrid::idx(10, 10);
        apply_tier_effects(ServiceType::LargePark, idx, &mut effects);
        assert!((effects.happiness_bonus[idx] - LARGE_PARK_HAPPINESS).abs() < f32::EPSILON);
        assert!((effects.land_value_bonus[idx] - LARGE_PARK_LAND_VALUE).abs() < f32::EPSILON);
        assert_eq!(effects.pollution_reduction[idx], LARGE_PARK_POLLUTION_REDUCTION);
    }

    #[test]
    fn test_apply_playground_effects() {
        let mut effects = ParkEffectsGrid::default();
        let idx = ParkEffectsGrid::idx(5, 5);
        apply_tier_effects(ServiceType::Playground, idx, &mut effects);
        assert!((effects.happiness_bonus[idx] - PLAYGROUND_HAPPINESS).abs() < f32::EPSILON);
        assert!(effects.has_playground[idx]);
    }

    #[test]
    fn test_apply_sports_field_effects() {
        let mut effects = ParkEffectsGrid::default();
        let idx = ParkEffectsGrid::idx(5, 5);
        apply_tier_effects(ServiceType::SportsField, idx, &mut effects);
        assert!((effects.happiness_bonus[idx] - SPORTS_FIELD_HAPPINESS).abs() < f32::EPSILON);
        assert!((effects.health_bonus[idx] - SPORTS_FIELD_HEALTH_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_plaza_effects() {
        let mut effects = ParkEffectsGrid::default();
        let idx = ParkEffectsGrid::idx(5, 5);
        apply_tier_effects(ServiceType::Plaza, idx, &mut effects);
        assert!((effects.happiness_bonus[idx] - PLAZA_HAPPINESS).abs() < f32::EPSILON);
        assert!(effects.has_plaza_boost[idx]);
    }

    #[test]
    fn test_higher_tier_wins() {
        let mut effects = ParkEffectsGrid::default();
        let idx = ParkEffectsGrid::idx(10, 10);
        // Apply SmallPark first, then LargePark — LargePark values should win
        apply_tier_effects(ServiceType::SmallPark, idx, &mut effects);
        apply_tier_effects(ServiceType::LargePark, idx, &mut effects);
        assert!((effects.happiness_bonus[idx] - LARGE_PARK_HAPPINESS).abs() < f32::EPSILON);
        assert!((effects.land_value_bonus[idx] - LARGE_PARK_LAND_VALUE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_deficit_penalty_computation() {
        // 500 pop, 0 parks → deficit
        let pop = 500.0_f32;
        let target = (pop / 1000.0) * NRPA_ACRES_PER_1000_POP; // 5 acres
        let actual = 0.0_f32;
        let ratio = actual / target;
        let penalty = (1.0 - ratio) * MAX_DEFICIT_PENALTY;
        assert!((penalty - MAX_DEFICIT_PENALTY).abs() < f32::EPSILON);
    }

    #[test]
    fn test_no_deficit_when_standard_met() {
        let pop = 1000.0_f32;
        let target = (pop / 1000.0) * NRPA_ACRES_PER_1000_POP; // 10 acres
        let actual = 10.0_f32;
        let ratio = (actual / target).min(2.0);
        let penalty = if ratio < 1.0 {
            (1.0 - ratio) * MAX_DEFICIT_PENALTY
        } else {
            0.0
        };
        assert!(penalty.abs() < f32::EPSILON);
    }

    #[test]
    fn test_clear_resets_effects() {
        let mut effects = ParkEffectsGrid::default();
        let idx = ParkEffectsGrid::idx(10, 10);
        apply_tier_effects(ServiceType::LargePark, idx, &mut effects);
        assert!(effects.happiness_bonus[idx] > 0.0);
        effects.clear();
        assert!(effects.happiness_bonus[idx].abs() < f32::EPSILON);
        assert_eq!(effects.pollution_reduction[idx], 0);
    }
}

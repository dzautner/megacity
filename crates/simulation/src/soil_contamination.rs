//! POLL-013: Soil Contamination Grid and Persistence Model
//!
//! Unlike air/water pollution which decays rapidly, soil contamination persists
//! for decades. Industrial buildings, landfills, and gas stations contribute
//! contamination that remains even after buildings are demolished.
//!
//! Key properties:
//! - f32 per cell, 0.0-500.0 range
//! - Very slow natural decay (0.9999 per update)
//! - Lateral spread when concentration > 50
//! - Updates every 30 ticks
//! - Seeps into groundwater quality

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::ZoneType;
use crate::groundwater::WaterQualityGrid;
use crate::landfill::LandfillState;
use crate::services::{ServiceBuilding, ServiceType};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Decay factor applied each update cycle. Near 1.0 = virtually permanent.
const SOIL_NATURAL_DECAY: f32 = 0.9999;

/// Maximum contamination level per cell.
const MAX_CONTAMINATION: f32 = 500.0;

/// Contamination threshold for lateral spread to neighbors.
const SPREAD_THRESHOLD: f32 = 50.0;

/// Rate of lateral spread to each cardinal neighbor.
const SPREAD_RATE: f32 = 0.01;

/// Base contamination rate per industrial building level per update.
const INDUSTRIAL_BASE_RATE: f32 = 3.0;

/// Contamination rate for lined landfills per update.
const LANDFILL_LINED_RATE: f32 = 1.0;

/// Contamination rate for unlined landfills per update.
const LANDFILL_UNLINED_RATE: f32 = 5.0;

/// Contamination rate for landfills with gas collection (lined).
const LANDFILL_COLLECTION_RATE: f32 = 0.5;

/// Update frequency in ticks.
pub const UPDATE_INTERVAL: u32 = 30;

/// Rate at which soil contamination seeps into groundwater quality per update.
const GROUNDWATER_SEEP_FACTOR: f32 = 0.02;

// ---------------------------------------------------------------------------
// SoilContaminationGrid resource
// ---------------------------------------------------------------------------

/// Grid tracking soil contamination level (0.0-500.0) per cell.
///
/// Soil contamination is nearly permanent — it persists even after the source
/// building is demolished. This models real-world brownfield site behavior.
#[derive(Resource, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct SoilContaminationGrid {
    pub levels: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for SoilContaminationGrid {
    fn default() -> Self {
        Self {
            levels: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl SoilContaminationGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.levels[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        self.levels[y * self.width + x] = val.clamp(0.0, MAX_CONTAMINATION);
    }

    #[inline]
    fn add(&mut self, x: usize, y: usize, amount: f32) {
        let idx = y * self.width + x;
        self.levels[idx] = (self.levels[idx] + amount).min(MAX_CONTAMINATION);
    }
}

// ---------------------------------------------------------------------------
// Tick counter for update frequency
// ---------------------------------------------------------------------------

/// Internal tick counter for soil contamination updates.
#[derive(Resource, Default)]
pub struct SoilContaminationTimer {
    counter: u32,
}

impl SoilContaminationTimer {
    fn tick(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }

    pub fn should_run(&self) -> bool {
        self.counter > 0 && self.counter.is_multiple_of(UPDATE_INTERVAL)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Advance the soil contamination tick counter each FixedUpdate.
fn tick_soil_timer(mut timer: ResMut<SoilContaminationTimer>) {
    timer.tick();
}

/// Main soil contamination update system.
///
/// Phases:
/// 1. Natural decay (multiply all cells by SOIL_NATURAL_DECAY)
/// 2. Industrial building emissions (3.0 per building level)
/// 3. Landfill emissions (based on liner type)
/// 4. Lateral spread (cells > 50 spread 0.01 to cardinal neighbors)
/// 5. Groundwater seepage (soil contamination reduces water quality)
#[allow(clippy::too_many_arguments)]
fn update_soil_contamination(
    timer: Res<SoilContaminationTimer>,
    mut grid: ResMut<SoilContaminationGrid>,
    mut water_quality: ResMut<WaterQualityGrid>,
    buildings: Query<&Building>,
    landfill_state: Res<LandfillState>,
    services: Query<&ServiceBuilding>,
) {
    if !timer.should_run() {
        return;
    }

    // --- Phase 1: Natural decay ---
    for val in grid.levels.iter_mut() {
        *val *= SOIL_NATURAL_DECAY;
        if *val < 0.001 {
            *val = 0.0;
        }
    }

    // --- Phase 2: Industrial building emissions ---
    for building in &buildings {
        if building.zone_type != ZoneType::Industrial {
            continue;
        }
        let amount = INDUSTRIAL_BASE_RATE * building.level as f32;
        let x = building.grid_x;
        let y = building.grid_y;
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            grid.add(x, y, amount);
        }
    }

    // --- Phase 3: Landfill emissions ---
    for site in &landfill_state.sites {
        if !site.status.is_active() {
            continue;
        }
        let amount = match site.liner_type {
            crate::landfill::LandfillLinerType::Unlined => LANDFILL_UNLINED_RATE,
            crate::landfill::LandfillLinerType::Lined => LANDFILL_LINED_RATE,
            crate::landfill::LandfillLinerType::LinedWithCollection => LANDFILL_COLLECTION_RATE,
        };
        let x = site.grid_x;
        let y = site.grid_y;
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            grid.add(x, y, amount);
        }
    }

    // Fallback: landfill service buildings not tracked in LandfillState
    for service in &services {
        if service.service_type != ServiceType::Landfill {
            continue;
        }
        let already_tracked = landfill_state
            .sites
            .iter()
            .any(|s| s.grid_x == service.grid_x && s.grid_y == service.grid_y);
        if already_tracked {
            continue;
        }
        let x = service.grid_x;
        let y = service.grid_y;
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            grid.add(x, y, LANDFILL_UNLINED_RATE);
        }
    }

    // --- Phase 4: Lateral spread ---
    let snapshot: Vec<f32> = grid.levels.clone();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let current = snapshot[y * GRID_WIDTH + x];
            if current <= SPREAD_THRESHOLD {
                continue;
            }
            let spread_amount = current * SPREAD_RATE;
            let neighbors: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
            for (dx, dy) in neighbors {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0
                    || ny < 0
                    || (nx as usize) >= GRID_WIDTH
                    || (ny as usize) >= GRID_HEIGHT
                {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;
                if current > snapshot[uy * GRID_WIDTH + ux] {
                    grid.add(ux, uy, spread_amount);
                }
            }
        }
    }

    // --- Phase 5: Groundwater seepage ---
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let contamination = grid.get(x, y);
            if contamination > 1.0 {
                let reduction = (contamination * GROUNDWATER_SEEP_FACTOR) as u8;
                if reduction > 0 {
                    let current_quality = water_quality.get(x, y);
                    water_quality.set(x, y, current_quality.saturating_sub(reduction));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl crate::Saveable for SoilContaminationGrid {
    const SAVE_KEY: &'static str = "soil_contamination";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        let has_contamination = self.levels.iter().any(|&v| v > 0.0);
        if !has_contamination {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SoilContaminationPlugin;

impl Plugin for SoilContaminationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoilContaminationGrid>()
            .init_resource::<SoilContaminationTimer>()
            .add_systems(
                FixedUpdate,
                (
                    tick_soil_timer,
                    update_soil_contamination.after(tick_soil_timer),
                )
                    .in_set(crate::SimulationSet::Simulation),
            );

        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<SoilContaminationGrid>();
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soil_contamination_grid_default() {
        let grid = SoilContaminationGrid::default();
        assert_eq!(grid.levels.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(grid.get(0, 0), 0.0);
        assert_eq!(grid.get(128, 128), 0.0);
    }

    #[test]
    fn test_soil_contamination_grid_set_get() {
        let mut grid = SoilContaminationGrid::default();
        grid.set(10, 20, 100.0);
        assert_eq!(grid.get(10, 20), 100.0);
    }

    #[test]
    fn test_soil_contamination_grid_clamp_max() {
        let mut grid = SoilContaminationGrid::default();
        grid.set(5, 5, 600.0);
        assert_eq!(grid.get(5, 5), MAX_CONTAMINATION);
    }

    #[test]
    fn test_soil_contamination_grid_add() {
        let mut grid = SoilContaminationGrid::default();
        grid.add(3, 3, 50.0);
        assert_eq!(grid.get(3, 3), 50.0);
        grid.add(3, 3, 30.0);
        assert_eq!(grid.get(3, 3), 80.0);
    }

    #[test]
    fn test_soil_contamination_grid_add_clamp() {
        let mut grid = SoilContaminationGrid::default();
        grid.set(1, 1, 490.0);
        grid.add(1, 1, 20.0);
        assert_eq!(grid.get(1, 1), MAX_CONTAMINATION);
    }

    #[test]
    fn test_decay_preserves_most_contamination() {
        let initial = 200.0_f32;
        let after_decay = initial * SOIL_NATURAL_DECAY;
        let loss_pct = (initial - after_decay) / initial * 100.0;
        assert!(
            loss_pct < 0.02,
            "Decay should be less than 0.02% per cycle, got {:.4}%",
            loss_pct
        );
    }

    #[test]
    fn test_spread_threshold() {
        assert!(49.0 <= SPREAD_THRESHOLD);
        assert!(51.0 > SPREAD_THRESHOLD);
    }

    #[test]
    fn test_timer_interval() {
        let mut timer = SoilContaminationTimer::default();
        assert!(!timer.should_run());
        for _ in 0..29 {
            timer.tick();
            assert!(!timer.should_run(), "Should not run before 30 ticks");
        }
        timer.tick(); // tick 30
        assert!(timer.should_run(), "Should run at tick 30");
    }

    #[test]
    fn test_saveable_key() {
        assert_eq!(SoilContaminationGrid::SAVE_KEY, "soil_contamination");
    }

    #[test]
    fn test_saveable_empty_grid_returns_none() {
        let grid = SoilContaminationGrid::default();
        assert!(grid.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut grid = SoilContaminationGrid::default();
        grid.set(50, 50, 123.456);
        grid.set(100, 200, 42.0);

        let bytes = grid.save_to_bytes().expect("Should save non-empty grid");
        let restored = SoilContaminationGrid::load_from_bytes(&bytes);

        assert!((restored.get(50, 50) - 123.456).abs() < 0.01);
        assert!((restored.get(100, 200) - 42.0).abs() < 0.01);
        assert_eq!(restored.get(0, 0), 0.0);
    }
}

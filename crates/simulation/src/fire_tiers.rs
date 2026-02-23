//! SVC-004: Fire Service Multi-Tier System
//!
//! Implements multi-tier fire service with different coverage radii and
//! suppression effectiveness:
//! - **Small Fire Station (FireHouse):** local coverage, basic equipment
//! - **Fire Station:** standard coverage, better equipment, faster response
//! - **Fire HQ:** city-wide coordination, specialized units (hazmat, ladder)
//!
//! Each tier has different fire suppression effectiveness and coverage radius.
//! The system tracks per-cell best tier and aggregates stats.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::fire::{FireGrid, OnFire};
use crate::buildings::Building;
use crate::budget::ExtendedBudget;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;
use crate::Saveable;

// =============================================================================
// Fire Tier Enum
// =============================================================================

/// Tier of fire service, ordered by capability.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
    Serialize, Deserialize, Encode, Decode,
)]
pub enum FireTier {
    /// Small Fire Station (FireHouse): local coverage, basic equipment.
    Small = 0,
    /// Fire Station: standard coverage, better equipment, faster response.
    Standard = 1,
    /// Fire HQ: city-wide coordination, specialized units (hazmat, ladder).
    Headquarters = 2,
}

impl FireTier {
    /// Fire suppression rate (intensity reduction per tick) for this tier.
    pub fn suppression_rate(self) -> f32 {
        match self {
            FireTier::Small => 1.0,
            FireTier::Standard => 2.0,
            FireTier::Headquarters => 4.0,
        }
    }

    /// Display name for the tier.
    pub fn name(self) -> &'static str {
        match self {
            FireTier::Small => "Small Fire Station",
            FireTier::Standard => "Fire Station",
            FireTier::Headquarters => "Fire HQ",
        }
    }

    /// Map from ServiceType to FireTier, if applicable.
    pub fn from_service_type(st: ServiceType) -> Option<FireTier> {
        match st {
            ServiceType::FireHouse => Some(FireTier::Small),
            ServiceType::FireStation => Some(FireTier::Standard),
            ServiceType::FireHQ => Some(FireTier::Headquarters),
            _ => None,
        }
    }
}

// =============================================================================
// Per-cell Tier Coverage Grid
// =============================================================================

/// Per-cell tracking of the best (highest) fire tier covering each cell.
/// `None` means no fire service coverage; `Some(tier)` means the best tier.
#[derive(Resource)]
pub struct FireTierCoverageGrid {
    /// One entry per grid cell. `0` = no coverage, `1..=3` = tier + 1.
    tiers: Vec<u8>,
    pub dirty: bool,
}

impl Default for FireTierCoverageGrid {
    fn default() -> Self {
        Self {
            tiers: vec![0; GRID_WIDTH * GRID_HEIGHT],
            dirty: true,
        }
    }
}

impl FireTierCoverageGrid {
    #[inline]
    fn idx(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    /// Get the best fire tier covering this cell, or `None`.
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<FireTier> {
        match self.tiers[Self::idx(x, y)] {
            1 => Some(FireTier::Small),
            2 => Some(FireTier::Standard),
            3 => Some(FireTier::Headquarters),
            _ => None,
        }
    }

    /// Set the tier for a cell, keeping the highest tier seen.
    #[inline]
    fn set_max(&mut self, x: usize, y: usize, tier: FireTier) {
        let idx = Self::idx(x, y);
        let encoded = tier as u8 + 1;
        if encoded > self.tiers[idx] {
            self.tiers[idx] = encoded;
        }
    }

    pub fn clear(&mut self) {
        self.tiers.fill(0);
    }
}

// =============================================================================
// Fire Tiers State (persisted stats)
// =============================================================================

/// Aggregated statistics for fire service tiers.
#[derive(Resource, Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct FireTiersState {
    /// Cumulative fires extinguished by each tier.
    pub extinguished_by_small: u32,
    pub extinguished_by_standard: u32,
    pub extinguished_by_hq: u32,
    /// Count of fire service buildings per tier.
    pub count_small: u32,
    pub count_standard: u32,
    pub count_hq: u32,
    /// Total fire intensity suppressed this cycle (resets each slow tick).
    pub suppression_this_cycle: f32,
}

impl Saveable for FireTiersState {
    const SAVE_KEY: &'static str = "fire_tiers";
    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }
    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Recompute the per-cell fire tier coverage grid whenever service buildings
/// are added or budget changes.
pub fn update_fire_tier_coverage(
    services: Query<&ServiceBuilding>,
    added: Query<Entity, Added<ServiceBuilding>>,
    ext_budget: Res<ExtendedBudget>,
    mut tier_grid: ResMut<FireTierCoverageGrid>,
    mut state: ResMut<FireTiersState>,
) {
    if !added.is_empty() {
        tier_grid.dirty = true;
    }
    if ext_budget.is_changed() {
        tier_grid.dirty = true;
    }
    if !tier_grid.dirty {
        return;
    }
    tier_grid.dirty = false;
    tier_grid.clear();

    // Reset building counts.
    state.count_small = 0;
    state.count_standard = 0;
    state.count_hq = 0;

    for service in &services {
        let tier = match FireTier::from_service_type(service.service_type) {
            Some(t) => t,
            None => continue,
        };

        match tier {
            FireTier::Small => state.count_small += 1,
            FireTier::Standard => state.count_standard += 1,
            FireTier::Headquarters => state.count_hq += 1,
        }

        let budget_level = ext_budget.service_budgets.for_service(service.service_type);
        let effective_radius = service.radius * budget_level;
        let radius_cells = (effective_radius / CELL_SIZE).ceil() as i32;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = effective_radius * effective_radius;

        for dy in -radius_cells..=radius_cells {
            for dx in -radius_cells..=radius_cells {
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
                tier_grid.set_max(cx as usize, cy as usize, tier);
            }
        }
    }
}

/// Enhanced fire suppression that uses tier-based rates instead of a flat
/// reduction. Higher-tier stations extinguish fires faster.
/// Runs every tick on buildings that are on fire.
pub fn tier_based_suppression(
    mut commands: Commands,
    mut fire_grid: ResMut<FireGrid>,
    tier_grid: Res<FireTierCoverageGrid>,
    mut state: ResMut<FireTiersState>,
    mut burning: Query<(Entity, &Building, &mut OnFire)>,
) {
    for (entity, building, mut on_fire) in &mut burning {
        let tier = match tier_grid.get(building.grid_x, building.grid_y) {
            Some(t) => t,
            None => continue, // No tier coverage — base extinguish_fires handles binary coverage
        };

        let rate = tier.suppression_rate();
        let old_intensity = on_fire.intensity;
        on_fire.intensity = (on_fire.intensity - rate).max(0.0);
        state.suppression_this_cycle += old_intensity - on_fire.intensity;

        if on_fire.intensity <= 0.0 {
            fire_grid.set(building.grid_x, building.grid_y, 0);
            commands.entity(entity).remove::<OnFire>();

            match tier {
                FireTier::Small => state.extinguished_by_small += 1,
                FireTier::Standard => state.extinguished_by_standard += 1,
                FireTier::Headquarters => state.extinguished_by_hq += 1,
            }
        } else {
            fire_grid.set(building.grid_x, building.grid_y, on_fire.intensity as u8);
        }
    }
}

/// Reset per-cycle stats on each slow tick.
pub fn reset_cycle_stats(slow_timer: Res<SlowTickTimer>, mut state: ResMut<FireTiersState>) {
    if !slow_timer.should_run() {
        return;
    }
    state.suppression_this_cycle = 0.0;
}

// =============================================================================
// Plugin
// =============================================================================

pub struct FireTiersPlugin;

impl Plugin for FireTiersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FireTierCoverageGrid>();
        app.init_resource::<FireTiersState>();

        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<FireTiersState>();

        app.add_systems(
            FixedUpdate,
            (
                update_fire_tier_coverage,
                tier_based_suppression,
                reset_cycle_stats,
            )
                .chain()
                .after(crate::happiness::update_service_coverage)
                .before(crate::fire::extinguish_fires)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fire_tier_ordering() {
        assert!(FireTier::Small < FireTier::Standard);
        assert!(FireTier::Standard < FireTier::Headquarters);
    }

    #[test]
    fn test_suppression_rates() {
        assert_eq!(FireTier::Small.suppression_rate(), 1.0);
        assert_eq!(FireTier::Standard.suppression_rate(), 2.0);
        assert_eq!(FireTier::Headquarters.suppression_rate(), 4.0);
    }

    #[test]
    fn test_from_service_type() {
        assert_eq!(
            FireTier::from_service_type(ServiceType::FireHouse),
            Some(FireTier::Small)
        );
        assert_eq!(
            FireTier::from_service_type(ServiceType::FireStation),
            Some(FireTier::Standard)
        );
        assert_eq!(
            FireTier::from_service_type(ServiceType::FireHQ),
            Some(FireTier::Headquarters)
        );
        assert_eq!(
            FireTier::from_service_type(ServiceType::PoliceStation),
            None
        );
    }

    #[test]
    fn test_tier_coverage_grid_default() {
        let grid = FireTierCoverageGrid::default();
        assert_eq!(grid.tiers.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(grid.tiers.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_tier_coverage_grid_set_get() {
        let mut grid = FireTierCoverageGrid::default();
        assert_eq!(grid.get(10, 10), None);
        grid.set_max(10, 10, FireTier::Small);
        assert_eq!(grid.get(10, 10), Some(FireTier::Small));
        // Higher tier should win.
        grid.set_max(10, 10, FireTier::Headquarters);
        assert_eq!(grid.get(10, 10), Some(FireTier::Headquarters));
        // Lower tier should NOT overwrite.
        grid.set_max(10, 10, FireTier::Standard);
        assert_eq!(grid.get(10, 10), Some(FireTier::Headquarters));
    }

    #[test]
    fn test_tier_coverage_grid_clear() {
        let mut grid = FireTierCoverageGrid::default();
        grid.set_max(5, 5, FireTier::Standard);
        assert_eq!(grid.get(5, 5), Some(FireTier::Standard));
        grid.clear();
        assert_eq!(grid.get(5, 5), None);
    }

    #[test]
    fn test_fire_tiers_state_default() {
        let state = FireTiersState::default();
        assert_eq!(state.extinguished_by_small, 0);
        assert_eq!(state.extinguished_by_standard, 0);
        assert_eq!(state.extinguished_by_hq, 0);
        assert_eq!(state.count_small, 0);
        assert_eq!(state.count_standard, 0);
        assert_eq!(state.count_hq, 0);
        assert_eq!(state.suppression_this_cycle, 0.0);
    }

    #[test]
    fn test_tier_names() {
        assert_eq!(FireTier::Small.name(), "Small Fire Station");
        assert_eq!(FireTier::Standard.name(), "Fire Station");
        assert_eq!(FireTier::Headquarters.name(), "Fire HQ");
    }
}

//! Urban Heat Island (UHI) mitigation measures (WEATHER-009).
//!
//! Provides several mitigation options that reduce `UhiGrid` values in affected
//! cells, lowering the urban heat island effect:
//!
//! - **Tree planting**: -1.5F UHI per tree cell (passive, from `TreeGrid`)
//! - **Green roofs**: -2.0F, building upgrade $15K/building
//! - **Cool (white) roofs**: -1.5F, building upgrade $3K/building
//! - **Cool pavement**: -1.0F, road upgrade $5K/cell
//! - **Parks**: -3.0F in radius 2, $10K/cell
//! - **Water features (fountains)**: -2.0F, placeable $8K each
//! - **Permeable surfaces**: -0.5F, $4K/cell
//! - **District cooling**: -1.0F in radius 3, large facility $50K each
//!
//! Each mitigation reduces `UhiGrid` values in affected cells after the base
//! UHI calculation runs.

pub mod reductions;
pub mod state;

#[cfg(test)]
mod tests_integration;
#[cfg(test)]
mod tests_unit;

// Re-export all public items so external code can use `uhi_mitigation::Foo`
// without knowing the internal module structure.
pub use reductions::{
    cool_roof_reduction, district_cooling_reduction_at, green_roof_reduction, park_reduction_at,
    total_cell_reduction, tree_uhi_reduction, water_feature_reduction_at,
};
pub use state::UhiMitigationState;

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::trees::TreeGrid;
use crate::urban_heat_island::UhiGrid;
use crate::TickCounter;

// =============================================================================
// Cost constants (publicly re-exported for UI / budget code)
// =============================================================================

/// Cost per building for green roof upgrade.
pub const GREEN_ROOF_COST: f64 = 15_000.0;

/// Cost per building for cool roof upgrade.
pub const COOL_ROOF_COST: f64 = 3_000.0;

/// Cost per cell for cool pavement upgrade.
pub const COOL_PAVEMENT_COST: f64 = 5_000.0;

/// Cost per cell for park placement.
pub const PARK_COST: f64 = 10_000.0;

/// Cost per water feature.
pub const WATER_FEATURE_COST: f64 = 8_000.0;

/// Cost per cell for permeable surfaces.
pub const PERMEABLE_SURFACE_COST: f64 = 4_000.0;

/// Cost per district cooling facility.
pub const DISTRICT_COOLING_COST: f64 = 50_000.0;

/// UHI update frequency -- must match `urban_heat_island::UHI_UPDATE_INTERVAL`.
const UHI_MITIGATION_UPDATE_INTERVAL: u64 = 30;

// =============================================================================
// System
// =============================================================================

/// System that applies UHI mitigation reductions to the `UhiGrid` after the
/// base UHI calculation has run.
///
/// Runs at the same interval as the UHI update system and must be scheduled
/// after `update_uhi_grid`.
pub fn apply_uhi_mitigation(
    tick: Res<TickCounter>,
    mut uhi: ResMut<UhiGrid>,
    tree_grid: Res<TreeGrid>,
    mitigation: Res<UhiMitigationState>,
    buildings: Query<&crate::buildings::Building>,
) {
    if !tick.0.is_multiple_of(UHI_MITIGATION_UPDATE_INTERVAL) {
        return;
    }

    let total_buildings = buildings.iter().count() as u32;
    let avg_green = green_roof_reduction(mitigation.green_roof_count, total_buildings);
    let avg_cool = cool_roof_reduction(mitigation.cool_roof_count, total_buildings);

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let reduction =
                total_cell_reduction(&mitigation, &tree_grid, x, y, avg_green, avg_cool);
            if reduction > 0.0 {
                let current = uhi.get(x, y);
                uhi.set(x, y, current - reduction);
            }
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct UhiMitigationPlugin;

impl Plugin for UhiMitigationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UhiMitigationState>().add_systems(
            FixedUpdate,
            apply_uhi_mitigation
                .after(crate::urban_heat_island::update_uhi_grid)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<UhiMitigationState>();
    }
}

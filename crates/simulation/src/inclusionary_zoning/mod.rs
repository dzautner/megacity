//! Inclusionary Zoning Requirements (ZONE-010).
//!
//! Implements inclusionary zoning as a district policy requiring new residential
//! developments to reserve a percentage of units as affordable housing.
//!
//! Features:
//! - **District policy toggle**: Inclusionary Zoning can be enabled per player-defined district
//! - **Affordable unit percentage**: 10-20% of residential units reserved as affordable housing
//! - **FAR bonus**: +10-20% Floor Area Ratio bonus to offset affordable unit costs
//! - **Affordable units**: House lower-income citizens who would otherwise be priced out
//! - **Profitability impact**: Affects building profitability and construction rate
//!
//! The system tracks which districts have inclusionary zoning enabled and computes
//! per-district effects (effective capacity reduction, FAR bonus, affordable unit counts).

mod config;
mod helpers;
mod systems;
mod tests;

// Re-export all public items so external code sees a flat module.

pub use config::*;
pub use helpers::*;
pub use systems::*;

// =============================================================================
// Plugin
// =============================================================================

use bevy::prelude::*;

pub struct InclusionaryZoningPlugin;

impl Plugin for InclusionaryZoningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InclusionaryZoningState>().add_systems(
            FixedUpdate,
            update_inclusionary_zoning
                .after(crate::buildings::progress_construction)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<InclusionaryZoningState>();
    }
}

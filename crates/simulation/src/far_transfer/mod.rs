//! FAR Bonuses and Transfer of Development Rights (ZONE-012).
//!
//! Implements two complementary FAR enhancement mechanics:
//!
//! **FAR Bonuses**: Developers can exceed the base FAR limit in exchange for
//! public benefits:
//! - Affordable housing inclusion: +20% FAR bonus
//! - Public plaza provision: +10% FAR bonus
//! - Transit contribution: +15% FAR bonus
//!
//! **Transfer of Development Rights (TDR)**: Unused FAR capacity from
//! historic preservation districts and park parcels can be transferred to
//! nearby development sites:
//! - Source parcels: historic districts and park service buildings
//! - Transfer radius: within the same district or adjacent districts
//! - Transferred FAR is removed from the source (prevents double-counting)
//! - Creates a gameplay market for development rights
//!
//! The effective FAR for a cell is:
//!   `base_far + bonus_far + transferred_far`
//!
//! Computed on the slow tick (every ~100 ticks).

pub mod systems;
pub mod types;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_tdr;

use bevy::prelude::*;

// Re-export all public items from sub-modules.
pub use systems::{update_far_bonuses, update_far_transfers};
pub use types::{
    bonus_type_to_bit, calculate_bonus_multiplier, calculate_far_bonus,
    districts_within_transfer_radius, effective_far, eligible_bonuses, is_park_service,
    FarBonusType, FarTransferState, AFFORDABLE_HOUSING_BONUS, HISTORIC_UNUSED_FAR_PER_CELL,
    MAX_BONUS_MULTIPLIER, MAX_TRANSFER_FAR_PER_CELL, PARK_UNUSED_FAR_PER_CELL, PUBLIC_PLAZA_BONUS,
    TRANSFER_DISTRICT_RADIUS, TRANSIT_CONTRIBUTION_BONUS,
};

// =============================================================================
// Plugin
// =============================================================================

pub struct FarTransferPlugin;

impl Plugin for FarTransferPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FarTransferState>().add_systems(
            FixedUpdate,
            (update_far_bonuses, update_far_transfers)
                .chain()
                .after(crate::buildings::building_spawner)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<FarTransferState>();
    }
}

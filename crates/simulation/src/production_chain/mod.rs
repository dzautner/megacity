//! SERV-009: Deep production chain commodity system.
//!
//! Implements multi-stage production chains where raw materials are extracted,
//! processed through intermediate stages, and delivered as finished goods.
//! Chains: grain->flour->bread, timber->lumber->furniture, oil->petroleum->plastics,
//! ore->steel->machinery.

mod systems;
#[cfg(test)]
mod tests;
mod types;

pub use systems::{
    update_chain_disruptions, update_chain_import_export, update_deep_production_chains,
};
pub use types::{
    Commodity, DeepChainBuilding, DeepProductionChainState, ProductionStage, WarehouseBuilding,
};

use bevy::prelude::*;

pub struct ProductionChainPlugin;

impl Plugin for ProductionChainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeepProductionChainState>().add_systems(
            FixedUpdate,
            (
                update_deep_production_chains,
                update_chain_disruptions,
                update_chain_import_export,
            )
                .chain()
                .after(crate::production::update_production_chains)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<DeepProductionChainState>();
    }
}

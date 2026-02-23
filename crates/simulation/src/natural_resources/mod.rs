mod balance;
mod generation;
mod grid;
mod systems;
#[cfg(test)]
mod tests;
mod types;

pub use balance::ResourceBalance;
pub use generation::generate_resources;
pub use grid::ResourceGrid;
pub use systems::update_resource_production;
pub use types::{ResourceDeposit, ResourceType};

use bevy::prelude::*;

pub struct NaturalResourcesPlugin;

impl Plugin for NaturalResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourceGrid>()
            .init_resource::<ResourceBalance>()
            .add_systems(
                FixedUpdate,
                update_resource_production
                    .after(crate::imports_exports::process_trade)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}

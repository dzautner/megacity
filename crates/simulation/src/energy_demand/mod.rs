//! POWER-001: Per-Building Energy Demand Calculation
//!
//! Calculates electricity demand for each building with time-of-day variation,
//! HDD/CDD weather modifiers, and seasonal power multipliers. Aggregates
//! total demand into the `EnergyGrid` resource every 4 ticks.

pub mod systems;
#[cfg(test)]
mod tests;
pub mod types;

pub use systems::{
    aggregate_energy_demand, attach_energy_consumer_to_buildings,
    attach_energy_consumer_to_services, compute_demand_mw, time_of_use_multiplier,
};
pub use types::{EnergyConsumer, EnergyGrid, LoadPriority};

use bevy::prelude::*;

pub struct EnergyDemandPlugin;

impl Plugin for EnergyDemandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnergyGrid>().add_systems(
            FixedUpdate,
            (
                attach_energy_consumer_to_buildings,
                attach_energy_consumer_to_services,
                aggregate_energy_demand
                    .after(attach_energy_consumer_to_buildings)
                    .after(attach_energy_consumer_to_services),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<EnergyGrid>();
    }
}

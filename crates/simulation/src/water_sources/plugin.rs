use bevy::prelude::*;

use super::systems::{aggregate_water_source_supply, replenish_reservoirs, update_water_sources};

pub struct WaterSourcesPlugin;

impl Plugin for WaterSourcesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                update_water_sources,
                aggregate_water_source_supply,
                replenish_reservoirs,
            )
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

//! Plugin registration for the freight traffic system.

use bevy::prelude::*;

use super::systems::{
    compute_freight_demand, generate_freight_trips, move_freight_trucks,
    update_freight_satisfaction,
};
use super::types::FreightTrafficState;

pub struct FreightTrafficPlugin;

impl Plugin for FreightTrafficPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FreightTrafficState>().add_systems(
            FixedUpdate,
            (
                compute_freight_demand,
                generate_freight_trips,
                move_freight_trucks,
                update_freight_satisfaction,
            )
                .chain()
                .after(crate::traffic::update_traffic_density)
                .in_set(crate::SimulationSet::Simulation),
        );
        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<FreightTrafficState>();
    }
}

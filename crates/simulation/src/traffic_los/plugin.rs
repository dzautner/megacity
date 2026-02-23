//! Plugin registration for the Traffic LOS system.

use bevy::prelude::*;

use super::grid::TrafficLosGrid;
use super::segment_los::{LosDistribution, TrafficLosState};
use super::systems::{update_segment_los, update_traffic_los};

pub struct TrafficLosPlugin;

impl Plugin for TrafficLosPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrafficLosGrid>()
            .init_resource::<TrafficLosState>()
            .init_resource::<LosDistribution>()
            .add_systems(
                FixedUpdate,
                (
                    update_traffic_los
                        .after(crate::traffic::update_traffic_density)
                        .in_set(crate::SimulationSet::Simulation),
                    update_segment_los
                        .after(update_traffic_los)
                        .in_set(crate::SimulationSet::Simulation),
                ),
            );

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<TrafficLosGrid>();
        registry.register::<TrafficLosState>();
    }
}

//! Outside connections system: detects and manages highway, railway, sea port,
//! and airport connections linking the city to the wider world.

pub mod detection;
pub mod effects;
pub mod systems;
pub mod types;

mod tests_application;
mod tests_detection_edge_highway;
mod tests_detection_services;
mod tests_effects;
mod tests_types;

// Re-export all public items so callers don't need to change their imports.
pub use effects::ConnectionEffects;
pub use systems::update_outside_connections;
pub use types::{ConnectionStat, ConnectionType, OutsideConnection, OutsideConnections};

use bevy::prelude::*;

pub struct OutsideConnectionsPlugin;

impl Plugin for OutsideConnectionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OutsideConnections>().add_systems(
            FixedUpdate,
            update_outside_connections
                .after(crate::airport::update_airports)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

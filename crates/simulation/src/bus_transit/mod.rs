//! TRAF-005: Bus Public Transit System
//!
//! Implements bus lines as the first public transit type. Buses follow
//! player-drawn routes with stops, pick up citizens, and reduce car traffic.
//!
//! ## Data model
//! - `BusStop`: a stop placed on a road cell (grid coords)
//! - `BusRoute`: an ordered sequence of bus stop IDs
//! - `Bus`: an entity that travels along a route, picking up/dropping off passengers
//! - `BusTransitState`: top-level resource storing all stops, routes, and stats
//!
//! ## Costs
//! - $400/month per route + $100/month per active bus
//! - Fare revenue: $2 per ride
//!
//! ## Citizen mode choice
//! Citizens choose bus when: walk_to_stop + wait + ride + walk_from_stop < drive_time * 1.3

pub mod state;
pub mod systems;
mod tests;
pub mod types;

// Re-export all public items for backward compatibility.
pub use systems::*;
pub use types::*;

use bevy::prelude::*;

// =============================================================================
// Plugin
// =============================================================================

pub struct BusTransitPlugin;

impl Plugin for BusTransitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BusTransitState>().add_systems(
            FixedUpdate,
            (
                update_route_activation,
                spawn_buses,
                update_buses,
                apply_transit_costs,
                simulate_waiting_citizens,
            )
                .chain()
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<BusTransitState>();
    }
}

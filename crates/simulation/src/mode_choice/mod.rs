//! TRAF-007: Citizen Mode Choice (Car/Transit/Walk/Bike)
//!
//! Closes #858
//!
//! Citizens choose a transport mode for each trip based on distance, available
//! infrastructure, and perceived travel time. This is the core mechanic that
//! makes transit investment worthwhile.
//!
//! ## Transport Modes
//!
//! | Mode    | Speed (cells/tick) | Multiplier | Availability                    |
//! |---------|-------------------|------------|----------------------------------|
//! | Walk    | 0.3x base         | 0.30       | Always available, practical <25 cells |
//! | Bike    | 0.6x base         | 0.60       | Requires Path road type nearby   |
//! | Drive   | 1.0x base         | 1.00       | Requires road access (vehicle road) |
//! | Transit | 0.8x base         | 0.80       | Requires transit stop within 15 cells |
//!
//! ## Mode Choice Algorithm
//!
//! For each trip, perceived time = travel_time / comfort_factor:
//! - Walk:    distance / walk_speed, comfort 1.0 (pleasant for short trips)
//! - Bike:    distance / bike_speed, comfort 0.95
//! - Drive:   distance / drive_speed + parking_overhead, comfort 0.90
//! - Transit: walk_to_stop + wait_time + ride_time + walk_from_stop, comfort 0.85
//!
//! Citizens pick the mode with the lowest perceived time from the set of
//! available modes.
//!
//! ## Statistics
//!
//! `ModeShareStats` tracks the percentage of trips by each mode, updated
//! every slow tick (~10 seconds). This feeds into the transportation panel.

use bevy::prelude::*;

pub mod constants;
pub mod evaluation;
pub mod systems;
pub mod types;

mod tests;

// Re-export all public items for backward compatibility.
pub use constants::*;
pub use evaluation::{evaluate_walk, manhattan_distance};
pub use systems::{assign_transport_mode, refresh_infrastructure_cache, update_mode_share_stats};
pub use types::{ChosenTransportMode, ModeInfrastructureCache, ModeShareStats, TransportMode};

// =============================================================================
// Plugin
// =============================================================================

pub struct ModeChoicePlugin;

impl Plugin for ModeChoicePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModeShareStats>()
            .init_resource::<ModeInfrastructureCache>()
            .add_systems(
                FixedUpdate,
                (
                    refresh_infrastructure_cache,
                    assign_transport_mode
                        .after(refresh_infrastructure_cache)
                        .before(crate::movement::process_path_requests),
                    update_mode_share_stats,
                )
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ModeShareStats>();
    }
}

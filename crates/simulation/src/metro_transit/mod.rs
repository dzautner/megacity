//! TRAF-006: Metro/Subway Transit System
//!
//! Implements an underground metro system for high-capacity transit.
//! Metro stations are placed on grid cells (independent of roads) and
//! connected into named lines. Trains run between stations at 80 km/h
//! with 180-passenger capacity.
//!
//! Key mechanics:
//! - Metro stations placed on grid cells (underground, no road required)
//! - Metro lines connect stations in sequence
//! - Citizens walk to nearest station, ride, then walk to destination station
//! - Metro is immune to surface traffic (separate graph)
//! - Stations boost nearby land value (+15-25 in radius)
//! - Construction and maintenance costs tracked in city budget
//! - Ridership statistics updated every slow tick
//!
//! The `MetroTransitState` resource is the source of truth and is persisted
//! via the `Saveable` extension map.

pub mod constants;
pub mod state;
pub mod systems;
mod tests;
pub mod types;

// Re-export all public items so external code can use `metro_transit::Foo`
// without needing to know the internal module structure.
pub use constants::*;
pub use state::MetroTransitState;
pub use systems::*;
pub use types::*;

use bevy::prelude::*;

/// Plugin for the metro transit system.
pub struct MetroTransitPlugin;

impl Plugin for MetroTransitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetroTransitState>().add_systems(
            FixedUpdate,
            (
                update_metro_stats,
                deduct_metro_costs,
                metro_land_value_boost.after(crate::land_value::update_land_value),
            ),
        );

        // Register for save/load via the extension map
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<MetroTransitState>();
    }
}

pub mod demand;
pub mod market;
pub mod stats;
pub mod systems;

#[cfg(test)]
mod tests;

// Re-export all public items for backward compatibility.
pub use demand::ZoneDemand;
pub use market::{compute_market_demand, compute_market_demand_with_params};
pub use stats::{gather_zone_stats, ZoneStats};
pub use systems::{is_adjacent_to_road, update_zone_demand};

use bevy::prelude::*;

pub struct ZonesPlugin;

impl Plugin for ZonesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoneDemand>().add_systems(
            FixedUpdate,
            update_zone_demand
                .after(crate::time_of_day::tick_game_clock)
                .in_set(crate::SimulationSet::PreSim),
        );
    }
}

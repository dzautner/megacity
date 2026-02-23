//! Storm drain and retention pond infrastructure (WATER-006).
//!
//! Storm drains follow road placement and remove runoff capacity (0.5 in/hr each).
//! Retention ponds are 4x4 buildings that store 500,000 gallons, slowly releasing stored water.
//! Rain gardens are 1x1 buildings that absorb 100% of local cell runoff and 50% from 4 neighbors.
//! The system tracks drainage network capacity vs. runoff, triggering flooding when exceeded.

pub mod types;

mod systems;
mod tests;

pub use systems::update_storm_drainage;
pub use types::{StormDrainageInfrastructure, StormDrainageState, StormDrainageType};

use bevy::prelude::*;

pub struct StormDrainagePlugin;

impl Plugin for StormDrainagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StormDrainageState>().add_systems(
            FixedUpdate,
            update_storm_drainage
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

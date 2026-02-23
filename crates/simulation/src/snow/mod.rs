//! Snow accumulation, melting, and plowing system (WEATHER-014).
//!
//! During winter precipitation events when temperature < 0C (32F), snow
//! accumulates on the grid. Snow affects traffic speed, heating demand,
//! and visual rendering. A snow plowing service clears roads at a cost,
//! prioritizing highways > arterials > local roads.
//!
//! The `SnowGrid` resource tracks per-cell snow depth in inches. The
//! `SnowPlowingState` resource tracks plowing service state and costs.

pub mod systems;
mod tests;
pub mod types;

pub use systems::{
    snow_accumulation_amount, snow_heating_modifier, snow_melt_amount, snow_speed_multiplier,
    update_snow, update_snow_plowing,
};
pub use types::{SnowGrid, SnowPlowingState, SnowStats};

use bevy::prelude::*;

pub struct SnowPlugin;

impl Plugin for SnowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SnowGrid>()
            .init_resource::<SnowPlowingState>()
            .init_resource::<SnowStats>()
            .add_systems(
                FixedUpdate,
                (update_snow, update_snow_plowing)
                    .chain()
                    .after(crate::weather::update_weather)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}

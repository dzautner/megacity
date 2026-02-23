//! Weather simulation module.
//!
//! Handles seasons, temperature, precipitation, atmospheric state, and
//! construction modifiers driven by weather conditions and climate zones.

pub mod climate;
pub mod state;
pub mod systems;
mod tests_climate;
mod tests_systems;
mod tests_types;
pub mod types;

// Re-export all public items so callers can use `weather::Weather`, etc.
pub use climate::{ClimateZone, SeasonClimateParams};
pub use state::{ConstructionModifiers, Weather};
pub use systems::{
    diurnal_factor, precipitation_intensity_for_event, update_construction_modifiers,
    update_precipitation, update_weather,
};
pub use types::{
    is_extreme_weather, PrecipitationCategory, Season, WeatherChangeEvent, WeatherCondition,
    WeatherEvent,
};

use bevy::prelude::*;

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Weather>()
            .init_resource::<ClimateZone>()
            .init_resource::<ConstructionModifiers>()
            .add_event::<WeatherChangeEvent>()
            .add_systems(
                FixedUpdate,
                (
                    update_weather,
                    update_precipitation,
                    update_construction_modifiers,
                )
                    .after(crate::imports_exports::process_trade)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}

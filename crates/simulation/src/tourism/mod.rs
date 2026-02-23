mod systems;
mod types;
mod weather;

#[cfg(test)]
mod tests;

pub use systems::update_tourism;
pub use types::{Tourism, TourismWeatherEvent};
pub use weather::{
    seasonal_tourism_multiplier, tourism_seasonal_modifier, tourism_weather_event,
    weather_tourism_multiplier,
};

use bevy::prelude::*;

pub struct TourismPlugin;

impl Plugin for TourismPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tourism>().add_systems(
            FixedUpdate,
            update_tourism
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

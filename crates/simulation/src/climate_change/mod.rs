//! Long-term climate change from cumulative CO2 emissions (WEATHER-016).
//!
//! Tracks cumulative CO2 emissions from fossil fuel power plants and industrial
//! buildings. As emissions accumulate past thresholds, long-term climate effects
//! are triggered: temperature increases, more extreme weather events, sea level
//! rise (permanent flooding of low-elevation coastal cells), and longer droughts.
//!
//! CO2 emission rates per MWh:
//! - Coal power plant: 1.0 ton/MWh
//! - Gas power plant:  0.4 ton/MWh
//! - Oil power plant:  0.8 ton/MWh
//! - Biomass:          0.0 ton/MWh (carbon neutral)
//!
//! Climate thresholds (cumulative tons):
//! - 1,000,000 tons: +1F average temperature increase
//! - 5,000,000 tons: +2F average temperature increase
//! - 20,000,000 tons: +3F average temperature increase
//!
//! Effects:
//! - Disaster frequency increases by +10% per 1F increase
//! - At +3F, lowest-elevation water-adjacent cells flood permanently
//! - Drought duration extends with temperature increase

pub mod calculations;
pub mod constants;
pub mod state;
mod systems;

#[cfg(test)]
mod tests;

// Re-export all public items for backward compatibility.
pub use calculations::*;
pub use constants::*;
pub use state::*;
pub use systems::*;

use bevy::prelude::*;

/// Plugin that registers climate change resources and systems.
pub struct ClimateChangePlugin;

impl Plugin for ClimateChangePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClimateState>().add_systems(
            FixedUpdate,
            yearly_climate_assessment
                .after(crate::weather::update_weather)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ClimateState>();
    }
}

//! SVC-021: Wind-Aware Gaussian Plume Pollution Dispersion
//!
//! Replaces simple isotropic neighbor diffusion with a wind-aware Gaussian plume
//! model. Pollution from each source spreads downwind in a cone pattern, with
//! concentration following a simplified Gaussian distribution in the crosswind
//! direction. Wind direction from [`WindState`] determines dispersion direction.

mod config;
mod dispersion;
mod system;
#[cfg(test)]
mod tests;

pub use config::WindPollutionConfig;
pub use system::update_pollution_gaussian_plume;

use bevy::prelude::*;
use save::SaveableAppExt;

pub struct WindPollutionPlugin;

impl Plugin for WindPollutionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindPollutionConfig>()
            .register_saveable::<WindPollutionConfig>();
    }
}

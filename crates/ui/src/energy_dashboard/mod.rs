//! Energy Dashboard UI Panel (POWER-019).
//!
//! Displays a comprehensive energy dashboard showing:
//! - Total demand (MW), total supply (MW), reserve margin (%)
//! - Blackout status indicator (green/yellow/red)
//! - Current electricity price ($/kWh) with time-of-use period
//! - Generation mix: bar showing MW from each plant type (coal, gas, wind, battery)
//! - History graph: demand and supply over last 24 game-hours (ring buffer)

mod panels;
mod tests;
pub mod types;
mod ui_system;

use bevy::prelude::*;
use simulation::app_state::AppState;

pub use types::{EnergyDashboardVisible, EnergyHistory};
pub use ui_system::{energy_dashboard_ui, record_energy_history};

/// Plugin that registers the energy dashboard UI.
pub struct EnergyDashboardPlugin;

impl Plugin for EnergyDashboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnergyDashboardVisible>()
            .init_resource::<EnergyHistory>()
            .add_systems(
                Update,
                (record_energy_history, energy_dashboard_ui)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

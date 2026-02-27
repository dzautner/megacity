//! Keyboard shortcuts for toggling energy, water, and waste dashboards.
//!
//! Default keybindings: F3 (Energy), F4 (Water), F6 (Waste).
//! These are configurable through the keybindings settings panel.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use simulation::app_state::AppState;

use crate::energy_dashboard::EnergyDashboardVisible;
use crate::waste_dashboard::WasteDashboardVisible;
use crate::water_dashboard::WaterDashboardVisible;

/// Toggles dashboard visibility when keybinds are pressed.
/// F3 = Energy Dashboard, F4 = Water Dashboard, F6 = Waste Dashboard.
/// Keys are ignored when egui has keyboard focus (e.g. text input).
pub fn dashboard_keybinds(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut energy: ResMut<EnergyDashboardVisible>,
    mut water: ResMut<WaterDashboardVisible>,
    mut waste: ResMut<WasteDashboardVisible>,
    mut contexts: EguiContexts,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    if bindings.toggle_energy_dashboard.just_pressed(&keyboard) {
        energy.0 = !energy.0;
    }
    if bindings.toggle_water_dashboard.just_pressed(&keyboard) {
        water.0 = !water.0;
    }
    if bindings.toggle_waste_dashboard.just_pressed(&keyboard) {
        waste.0 = !waste.0;
    }
}

/// Plugin that registers the dashboard toggle keyboard shortcuts.
pub struct DashboardTogglesPlugin;

impl Plugin for DashboardTogglesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            dashboard_keybinds.run_if(in_state(AppState::Playing)),
        );
    }
}

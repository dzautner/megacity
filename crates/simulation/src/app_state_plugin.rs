//! Plugin that registers the [`AppState`] state machine.
//!
//! Kept in its own file following the one-plugin-per-file convention.

use bevy::prelude::*;

use crate::app_state::AppState;

/// Plugin that initializes the [`AppState`] state resource.
///
/// The actual run-condition gating of simulation sets is handled in
/// [`SimulationPlugin::build`](crate::SimulationPlugin) alongside the
/// existing [`SaveLoadState`](crate::SaveLoadState) guards.
///
/// Note: the state defaults to `MainMenu`. Callers (the app binary or the
/// test harness) must call `app.insert_state(AppState::Playing)` **before**
/// adding `SimulationPlugin` to start in gameplay mode.
pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
    }
}

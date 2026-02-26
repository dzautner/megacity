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
/// Until a main-menu UI is implemented, a `Startup` system automatically
/// transitions from `MainMenu` to `Playing` so the game launches directly
/// into gameplay as before.
pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
        // Auto-transition to Playing on startup so the game works without a
        // main-menu screen. A future main-menu feature will remove this
        // system and let the player start from the menu.
        app.add_systems(Startup, auto_enter_playing);
    }
}

/// Transition from `MainMenu` to `Playing` on startup.
///
/// This is a temporary bridge system: once a real main menu is added, this
/// system should be removed and the menu UI should trigger the transition
/// instead.
fn auto_enter_playing(
    current: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if *current.get() == AppState::MainMenu {
        next.set(AppState::Playing);
    }
}

//! P0-09: Unify pause authority — keep `AppState` in sync with `GameClock.paused`.
//!
//! Two independent pause mechanisms exist:
//! - `GameClock.paused` — toggled by toolbar speed controls and keybinds
//! - `AppState::Paused` — toggled by the ESC pause menu
//!
//! This module adds a sync system that watches `GameClock.paused` and transitions
//! `AppState` accordingly, so both mechanisms always agree.  When `GameClock.paused`
//! is set to `true` while `AppState` is `Playing`, we transition to `Paused`.
//! When `GameClock.paused` is set to `false` while `AppState` is `Paused`, we
//! transition back to `Playing`.
//!
//! The sync is one-directional: `GameClock.paused` is the source of truth.
//! Code that transitions `AppState` (e.g. the pause menu) must also set
//! `GameClock.paused` — which it already does.

use bevy::prelude::*;

use crate::app_state::AppState;
use crate::time_of_day::GameClock;

/// Keeps `AppState` in sync with `GameClock.paused`.
///
/// Runs every frame (not gated by `AppState::Playing`) so it can detect
/// when the toolbar or speed keybinds pause/unpause via `GameClock` and
/// mirror the change to `AppState`.
fn sync_pause_state(
    clock: Res<GameClock>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match app_state.get() {
        AppState::Playing if clock.paused => {
            next_state.set(AppState::Paused);
        }
        AppState::Paused if !clock.paused => {
            next_state.set(AppState::Playing);
        }
        _ => {}
    }
}

pub struct PauseSyncPlugin;

impl Plugin for PauseSyncPlugin {
    fn build(&self, app: &mut App) {
        // Run in Update without any state gate — this system must observe
        // transitions *from* any gameplay state.
        app.add_systems(Update, sync_pause_state);
    }
}

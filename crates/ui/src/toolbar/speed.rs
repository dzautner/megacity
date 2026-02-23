use bevy::prelude::*;
use bevy_egui::EguiContexts;

use simulation::time_of_day::GameClock;

// ---------------------------------------------------------------------------
// Simulation speed keybinds (Space / 1 / 2 / 3)
// ---------------------------------------------------------------------------

/// Handles keyboard shortcuts for simulation speed control:
/// - Space: toggle pause / unpause
/// - 1: normal speed (1x)
/// - 2: fast speed (2x)
/// - 3: fastest speed (4x)
pub fn speed_keybinds(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clock: ResMut<GameClock>,
    mut contexts: EguiContexts,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    if bindings.toggle_pause.just_pressed(&keyboard) {
        clock.paused = !clock.paused;
    }
    if bindings.speed_normal.just_pressed(&keyboard) {
        clock.speed = 1.0;
        clock.paused = false;
    }
    if bindings.speed_fast.just_pressed(&keyboard) {
        clock.speed = 2.0;
        clock.paused = false;
    }
    if bindings.speed_fastest.just_pressed(&keyboard) {
        clock.speed = 4.0;
        clock.paused = false;
    }
}

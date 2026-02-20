use bevy::prelude::*;
use bevy_egui::EguiContexts;

use super::{AdvisorVisible, ChartsVisible, JournalVisible, PoliciesVisible};

/// Toggles UI panel visibility when keybinds are pressed.
/// J = Event Journal, C = Charts, A = Advisors, P = Policies.
/// Keys are ignored when egui has keyboard focus (e.g. text input).
#[allow(clippy::too_many_arguments)]
pub fn panel_keybinds(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut journal: ResMut<JournalVisible>,
    mut charts: ResMut<ChartsVisible>,
    mut advisor: ResMut<AdvisorVisible>,
    mut policies: ResMut<PoliciesVisible>,
    mut contexts: EguiContexts,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    if bindings.toggle_journal.just_pressed(&keyboard) {
        journal.0 = !journal.0;
    }
    if bindings.toggle_charts.just_pressed(&keyboard) {
        charts.0 = !charts.0;
    }
    if bindings.toggle_advisor.just_pressed(&keyboard) {
        advisor.0 = !advisor.0;
    }
    if bindings.toggle_policies.just_pressed(&keyboard) {
        policies.0 = !policies.0;
    }
    // B key is now used for the Bulldoze tool shortcut (issue #905).
    // Budget panel is accessible via the toolbar UI.
}

/// Keyboard shortcuts for quick save (Ctrl+S), quick load (Ctrl+L), and new game (Ctrl+N).
/// Skipped when egui wants keyboard input (e.g. a text field is focused).
#[allow(clippy::too_many_arguments)]
pub fn quick_save_load_keybinds(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut save_events: EventWriter<save::SaveGameEvent>,
    mut load_events: EventWriter<save::LoadGameEvent>,
    mut new_game_events: EventWriter<save::NewGameEvent>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    if bindings.quick_save.just_pressed(&keyboard) {
        save_events.send(save::SaveGameEvent);
    }
    if bindings.quick_load.just_pressed(&keyboard) {
        load_events.send(save::LoadGameEvent);
    }
    if bindings.new_game.just_pressed(&keyboard) {
        new_game_events.send(save::NewGameEvent);
    }
}

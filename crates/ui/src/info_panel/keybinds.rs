use bevy::prelude::*;
use bevy_egui::EguiContexts;

use super::{AdvisorVisible, BudgetPanelVisible, ChartsVisible, JournalVisible, PoliciesVisible};

/// Toggles UI panel visibility when keybinds are pressed.
/// J = Event Journal, C = Charts, A = Advisors, P = Policies, B = Budget.
/// Keys are ignored when egui has keyboard focus (e.g. text input).
pub fn panel_keybinds(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut journal: ResMut<JournalVisible>,
    mut charts: ResMut<ChartsVisible>,
    mut advisor: ResMut<AdvisorVisible>,
    mut policies: ResMut<PoliciesVisible>,
    mut budget_panel: ResMut<BudgetPanelVisible>,
    mut contexts: EguiContexts,
) {
    // Don't toggle panels when a text field or other egui widget wants keyboard input
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyJ) {
        journal.0 = !journal.0;
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        charts.0 = !charts.0;
    }
    if keyboard.just_pressed(KeyCode::KeyA) {
        advisor.0 = !advisor.0;
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        policies.0 = !policies.0;
    }
    if keyboard.just_pressed(KeyCode::KeyB) {
        budget_panel.0 = !budget_panel.0;
    }
}

/// Keyboard shortcuts for quick save (Ctrl+S), quick load (Ctrl+L), and new game (Ctrl+N).
/// Skipped when egui wants keyboard input (e.g. a text field is focused).
pub fn quick_save_load_keybinds(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut save_events: EventWriter<save::SaveGameEvent>,
    mut load_events: EventWriter<save::LoadGameEvent>,
    mut new_game_events: EventWriter<save::NewGameEvent>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyS) {
        save_events.send(save::SaveGameEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        load_events.send(save::LoadGameEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyN) {
        new_game_events.send(save::NewGameEvent);
    }
}

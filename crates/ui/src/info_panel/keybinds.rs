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

/// Keyboard shortcuts for quick save (F5) and quick load (F9) to `quicksave.bin`,
/// plus new game (Ctrl+N).
/// Skipped when egui wants keyboard input (e.g. a text field is focused).
#[allow(clippy::too_many_arguments)]
pub fn quick_save_load_keybinds(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut save_events: EventWriter<save::SaveGameEvent>,
    mut load_events: EventWriter<save::LoadGameEvent>,
    mut new_game_events: EventWriter<save::NewGameEvent>,
    mut notifications: EventWriter<simulation::notifications::NotificationEvent>,
    mut path_override: ResMut<save::PendingSavePath>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    if bindings.quick_save.just_pressed(&keyboard) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            path_override.0 = Some(save::quicksave_file_path());
        }
        save_events.send(save::SaveGameEvent);
        notifications.send(simulation::notifications::NotificationEvent {
            text: "Quick saved".to_string(),
            priority: simulation::notifications::NotificationPriority::Info,
            location: None,
        });
    }
    if bindings.quick_load.just_pressed(&keyboard) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let qs_path = save::quicksave_file_path();
            if !std::path::Path::new(&qs_path).exists() {
                notifications.send(simulation::notifications::NotificationEvent {
                    text: "No quicksave found".to_string(),
                    priority: simulation::notifications::NotificationPriority::Warning,
                    location: None,
                });
                return;
            }
            path_override.0 = Some(qs_path);
        }
        load_events.send(save::LoadGameEvent);
        notifications.send(simulation::notifications::NotificationEvent {
            text: "Quick loaded".to_string(),
            priority: simulation::notifications::NotificationPriority::Info,
            location: None,
        });
    }
    if bindings.new_game.just_pressed(&keyboard) {
        new_game_events.send(save::NewGameEvent);
    }
}

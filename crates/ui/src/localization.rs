use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::localization::{LocalizationState, LOCALE_NAMES, SUPPORTED_LOCALES};

// =============================================================================
// Resources
// =============================================================================

/// Controls visibility of the language selector window.
#[derive(Resource, Default)]
pub struct LanguageSelectorVisible(pub bool);

// =============================================================================
// Systems
// =============================================================================

/// Keyboard shortcut to toggle the language selector (L key while holding Ctrl).
pub fn language_selector_keybind(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<LanguageSelectorVisible>,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyL) {
        visible.0 = !visible.0;
    }
}

/// Draw the language selector window using egui.
pub fn language_selector_ui(
    mut contexts: EguiContexts,
    mut visible: ResMut<LanguageSelectorVisible>,
    mut localization: ResMut<LocalizationState>,
) {
    if !visible.0 {
        return;
    }

    let mut open = visible.0;
    egui::Window::new(localization.t("ui.language"))
        .open(&mut open)
        .resizable(false)
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(localization.t("ui.language"));
            ui.separator();

            let current_locale = localization.active_locale.clone();

            for (i, locale_code) in SUPPORTED_LOCALES.iter().enumerate() {
                let display_name = LOCALE_NAMES[i];
                let is_selected = current_locale == *locale_code;

                if ui
                    .selectable_label(is_selected, format!("{} ({})", display_name, locale_code))
                    .clicked()
                {
                    localization.set_locale(locale_code);
                }
            }

            ui.separator();
            ui.label(
                egui::RichText::new(format!(
                    "{}: {}",
                    localization.t("ui.language"),
                    localization.active_locale_name()
                ))
                .small(),
            );
        });

    visible.0 = open;
}

// =============================================================================
// Plugin
// =============================================================================

pub struct LocalizationUiPlugin;

impl Plugin for LocalizationUiPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: LocalizationState is registered with SaveableRegistry in
        // LocalizationPlugin (simulation crate), not here.
        app.init_resource::<LanguageSelectorVisible>()
            .add_systems(Update, (language_selector_keybind, language_selector_ui));
    }
}

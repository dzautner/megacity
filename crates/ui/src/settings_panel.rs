//! Settings panel UI (UX-039 Colorblind Accessibility).
//!
//! Provides an egui window with colorblind mode selection and other
//! accessibility settings. Toggled via the F9 key.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::colorblind::{ColorblindMode, ColorblindSettings};

use crate::keybindings_panel::KeybindingsPanelVisible;

// =============================================================================
// Resources
// =============================================================================

/// Whether the settings panel is visible.
#[derive(Resource, Default)]
pub struct SettingsPanelVisible(pub bool);

// =============================================================================
// Systems
// =============================================================================

/// Toggles the settings panel with F9.
pub fn settings_panel_keybind(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<SettingsPanelVisible>,
    mut contexts: EguiContexts,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }
    if bindings.toggle_settings.just_pressed(&keyboard) {
        visible.0 = !visible.0;
    }
}

/// Renders the settings panel window.
pub fn settings_panel_ui(
    mut contexts: EguiContexts,
    mut visible: ResMut<SettingsPanelVisible>,
    mut cb_settings: ResMut<ColorblindSettings>,
    mut kb_visible: ResMut<KeybindingsPanelVisible>,
) {
    if !visible.0 {
        return;
    }

    let mut open = true;
    egui::Window::new("Settings")
        .open(&mut open)
        .resizable(false)
        .default_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;

            // --- Accessibility section ---
            ui.heading("Accessibility");
            ui.separator();

            ui.label("Colorblind Mode:");
            ui.add_space(4.0);

            let current = cb_settings.mode;
            for mode in ColorblindMode::ALL {
                let selected = current == mode;
                if ui.radio(selected, mode.label()).clicked() && !selected {
                    cb_settings.mode = mode;
                }
            }

            ui.add_space(8.0);

            // Show a description of what the current mode does
            let description = match current {
                ColorblindMode::Normal => "Standard color vision. No adjustments applied.",
                ColorblindMode::Protanopia => {
                    "Adapted for red-blindness. Traffic indicators use blue-to-orange ramp instead of green-to-red."
                }
                ColorblindMode::Deuteranopia => {
                    "Adapted for green-blindness. Traffic indicators use blue-to-orange ramp instead of green-to-red."
                }
                ColorblindMode::Tritanopia => {
                    "Adapted for blue-blindness. Indicators use teal-to-magenta ramp instead of blue-to-yellow."
                }
            };
            ui.label(
                egui::RichText::new(description)
                    .small()
                    .color(egui::Color32::from_gray(160)),
            );

            ui.add_space(16.0);

            // --- Controls section ---
            ui.heading("Controls");
            ui.separator();

            if ui.button("Customize Keybindings...").clicked() {
                kb_visible.0 = true;
            }

            ui.add_space(16.0);

            // --- About / Version section ---
            ui.heading("About");
            ui.separator();

            let build_version = env!("CARGO_PKG_VERSION");
            let save_version = save::serialization::CURRENT_SAVE_VERSION;

            ui.horizontal(|ui| {
                ui.label("Build version:");
                ui.label(
                    egui::RichText::new(build_version)
                        .strong()
                        .color(egui::Color32::from_gray(200)),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Save format:");
                ui.label(
                    egui::RichText::new(format!("v{save_version}"))
                        .strong()
                        .color(egui::Color32::from_gray(200)),
                );
            });
        });

    if !open {
        visible.0 = false;
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct SettingsPanelPlugin;

impl Plugin for SettingsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsPanelVisible>()
            .add_systems(Update, (settings_panel_keybind, settings_panel_ui));
    }
}

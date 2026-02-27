//! PLAY-024: In-game help overlay showing all keybindings grouped by category.
//!
//! Toggled via F1 (or the user's configured `toggle_help` binding).
//! Displays a read-only reference of all current keybindings in a centered
//! egui window, grouped by category. Can be dismissed with the Close button,
//! Escape, or pressing F1 again.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::app_state::AppState;
use simulation::keybindings::{BindableAction, KeyBindings};

/// Whether the help overlay is currently visible.
#[derive(Resource, Default)]
pub struct HelpOverlayOpen(pub bool);

/// System: toggle the help overlay when the configured key is pressed.
fn toggle_help_overlay(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    bindings: Res<KeyBindings>,
    mut open: ResMut<HelpOverlayOpen>,
) {
    let Some(keys) = keys else {
        return;
    };
    if bindings.toggle_help.just_pressed(&keys) {
        open.0 = !open.0;
    }
    // Also close on Escape when open
    if open.0 && bindings.escape.just_pressed(&keys) {
        open.0 = false;
    }
}

/// System: render the help overlay egui window.
fn help_overlay_ui(
    mut contexts: EguiContexts,
    mut open: ResMut<HelpOverlayOpen>,
    bindings: Res<KeyBindings>,
) {
    if !open.0 {
        return;
    }

    let mut should_close = false;

    egui::Window::new("Help — Keyboard Shortcuts")
        .collapsible(false)
        .resizable(true)
        .default_width(480.0)
        .min_width(380.0)
        .max_height(600.0)
        .vscroll(true)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;

            ui.colored_label(
                egui::Color32::from_gray(160),
                "Press F1 or Escape to close. Bindings can be changed in Settings.",
            );
            ui.add_space(8.0);

            let mut current_category = "";

            for &action in BindableAction::ALL {
                let category = action.category();
                if category != current_category {
                    if !current_category.is_empty() {
                        ui.add_space(6.0);
                    }
                    ui.heading(category);
                    ui.separator();
                    current_category = category;
                }

                let binding = bindings.get(action);
                ui.horizontal(|ui| {
                    ui.colored_label(
                        egui::Color32::from_gray(220),
                        action.label(),
                    );
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(130, 200, 255),
                                egui::RichText::new(binding.display_label())
                                    .monospace(),
                            );
                        },
                    );
                });
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        open.0 = false;
    }
}

pub struct HelpOverlayPlugin;

impl Plugin for HelpOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HelpOverlayOpen>().add_systems(
            Update,
            (toggle_help_overlay, help_overlay_ui)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

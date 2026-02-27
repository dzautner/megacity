//! Keybindings settings panel (UX-035).
//!
//! Provides an egui window for viewing current keybindings, click-to-rebind,
//! conflict detection with warnings, and a "Reset to Defaults" button.
//! Accessible from the Settings panel.

use bevy::prelude::*;
use simulation::app_state::AppState;
use bevy_egui::{egui, EguiContexts};

use simulation::keybindings::{BindableAction, KeyBindings, RebindState};

/// Whether the keybindings settings panel is visible.
#[derive(Resource, Default)]
pub struct KeybindingsPanelVisible(pub bool);

/// Renders the keybindings settings panel.
pub fn keybindings_panel_ui(
    mut contexts: EguiContexts,
    mut visible: ResMut<KeybindingsPanelVisible>,
    mut bindings: ResMut<KeyBindings>,
    mut rebind_state: ResMut<RebindState>,
) {
    if !visible.0 {
        return;
    }

    let conflicts = bindings.find_conflicts();
    let conflict_actions: Vec<BindableAction> =
        conflicts.iter().flat_map(|(a, b)| [*a, *b]).collect();

    let mut open = true;
    egui::Window::new("Keybindings")
        .open(&mut open)
        .resizable(true)
        .default_width(420.0)
        .min_width(350.0)
        .max_height(600.0)
        .vscroll(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;

            if !conflicts.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 180, 50),
                    format!(
                        "Warning: {} conflict(s) detected (same key in same category)",
                        conflicts.len()
                    ),
                );
                ui.add_space(4.0);
            }

            if rebind_state.awaiting.is_some() {
                ui.colored_label(
                    egui::Color32::from_rgb(100, 200, 255),
                    "Press a key to assign (Esc to cancel)...",
                );
                ui.add_space(4.0);
            }

            let mut current_category = "";

            for &action in BindableAction::ALL {
                let category = action.category();
                if category != current_category {
                    if !current_category.is_empty() {
                        ui.add_space(8.0);
                    }
                    ui.heading(category);
                    ui.separator();
                    current_category = category;
                }

                let binding = bindings.get(action);
                let is_awaiting = rebind_state.awaiting == Some(action);
                let has_conflict = conflict_actions.contains(&action);

                ui.horizontal(|ui| {
                    let label_color = if has_conflict {
                        egui::Color32::from_rgb(255, 180, 50)
                    } else {
                        egui::Color32::from_gray(220)
                    };
                    ui.colored_label(label_color, action.label());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let button_text = if is_awaiting {
                            "[ ... ]".to_string()
                        } else {
                            binding.display_label()
                        };

                        let button_color = if is_awaiting {
                            egui::Color32::from_rgb(100, 200, 255)
                        } else if has_conflict {
                            egui::Color32::from_rgb(255, 180, 50)
                        } else {
                            egui::Color32::from_gray(180)
                        };

                        let btn = egui::Button::new(
                            egui::RichText::new(&button_text)
                                .color(button_color)
                                .monospace(),
                        )
                        .min_size(egui::vec2(100.0, 0.0));

                        if ui.add(btn).clicked() {
                            if is_awaiting {
                                rebind_state.awaiting = None;
                            } else {
                                rebind_state.awaiting = Some(action);
                            }
                        }
                    });
                });
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Reset to Defaults").clicked() {
                    *bindings = KeyBindings::default();
                    rebind_state.awaiting = None;
                }

                if rebind_state.awaiting.is_some() && ui.button("Cancel Rebind").clicked() {
                    rebind_state.awaiting = None;
                }
            });
        });

    if !open {
        visible.0 = false;
        rebind_state.awaiting = None;
    }
}

pub struct KeybindingsPanelPlugin;

impl Plugin for KeybindingsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeybindingsPanelVisible>()
            .add_systems(
                Update,
                keybindings_panel_ui.run_if(in_state(AppState::Playing)),
            );
    }
}

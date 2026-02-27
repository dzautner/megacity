//! Actionable Advisor Tips UI (UX-045).
//!
//! Displays advisor messages as a non-intrusive notification icon in the
//! bottom-right corner. Clicking the icon expands a tips panel where each
//! tip has a "Show Location" button (jumps camera) and a "Dismiss" button
//! (permanently hides that tip type). Tips only appear when their triggering
//! condition is active.

use bevy::prelude::*;
use simulation::app_state::AppState;
use bevy_egui::{egui, EguiContexts};

use rendering::camera::OrbitCamera;
use simulation::advisors::{AdvisorJumpToLocation, AdvisorPanel, DismissedAdvisorTips};
use simulation::config::CELL_SIZE;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Controls whether the advisor tips panel is expanded.
#[derive(Resource, Default)]
pub struct AdvisorTipsPanelOpen(pub bool);

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Renders the advisor notification icon and expanded tips panel.
///
/// The icon sits in the bottom-right corner and shows a badge with the
/// number of active tips. Clicking it toggles the expanded panel.
#[allow(clippy::too_many_arguments)]
pub fn advisor_tips_ui(
    mut contexts: EguiContexts,
    advisor_panel: Res<AdvisorPanel>,
    mut dismissed: ResMut<DismissedAdvisorTips>,
    mut panel_open: ResMut<AdvisorTipsPanelOpen>,
    mut jump_events: EventWriter<AdvisorJumpToLocation>,
) {
    let tip_count = advisor_panel.messages.len();

    // --- Notification Icon (bottom-right) ---
    egui::Window::new("advisor_tips_icon")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-8.0, -8.0))
        .fixed_size(egui::vec2(48.0, 48.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                let icon_text = if tip_count > 0 {
                    format!("[{}]", tip_count)
                } else {
                    "[-]".to_string()
                };

                let icon_color = if tip_count > 0 {
                    // Highest priority color
                    advisor_panel
                        .messages
                        .first()
                        .map(|m| priority_color(m.priority))
                        .unwrap_or(egui::Color32::from_rgb(150, 150, 150))
                } else {
                    egui::Color32::from_rgb(150, 150, 150)
                };

                let button =
                    egui::Button::new(egui::RichText::new(&icon_text).size(20.0).color(icon_color))
                        .min_size(egui::vec2(44.0, 44.0));

                let response = ui.add(button);
                if response.clicked() {
                    panel_open.0 = !panel_open.0;
                }
                if response.hovered() {
                    response.on_hover_text(if tip_count > 0 {
                        format!("{} advisor tip(s) -- click to expand", tip_count)
                    } else {
                        "No advisor tips -- city is running smoothly".to_string()
                    });
                }
            });
        });

    // --- Expanded Tips Panel ---
    if !panel_open.0 {
        return;
    }

    egui::Window::new("Advisor Tips")
        .default_open(true)
        .resizable(true)
        .default_width(380.0)
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-8.0, -64.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.heading("Advisor Tips");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Close").clicked() {
                        panel_open.0 = false;
                    }
                });
            });
            ui.separator();

            if advisor_panel.messages.is_empty() {
                ui.label("No active advisor tips. Your city is running smoothly!");
                ui.add_space(8.0);
            } else {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        // Clone messages to avoid borrow conflict
                        let messages: Vec<_> = advisor_panel.messages.clone();
                        for msg in &messages {
                            let pcolor = priority_color(msg.priority);

                            // Header row: colored dot + advisor name + priority
                            ui.horizontal(|ui| {
                                let (dot_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(10.0, 10.0),
                                    egui::Sense::hover(),
                                );
                                let painter = ui.painter_at(dot_rect);
                                painter.circle_filled(dot_rect.center(), 4.0, pcolor);

                                ui.colored_label(
                                    pcolor,
                                    format!(
                                        "[{}] {}",
                                        msg.advisor_type.icon(),
                                        msg.advisor_type.name()
                                    ),
                                );
                            });

                            // Message and suggestion
                            ui.label(&msg.message);
                            ui.small(&msg.suggestion);

                            // Action buttons row
                            ui.horizontal(|ui| {
                                // "Show Location" button (only if location is available)
                                if let Some((gx, gy)) = msg.location {
                                    if ui
                                        .small_button("Show Location")
                                        .on_hover_text("Jump camera to this location")
                                        .clicked()
                                    {
                                        jump_events.send(AdvisorJumpToLocation {
                                            grid_x: gx,
                                            grid_y: gy,
                                        });
                                    }
                                }

                                // "Dismiss" button
                                let tip_id = msg.tip_id;
                                if ui
                                    .small_button("Dismiss")
                                    .on_hover_text(format!(
                                        "Permanently hide '{}' tips",
                                        tip_id.label()
                                    ))
                                    .clicked()
                                {
                                    dismissed.dismiss(tip_id);
                                }
                            });

                            ui.add_space(6.0);
                            ui.separator();
                        }
                    });
            }

            // "Restore All" button at the bottom if there are dismissed tips
            if !dismissed.dismissed.is_empty() {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.small(format!(
                        "{} tip type(s) dismissed",
                        dismissed.dismissed.len()
                    ));
                    if ui.small_button("Restore All").clicked() {
                        dismissed.restore_all();
                    }
                });
            }
        });
}

/// Handles `AdvisorJumpToLocation` events by moving the orbit camera focus
/// to the requested grid position.
pub fn handle_advisor_jump(
    mut events: EventReader<AdvisorJumpToLocation>,
    mut orbit: ResMut<OrbitCamera>,
) {
    for event in events.read() {
        // Convert grid coords to world position
        let world_x = event.grid_x as f32 * CELL_SIZE + CELL_SIZE / 2.0;
        let world_z = event.grid_y as f32 * CELL_SIZE + CELL_SIZE / 2.0;

        orbit.focus.x = world_x;
        orbit.focus.z = world_z;
        // Zoom in to a comfortable viewing distance
        orbit.distance = orbit.distance.min(500.0);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn priority_color(priority: u8) -> egui::Color32 {
    match priority {
        5 => egui::Color32::from_rgb(220, 50, 50),   // red
        4 => egui::Color32::from_rgb(230, 150, 30),  // orange
        3 => egui::Color32::from_rgb(220, 200, 50),  // yellow
        2 => egui::Color32::from_rgb(50, 130, 220),  // blue
        _ => egui::Color32::from_rgb(150, 150, 150), // grey
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct AdvisorTipsPlugin;

impl Plugin for AdvisorTipsPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: DismissedAdvisorTips is registered with SaveableRegistry in
        // AdvisorsPlugin (simulation crate), not here.
        app.init_resource::<AdvisorTipsPanelOpen>()
            .add_systems(
                Update,
                (advisor_tips_ui, handle_advisor_jump)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

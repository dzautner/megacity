//! Systems for citizen selection, camera follow, and info panel rendering.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::camera::OrbitCamera;
use rendering::enhanced_select::SelectionKind;
use rendering::input::ActiveTool;
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenStateComp, Family, HomeLocation, Needs, Personality, Position,
    WorkLocation,
};
use simulation::config::CELL_SIZE;

use super::display::{education_label, gender_label, happiness_color, needs_bar, state_label};
use super::names::citizen_name;
use super::resources::{FollowCitizen, SelectedCitizen};

/// Detect citizen clicks when in Inspect mode.
///
/// When the user left-clicks in Inspect mode, find the nearest citizen within
/// a small radius of the cursor position. Citizens are matched by their
/// world-space Position component against the cursor grid position.
pub fn detect_citizen_selection(
    selection_kind: Res<SelectionKind>,
    tool: Res<ActiveTool>,
    mut selected: ResMut<SelectedCitizen>,
    mut follow: ResMut<FollowCitizen>,
) {
    if *tool != ActiveTool::Inspect {
        selected.0 = None;
        follow.0 = None;
        return;
    }

    // Defer to the enhanced selection system for priority-based selection.
    // Only show the citizen panel when the enhanced selector determined a
    // citizen was the highest-priority entity at the click position.
    if let SelectionKind::Citizen(entity) = *selection_kind {
        selected.0 = Some(entity);
    } else if selection_kind.is_changed() {
        // Another entity type was selected; clear citizen selection.
        selected.0 = None;
        follow.0 = None;
    }
}

/// Camera follow system: if FollowCitizen is active, move the camera focus
/// to track the citizen's position each frame.
pub fn camera_follow_citizen(
    follow: Res<FollowCitizen>,
    citizens: Query<&Position, With<Citizen>>,
    mut orbit: ResMut<OrbitCamera>,
) {
    let Some(entity) = follow.0 else {
        return;
    };

    let Ok(pos) = citizens.get(entity) else {
        return;
    };

    // Update the camera focus to the citizen's world position.
    // The OrbitCamera uses (x, 0, z) where x = world_x, z = world_y.
    orbit.focus.x = pos.x;
    orbit.focus.z = pos.y;
}

/// Render the Citizen Info Panel when a citizen is selected.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn citizen_info_panel_ui(
    mut contexts: EguiContexts,
    selected: Res<SelectedCitizen>,
    mut follow: ResMut<FollowCitizen>,
    citizens: Query<
        (
            Entity,
            &CitizenDetails,
            &CitizenStateComp,
            &HomeLocation,
            Option<&WorkLocation>,
            Option<&Needs>,
            Option<&Personality>,
            Option<&Family>,
        ),
        With<Citizen>,
    >,
    mut orbit: ResMut<OrbitCamera>,
) {
    let Some(entity) = selected.0 else {
        return;
    };

    let Ok((ent, details, state, home, work, needs, personality, family)) = citizens.get(entity)
    else {
        return;
    };

    let name = citizen_name(ent, details.gender);

    egui::Window::new("Citizen Info")
        .default_width(300.0)
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
        .show(contexts.ctx_mut(), |ui| {
            // Name heading
            ui.heading(&name);
            ui.separator();

            // Basic info grid
            egui::Grid::new("citizen_basic_info")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Age:");
                    ui.label(format!("{}", details.age));
                    ui.end_row();

                    ui.label("Gender:");
                    ui.label(gender_label(details.gender));
                    ui.end_row();

                    ui.label("Education:");
                    ui.label(education_label(details.education));
                    ui.end_row();

                    ui.label("Salary:");
                    ui.label(format!("${:.0}/mo", details.salary));
                    ui.end_row();

                    ui.label("Savings:");
                    ui.label(format!("${:.0}", details.savings));
                    ui.end_row();

                    ui.label("Health:");
                    let health_color = happiness_color(details.health);
                    ui.colored_label(health_color, format!("{:.0}%", details.health));
                    ui.end_row();
                });

            // Happiness
            ui.separator();
            ui.heading("Happiness");
            let color = happiness_color(details.happiness);
            ui.colored_label(color, format!("{:.0}%", details.happiness));

            // Current state
            ui.separator();
            ui.heading("Status");
            egui::Grid::new("citizen_status")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Current:");
                    ui.label(state_label(state.0));
                    ui.end_row();

                    ui.label("Life stage:");
                    let stage = details.life_stage();
                    ui.label(format!("{:?}", stage));
                    ui.end_row();
                });

            // Needs breakdown
            if let Some(n) = needs {
                ui.separator();
                ui.heading("Needs");
                needs_bar(ui, "Hunger", n.hunger);
                needs_bar(ui, "Energy", n.energy);
                needs_bar(ui, "Social", n.social);
                needs_bar(ui, "Fun", n.fun);
                needs_bar(ui, "Comfort", n.comfort);

                let (critical_name, critical_val) = n.most_critical();
                if critical_val < 30.0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 50, 50),
                        format!("Critical: {} ({:.0}%)", critical_name, critical_val),
                    );
                }
            }

            // Personality traits
            if let Some(p) = personality {
                ui.separator();
                ui.heading("Personality");
                egui::Grid::new("citizen_personality")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Ambition:");
                        ui.label(format!("{:.0}%", p.ambition * 100.0));
                        ui.end_row();

                        ui.label("Sociability:");
                        ui.label(format!("{:.0}%", p.sociability * 100.0));
                        ui.end_row();

                        ui.label("Materialism:");
                        ui.label(format!("{:.0}%", p.materialism * 100.0));
                        ui.end_row();

                        ui.label("Resilience:");
                        ui.label(format!("{:.0}%", p.resilience * 100.0));
                        ui.end_row();
                    });
            }

            // Locations
            ui.separator();
            ui.heading("Locations");

            // Home location (clickable)
            ui.horizontal(|ui| {
                ui.label("Home:");
                let home_label = format!("({}, {})", home.grid_x, home.grid_y);
                if ui.small_button(&home_label).clicked() {
                    let wx = home.grid_x as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                    let wy = home.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                    orbit.focus.x = wx;
                    orbit.focus.z = wy;
                }
            });

            // Work location (clickable)
            if let Some(w) = work {
                ui.horizontal(|ui| {
                    ui.label("Work:");
                    let work_label = format!("({}, {})", w.grid_x, w.grid_y);
                    if ui.small_button(&work_label).clicked() {
                        let wx = w.grid_x as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                        let wy = w.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                        orbit.focus.x = wx;
                        orbit.focus.z = wy;
                    }
                });
            } else {
                ui.label("Work: Unemployed");
            }

            // Family info
            if let Some(fam) = family {
                let has_family =
                    fam.partner.is_some() || !fam.children.is_empty() || fam.parent.is_some();
                if has_family {
                    ui.separator();
                    ui.heading("Family");
                    egui::Grid::new("citizen_family")
                        .num_columns(2)
                        .show(ui, |ui| {
                            if fam.partner.is_some() {
                                ui.label("Partner:");
                                ui.label("Yes");
                                ui.end_row();
                            }
                            if !fam.children.is_empty() {
                                ui.label("Children:");
                                ui.label(format!("{}", fam.children.len()));
                                ui.end_row();
                            }
                            if fam.parent.is_some() {
                                ui.label("Has parent:");
                                ui.label("Yes");
                                ui.end_row();
                            }
                        });
                }
            }

            // Follow button
            ui.separator();
            let is_following = follow.0 == Some(entity);
            let btn_text = if is_following {
                "Stop Following"
            } else {
                "Follow"
            };
            if ui.button(btn_text).clicked() {
                if is_following {
                    follow.0 = None;
                } else {
                    follow.0 = Some(entity);
                }
            }
        });
}

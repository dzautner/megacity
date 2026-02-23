//! Systems for the search/filter feature: keybind handling, result updates, and UI rendering.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::camera::OrbitCamera;
use simulation::abandonment::Abandoned;
use simulation::buildings::{Building, UnderConstruction};
use simulation::citizen::{Citizen, CitizenDetails, HomeLocation, Position, WorkLocation};
use simulation::config::CELL_SIZE;

use super::helpers::{citizen_name, education_label, zone_label};
use super::types::{BuildingResult, CitizenResult, SearchState, MAX_RESULTS};

/// Toggle search panel visibility with Ctrl+F.
pub fn search_keybind(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut state: ResMut<SearchState>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    // Don't intercept when egui already wants keyboard (except for our own search field)
    if contexts.ctx_mut().wants_keyboard_input() && !state.visible {
        return;
    }

    if bindings.toggle_search.just_pressed(&keyboard) {
        state.visible = !state.visible;
        if state.visible {
            state.request_focus = true;
            state.dirty = true;
        }
    }

    // Also close on Escape
    if state.visible && bindings.escape.just_pressed(&keyboard) {
        state.visible = false;
    }
}

/// Refresh search results when the query changes or the dirty flag is set.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_search_results(
    mut state: ResMut<SearchState>,
    buildings: Query<(
        Entity,
        &Building,
        Option<&Abandoned>,
        Option<&UnderConstruction>,
    )>,
    citizens: Query<
        (
            Entity,
            &CitizenDetails,
            Option<&Position>,
            Option<&HomeLocation>,
            Option<&WorkLocation>,
        ),
        With<Citizen>,
    >,
) {
    if !state.visible {
        return;
    }

    // Detect query change
    if state.query != state.prev_query {
        state.dirty = true;
        state.prev_query = state.query.clone();
    }

    if !state.dirty {
        return;
    }
    state.dirty = false;

    let query_lower = state.query.to_lowercase();
    let query_lower = query_lower.trim();

    // --- Building search ---
    state.building_results.clear();
    if state.search_buildings && !query_lower.is_empty() {
        for (entity, building, abandoned, under_construction) in &buildings {
            let zone_str = zone_label(building.zone_type);
            let status = if abandoned.is_some() {
                "Abandoned"
            } else if under_construction.is_some() {
                "Under Construction"
            } else {
                "Active"
            };

            let level_str = format!("L{}", building.level);
            let combined = format!(
                "{} {} {} {}",
                zone_str, level_str, status, building.zone_type as u8
            )
            .to_lowercase();

            if combined.contains(query_lower)
                || zone_str.to_lowercase().contains(query_lower)
                || status.to_lowercase().contains(query_lower)
                || level_str.to_lowercase().contains(query_lower)
            {
                state.building_results.push(BuildingResult {
                    entity,
                    zone_label: zone_str.to_string(),
                    level: building.level,
                    status,
                    grid_x: building.grid_x,
                    grid_y: building.grid_y,
                });
                if state.building_results.len() >= MAX_RESULTS {
                    break;
                }
            }
        }
    }

    // --- Citizen search ---
    state.citizen_results.clear();
    if state.search_citizens && !query_lower.is_empty() {
        for (entity, details, pos, home, _work) in &citizens {
            let name = citizen_name(entity, details.gender);
            let edu = education_label(details.education);
            let age_str = format!("{}", details.age);

            let combined = format!("{} {} {}", name, age_str, edu).to_lowercase();

            if combined.contains(query_lower) {
                // Use position if available, otherwise home location
                let (gx, gy) = if let Some(p) = pos {
                    (p.x, p.y)
                } else if let Some(h) = home {
                    (
                        h.grid_x as f32 * CELL_SIZE + CELL_SIZE * 0.5,
                        h.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5,
                    )
                } else {
                    (0.0, 0.0)
                };

                state.citizen_results.push(CitizenResult {
                    entity,
                    name,
                    age: details.age,
                    education: edu,
                    grid_x: gx,
                    grid_y: gy,
                });
                if state.citizen_results.len() >= MAX_RESULTS {
                    break;
                }
            }
        }
    }
}

/// Render the search panel UI.
pub fn search_panel_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<SearchState>,
    mut orbit: ResMut<OrbitCamera>,
) {
    if !state.visible {
        return;
    }

    let mut jump_target: Option<(f32, f32)> = None;

    egui::Window::new("\u{1f50d} Search")
        .default_width(350.0)
        .default_height(400.0)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 40.0))
        .resizable(true)
        .collapsible(true)
        .show(contexts.ctx_mut(), |ui| {
            // Search input
            ui.horizontal(|ui| {
                ui.label("Search:");
                let text_edit = ui.text_edit_singleline(&mut state.query);
                if state.request_focus {
                    text_edit.request_focus();
                    state.request_focus = false;
                }
            });

            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut state.search_buildings, "Buildings")
                    .changed()
                {
                    state.dirty = true;
                }
                if ui
                    .checkbox(&mut state.search_citizens, "Citizens")
                    .changed()
                {
                    state.dirty = true;
                }
            });

            if state.query.trim().is_empty() {
                ui.separator();
                ui.label("Type to search buildings and citizens.");
                ui.label("Examples: \"residential\", \"abandoned\", \"L3\", \"James\"");
                return;
            }

            ui.separator();

            let total = state.building_results.len() + state.citizen_results.len();
            ui.label(format!(
                "{} result{} ({} buildings, {} citizens)",
                total,
                if total == 1 { "" } else { "s" },
                state.building_results.len(),
                state.citizen_results.len(),
            ));

            egui::ScrollArea::vertical()
                .max_height(500.0)
                .show(ui, |ui| {
                    // --- Building results ---
                    if !state.building_results.is_empty() {
                        ui.add_space(4.0);
                        ui.strong("Buildings");
                        ui.separator();

                        for result in &state.building_results {
                            let label = format!(
                                "{} L{} - {} ({},{})",
                                result.zone_label,
                                result.level,
                                result.status,
                                result.grid_x,
                                result.grid_y,
                            );
                            if ui
                                .selectable_label(false, &label)
                                .on_hover_text("Click to jump to location")
                                .clicked()
                            {
                                let wx = result.grid_x as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                                let wy = result.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                                jump_target = Some((wx, wy));
                            }
                        }
                    }

                    // --- Citizen results ---
                    if !state.citizen_results.is_empty() {
                        ui.add_space(4.0);
                        ui.strong("Citizens");
                        ui.separator();

                        for result in &state.citizen_results {
                            let label = format!(
                                "{} (Age {}, {})",
                                result.name, result.age, result.education,
                            );
                            if ui
                                .selectable_label(false, &label)
                                .on_hover_text("Click to jump to location")
                                .clicked()
                            {
                                jump_target = Some((result.grid_x, result.grid_y));
                            }
                        }
                    }

                    if state.building_results.is_empty() && state.citizen_results.is_empty() {
                        ui.add_space(8.0);
                        ui.label("No results found.");
                    }
                });
        });

    // Apply camera jump outside the egui closure (to avoid borrow issues)
    if let Some((wx, wy)) = jump_target {
        orbit.focus.x = wx;
        orbit.focus.z = wy;
        // Zoom in closer to show the target
        orbit.distance = orbit.distance.min(400.0);
    }
}

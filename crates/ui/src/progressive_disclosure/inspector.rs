//! Main Building Inspector UI system and plugin registration.
//!
//! The system dispatches to the appropriate tab renderer based on the
//! active [`SelectedBuildingTab`]. Service and utility buildings use
//! simpler flat layouts rendered inline here.

use bevy::prelude::*;
use simulation::app_state::AppState;
use bevy_egui::{egui, EguiContexts};

use rendering::input::SelectedBuilding;
use simulation::buildings::Building;
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenStateComp, Family, HomeLocation, Needs, Personality,
    WorkLocation,
};
use simulation::config::CELL_SIZE;
use simulation::budget::ExtendedBudget;
use simulation::grid::WorldGrid;
use simulation::land_value::LandValueGrid;
use simulation::noise::NoisePollutionGrid;
use simulation::pollution::PollutionGrid;
use simulation::services::ServiceBuilding;
use simulation::trees::TreeGrid;
use simulation::utilities::UtilitySource;

use simulation::config::GRID_WIDTH;

use crate::citizen_info::{FollowCitizen, SelectedCitizen};

use super::citizen_tabs::{render_residents_tab, render_workers_tab};
use super::economy_tab::render_economy_tab;
use super::helpers::{tab_bar, zone_type_label};
use super::simple_tabs::{render_environment_tab, render_overview_tab, render_services_tab};
use super::types::{BuildingTab, SelectedBuildingTab};

// =============================================================================
// Tabbed Building Inspector UI system
// =============================================================================

/// Renders the Building Inspector with a tabbed layout.
///
/// The tab bar at the top allows switching between Overview, Services, Economy,
/// Residents/Workers, and Environment. This system replaces the flat
/// `building_inspection_ui` when the plugin is active.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn progressive_building_inspection_ui(
    mut contexts: EguiContexts,
    selected: Res<SelectedBuilding>,
    buildings: Query<&Building>,
    service_buildings: Query<&ServiceBuilding>,
    utility_sources: Query<&UtilitySource>,
    citizens: Query<
        (
            Entity,
            &CitizenDetails,
            &HomeLocation,
            Option<&WorkLocation>,
            &CitizenStateComp,
            Option<&Needs>,
            Option<&Personality>,
            Option<&Family>,
        ),
        With<Citizen>,
    >,
    grid: Res<WorldGrid>,
    pollution: Res<PollutionGrid>,
    noise: Res<NoisePollutionGrid>,
    land_value: Res<LandValueGrid>,
    ext_budget: Res<ExtendedBudget>,
    tree_grid: Res<TreeGrid>,
    mut tab_state: ResMut<SelectedBuildingTab>,
    mut selected_citizen: ResMut<SelectedCitizen>,
    mut follow_citizen: ResMut<FollowCitizen>,
) {
    let Some(entity) = selected.0 else {
        return;
    };

    // === Zone building inspection with tabs ===
    if let Ok(building) = buildings.get(entity) {
        let cell = grid.get(building.grid_x, building.grid_y);
        let idx = building.grid_y * GRID_WIDTH + building.grid_x;
        let poll_level = pollution.levels.get(idx).copied().unwrap_or(0);
        let noise_level = noise.levels.get(idx).copied().unwrap_or(0);
        let lv = land_value.values.get(idx).copied().unwrap_or(0);
        let occupancy_pct = if building.capacity > 0 {
            (building.occupants as f32 / building.capacity as f32 * 100.0).min(100.0)
        } else {
            0.0
        };

        // Compute average happiness for this building's occupants
        let avg_happiness = if building.zone_type.is_residential() {
            let residents: Vec<f32> = citizens
                .iter()
                .filter(|(_, _, home, _, _, _, _, _)| home.building == entity)
                .map(|(_, details, _, _, _, _, _, _)| details.happiness)
                .collect();
            if residents.is_empty() {
                0.0
            } else {
                residents.iter().sum::<f32>() / residents.len() as f32
            }
        } else {
            let workers: Vec<f32> = citizens
                .iter()
                .filter(|(_, _, _, work, _, _, _, _)| {
                    work.map(|w| w.building == entity).unwrap_or(false)
                })
                .map(|(_, details, _, _, _, _, _, _)| details.happiness)
                .collect();
            if workers.is_empty() {
                0.0
            } else {
                workers.iter().sum::<f32>() / workers.len() as f32
            }
        };

        egui::Window::new("Building Inspector")
            .default_width(340.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
            .show(contexts.ctx_mut(), |ui| {
                // Heading: always visible regardless of tab
                ui.heading(zone_type_label(building.zone_type));
                ui.separator();

                // Tab bar
                tab_bar(ui, &mut tab_state.0);

                match tab_state.0 {
                    BuildingTab::Overview => {
                        render_overview_tab(
                            ui,
                            building,
                            occupancy_pct,
                            avg_happiness,
                            cell.has_power,
                            cell.has_water,
                        );
                    }
                    BuildingTab::Services => {
                        render_services_tab(
                            ui,
                            building,
                            cell.has_power,
                            cell.has_water,
                            &service_buildings,
                        );
                    }
                    BuildingTab::Economy => {
                        render_economy_tab(ui, entity, building, lv, &citizens, &ext_budget);
                    }
                    BuildingTab::Residents => {
                        if building.zone_type.is_residential() {
                            render_residents_tab(
                                ui,
                                entity,
                                &citizens,
                                &mut selected_citizen,
                                &mut follow_citizen,
                            );
                        } else {
                            render_workers_tab(
                                ui,
                                entity,
                                &citizens,
                                &mut selected_citizen,
                                &mut follow_citizen,
                            );
                        }
                    }
                    BuildingTab::Environment => {
                        render_environment_tab(
                            ui,
                            building,
                            poll_level,
                            noise_level,
                            lv,
                            &tree_grid,
                        );
                    }
                }
            });
        return;
    }

    // === Service building inspection (simpler, no tabs needed) ===
    if let Ok(service) = service_buildings.get(entity) {
        let cell = grid.get(service.grid_x, service.grid_y);
        let idx = service.grid_y * GRID_WIDTH + service.grid_x;
        let lv = land_value.values.get(idx).copied().unwrap_or(0);
        let monthly_cost =
            simulation::services::ServiceBuilding::monthly_maintenance(service.service_type);

        egui::Window::new("Building Inspector")
            .default_width(280.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.heading(service.service_type.name());
                ui.separator();

                egui::Grid::new("pd_service_overview")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Location:");
                        ui.label(format!("({}, {})", service.grid_x, service.grid_y));
                        ui.end_row();
                        ui.label("Coverage:");
                        ui.label(format!(
                            "{:.0} px ({:.0} cells)",
                            service.radius,
                            service.radius / CELL_SIZE
                        ));
                        ui.end_row();
                        ui.label("Monthly cost:");
                        ui.label(format!("${:.0}", monthly_cost));
                        ui.end_row();
                        ui.label("Land value:");
                        ui.label(format!("{}/255", lv));
                        ui.end_row();
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    let power_color = if cell.has_power {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else {
                        egui::Color32::from_rgb(200, 50, 50)
                    };
                    let water_color = if cell.has_water {
                        egui::Color32::from_rgb(50, 130, 220)
                    } else {
                        egui::Color32::from_rgb(200, 50, 50)
                    };
                    ui.colored_label(
                        power_color,
                        if cell.has_power {
                            "Power: ON"
                        } else {
                            "Power: OFF"
                        },
                    );
                    ui.colored_label(
                        water_color,
                        if cell.has_water {
                            "Water: ON"
                        } else {
                            "Water: OFF"
                        },
                    );
                });
            });
        return;
    }

    // === Utility building inspection ===
    if let Ok(utility) = utility_sources.get(entity) {
        let cell = grid.get(utility.grid_x, utility.grid_y);

        egui::Window::new("Building Inspector")
            .default_width(280.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.heading(utility.utility_type.name());
                ui.separator();

                egui::Grid::new("pd_utility_overview")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Type:");
                        ui.label(if utility.utility_type.is_power() {
                            "Power Generation"
                        } else {
                            "Water Supply"
                        });
                        ui.end_row();
                        ui.label("Location:");
                        ui.label(format!("({}, {})", utility.grid_x, utility.grid_y));
                        ui.end_row();
                        ui.label("Range:");
                        ui.label(format!("{} cells", utility.range));
                        ui.end_row();
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    let power_color = if cell.has_power {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else {
                        egui::Color32::from_rgb(200, 50, 50)
                    };
                    let water_color = if cell.has_water {
                        egui::Color32::from_rgb(50, 130, 220)
                    } else {
                        egui::Color32::from_rgb(200, 50, 50)
                    };
                    ui.colored_label(
                        power_color,
                        if cell.has_power {
                            "Power: ON"
                        } else {
                            "Power: OFF"
                        },
                    );
                    ui.colored_label(
                        water_color,
                        if cell.has_water {
                            "Water: ON"
                        } else {
                            "Water: OFF"
                        },
                    );
                });
            });
    }
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that adds the tabbed Building Inspector panel.
pub struct ProgressiveDisclosurePlugin;

impl Plugin for ProgressiveDisclosurePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedBuildingTab>()
            .add_systems(
                Update,
                progressive_building_inspection_ui.run_if(in_state(AppState::Playing)),
            );
    }
}

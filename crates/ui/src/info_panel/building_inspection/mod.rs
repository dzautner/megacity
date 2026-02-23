mod helpers;
mod residential;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::buildings::Building;
use simulation::citizen::Citizen;
use simulation::config::CELL_SIZE;
use simulation::config::GRID_WIDTH;
use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::land_value::LandValueGrid;
use simulation::pollution::PollutionGrid;
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilitySource;

use rendering::input::SelectedBuilding;

use helpers::{power_water_labels, zone_type_name};
use residential::{render_residential_section, render_workers_section, CitizenQuery};

#[allow(clippy::too_many_arguments)]
pub fn building_inspection_ui(
    mut contexts: EguiContexts,
    selected: Res<SelectedBuilding>,
    buildings: Query<&Building>,
    service_buildings: Query<&ServiceBuilding>,
    utility_sources: Query<&UtilitySource>,
    citizens: Query<CitizenQuery, With<Citizen>>,
    grid: Res<WorldGrid>,
    pollution: Res<PollutionGrid>,
    land_value: Res<LandValueGrid>,
    budget: Res<CityBudget>,
) {
    let Some(entity) = selected.0 else {
        return;
    };

    // Zone building inspection
    if let Ok(building) = buildings.get(entity) {
        render_zone_building(
            &mut contexts,
            entity,
            building,
            &citizens,
            &grid,
            &pollution,
            &land_value,
            &budget,
        );
        return;
    }

    // Service building inspection
    if let Ok(service) = service_buildings.get(entity) {
        render_service_building(&mut contexts, service, &grid, &land_value);
        return;
    }

    // Utility building inspection
    if let Ok(utility) = utility_sources.get(entity) {
        render_utility_building(&mut contexts, utility, &grid);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_zone_building(
    contexts: &mut EguiContexts,
    entity: Entity,
    building: &Building,
    citizens: &Query<CitizenQuery, With<Citizen>>,
    grid: &WorldGrid,
    pollution: &PollutionGrid,
    land_value: &LandValueGrid,
    budget: &CityBudget,
) {
    let cell = grid.get(building.grid_x, building.grid_y);
    let idx = building.grid_y * GRID_WIDTH + building.grid_x;
    let poll_level = pollution.levels.get(idx).copied().unwrap_or(0);
    let lv = land_value.values.get(idx).copied().unwrap_or(0);
    let occupancy_pct = if building.capacity > 0 {
        (building.occupants as f32 / building.capacity as f32 * 100.0).min(100.0)
    } else {
        0.0
    };

    egui::Window::new("Building Inspector")
        .default_width(320.0)
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading(zone_type_name(building.zone_type));
            ui.separator();

            render_building_overview(ui, building, occupancy_pct, lv, poll_level);

            // Power/Water status
            ui.separator();
            ui.horizontal(|ui| {
                power_water_labels(ui, cell.has_power, cell.has_water);
            });

            if building.zone_type.is_residential() {
                render_residential_section(ui, entity, citizens, budget);
            } else {
                render_workers_section(ui, entity, citizens);
            }
        });
}

fn render_building_overview(
    ui: &mut egui::Ui,
    building: &Building,
    occupancy_pct: f32,
    lv: u8,
    poll_level: u8,
) {
    egui::Grid::new("building_overview")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Level:");
            ui.label(format!(
                "{} / {}",
                building.level,
                building.zone_type.max_level()
            ));
            ui.end_row();

            ui.label("Occupancy:");
            let occ_color = if occupancy_pct >= 90.0 {
                egui::Color32::from_rgb(220, 50, 50)
            } else if occupancy_pct >= 70.0 {
                egui::Color32::from_rgb(220, 180, 50)
            } else {
                egui::Color32::from_rgb(50, 200, 50)
            };
            ui.colored_label(
                occ_color,
                format!(
                    "{} / {} ({:.0}%)",
                    building.occupants, building.capacity, occupancy_pct
                ),
            );
            ui.end_row();

            ui.label("Location:");
            ui.label(format!("({}, {})", building.grid_x, building.grid_y));
            ui.end_row();

            ui.label("Land Value:");
            ui.label(format!("{}/255", lv));
            ui.end_row();

            ui.label("Pollution:");
            let poll_color = if poll_level > 50 {
                egui::Color32::from_rgb(200, 50, 50)
            } else if poll_level > 20 {
                egui::Color32::from_rgb(200, 150, 50)
            } else {
                egui::Color32::from_rgb(50, 200, 50)
            };
            ui.colored_label(poll_color, format!("{}/255", poll_level));
            ui.end_row();
        });
}

fn render_service_building(
    contexts: &mut EguiContexts,
    service: &ServiceBuilding,
    grid: &WorldGrid,
    land_value: &LandValueGrid,
) {
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

            egui::Grid::new("service_overview")
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
                power_water_labels(ui, cell.has_power, cell.has_water);
            });
        });
}

fn render_utility_building(contexts: &mut EguiContexts, utility: &UtilitySource, grid: &WorldGrid) {
    let cell = grid.get(utility.grid_x, utility.grid_y);

    egui::Window::new("Building Inspector")
        .default_width(280.0)
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading(utility.utility_type.name());
            ui.separator();

            egui::Grid::new("utility_overview")
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
                power_water_labels(ui, cell.has_power, cell.has_water);
            });
        });
}

//! Simple tab renderers: Overview, Services, and Environment.
//!
//! These tabs have straightforward layouts with no clickable citizen rows,
//! so they are grouped together.

use bevy::prelude::*;
use bevy_egui::egui;

use simulation::buildings::Building;
use simulation::config::CELL_SIZE;
use simulation::services::ServiceBuilding;
use simulation::trees::TreeGrid;

use super::helpers::{
    count_nearby_trees, green_space_label, happiness_color, noise_color, occupancy_color,
    pollution_color, zone_type_label,
};

// =============================================================================
// Tab content: Overview
// =============================================================================

pub(crate) fn render_overview_tab(
    ui: &mut egui::Ui,
    building: &Building,
    occupancy_pct: f32,
    avg_happiness: f32,
    has_power: bool,
    has_water: bool,
) {
    egui::Grid::new("tab_overview_grid")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("Type:");
            ui.label(zone_type_label(building.zone_type));
            ui.end_row();

            ui.label("Level:");
            ui.label(format!(
                "{} / {}",
                building.level,
                building.zone_type.max_level()
            ));
            ui.end_row();

            ui.label("Occupancy:");
            ui.colored_label(
                occupancy_color(occupancy_pct),
                format!(
                    "{} / {} ({:.0}%)",
                    building.occupants, building.capacity, occupancy_pct
                ),
            );
            ui.end_row();

            ui.label("Happiness:");
            ui.colored_label(
                happiness_color(avg_happiness),
                format!("{:.0}%", avg_happiness),
            );
            ui.end_row();

            ui.label("Location:");
            ui.label(format!("({}, {})", building.grid_x, building.grid_y));
            ui.end_row();
        });

    // Quick power/water status
    ui.separator();
    ui.horizontal(|ui| {
        let power_color = if has_power {
            egui::Color32::from_rgb(50, 200, 50)
        } else {
            egui::Color32::from_rgb(200, 50, 50)
        };
        let water_color = if has_water {
            egui::Color32::from_rgb(50, 130, 220)
        } else {
            egui::Color32::from_rgb(200, 50, 50)
        };
        ui.colored_label(
            power_color,
            if has_power { "Power: ON" } else { "Power: OFF" },
        );
        ui.colored_label(
            water_color,
            if has_water { "Water: ON" } else { "Water: OFF" },
        );
    });
}

// =============================================================================
// Tab content: Services
// =============================================================================

pub(crate) fn render_services_tab(
    ui: &mut egui::Ui,
    building: &Building,
    has_power: bool,
    has_water: bool,
    service_buildings: &Query<&ServiceBuilding>,
) {
    // Utility connections
    ui.label("Utility Connections:");
    ui.horizontal(|ui| {
        let power_color = if has_power {
            egui::Color32::from_rgb(50, 200, 50)
        } else {
            egui::Color32::from_rgb(200, 50, 50)
        };
        let water_color = if has_water {
            egui::Color32::from_rgb(50, 130, 220)
        } else {
            egui::Color32::from_rgb(200, 50, 50)
        };
        ui.colored_label(
            power_color,
            if has_power { "Power: ON" } else { "Power: OFF" },
        );
        ui.colored_label(
            water_color,
            if has_water { "Water: ON" } else { "Water: OFF" },
        );
    });

    ui.separator();
    ui.label("Nearby Service Coverage:");

    // Gather services that cover this building
    let mut covered: Vec<(&str, f32, f32)> = Vec::new(); // (name, distance, radius)
    for service in service_buildings.iter() {
        let dx = (service.grid_x as f32 - building.grid_x as f32) * CELL_SIZE;
        let dy = (service.grid_y as f32 - building.grid_y as f32) * CELL_SIZE;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist <= service.radius {
            let name = service.service_type.name();
            // Avoid duplicates of the same service type
            if !covered.iter().any(|(n, _, _)| *n == name) {
                covered.push((name, dist, service.radius));
            }
        }
    }

    if covered.is_empty() {
        ui.colored_label(
            egui::Color32::from_rgb(160, 160, 160),
            "No services in range",
        );
    } else {
        covered.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        egui::Grid::new("tab_services_grid")
            .num_columns(3)
            .spacing([12.0, 2.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Service");
                ui.strong("Distance");
                ui.strong("Quality");
                ui.end_row();

                for (name, dist, radius) in &covered {
                    let cells_away = (dist / CELL_SIZE).round() as u32;
                    // Quality based on how close to the center of coverage
                    let quality_pct = ((1.0 - dist / radius) * 100.0).clamp(0.0, 100.0);
                    let quality_color = if quality_pct >= 60.0 {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else if quality_pct >= 30.0 {
                        egui::Color32::from_rgb(220, 180, 50)
                    } else {
                        egui::Color32::from_rgb(220, 120, 50)
                    };

                    ui.colored_label(egui::Color32::from_rgb(80, 200, 140), *name);
                    ui.colored_label(
                        egui::Color32::from_rgb(160, 160, 160),
                        format!("{} cells", cells_away),
                    );
                    ui.colored_label(quality_color, format!("{:.0}%", quality_pct));
                    ui.end_row();
                }
            });
    }
}

// =============================================================================
// Tab content: Environment
// =============================================================================

pub(crate) fn render_environment_tab(
    ui: &mut egui::Ui,
    building: &Building,
    poll_level: u8,
    noise_level: u8,
    land_val: u8,
    tree_grid: &TreeGrid,
) {
    let nearby_trees = count_nearby_trees(tree_grid, building.grid_x, building.grid_y, 5);
    let (gs_label, gs_color) = green_space_label(nearby_trees);

    egui::Grid::new("tab_env_grid")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("Pollution:");
            ui.colored_label(pollution_color(poll_level), format!("{}/255", poll_level));
            ui.end_row();

            ui.label("Noise:");
            ui.colored_label(noise_color(noise_level), format!("{}/100", noise_level));
            ui.end_row();

            ui.label("Green Space:");
            ui.colored_label(
                gs_color,
                format!("{} ({} trees nearby)", gs_label, nearby_trees),
            );
            ui.end_row();

            ui.label("Land Value:");
            let lv_color = if land_val >= 180 {
                egui::Color32::from_rgb(50, 200, 50)
            } else if land_val >= 80 {
                egui::Color32::from_rgb(220, 180, 50)
            } else {
                egui::Color32::from_rgb(180, 100, 60)
            };
            ui.colored_label(lv_color, format!("{}/255", land_val));
            ui.end_row();

            ui.label("Location:");
            ui.label(format!("({}, {})", building.grid_x, building.grid_y));
            ui.end_row();
        });

    // Environmental quality summary
    ui.separator();
    ui.label("Environmental Quality:");

    let env_score = {
        // Invert pollution and noise (lower is better), combine with green space
        let poll_score = (255.0 - poll_level as f32) / 255.0 * 40.0;
        let noise_score = (100.0 - noise_level as f32) / 100.0 * 30.0;
        let green_score = (nearby_trees as f32).min(20.0) / 20.0 * 30.0;
        (poll_score + noise_score + green_score).clamp(0.0, 100.0)
    };

    let env_color = if env_score >= 70.0 {
        egui::Color32::from_rgb(50, 200, 50)
    } else if env_score >= 40.0 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(220, 50, 50)
    };

    let env_label = if env_score >= 80.0 {
        "Excellent"
    } else if env_score >= 60.0 {
        "Good"
    } else if env_score >= 40.0 {
        "Fair"
    } else if env_score >= 20.0 {
        "Poor"
    } else {
        "Very Poor"
    };

    ui.colored_label(env_color, format!("{} ({:.0}/100)", env_label, env_score));
}

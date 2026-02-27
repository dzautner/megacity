//! Empty Cell Info Panel (UX-065).
//!
//! When the player clicks on an empty grass cell (no building), this panel
//! displays contextual information about that cell:
//! - Elevation
//! - Current land value
//! - Zone type (or "Unzoned")
//! - Pollution level
//! - Nearby service coverage
//! - Noise level

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::enhanced_select::SelectionKind;
use rendering::input::{ActiveTool, SelectedBuilding};
use simulation::config::{CELL_SIZE, GRID_WIDTH};
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::land_value::LandValueGrid;
use simulation::noise::NoisePollutionGrid;
use simulation::pollution::PollutionGrid;
use simulation::services::{ServiceBuilding, ServiceType};
use simulation::app_state::AppState;
use simulation::SaveLoadState;

/// Tracks which empty cell is currently selected for the info panel.
/// Set to `None` when no empty cell is selected or when a building is selected.
#[derive(Resource, Default)]
pub struct SelectedCell(pub Option<(usize, usize)>);

/// Update the selected cell when the player clicks on an empty cell with the
/// Inspect tool. Clears the selection when a building is selected instead.
pub fn update_selected_cell(
    selection_kind: Res<SelectionKind>,
    tool: Res<ActiveTool>,
    mut selected_cell: ResMut<SelectedCell>,
) {
    if *tool != ActiveTool::Inspect {
        selected_cell.0 = None;
        return;
    }

    // Defer to the enhanced selection system for priority-based selection.
    // Only show the cell panel when the enhanced selector determined an
    // empty cell was the lowest-priority selection.
    if let SelectionKind::Cell(gx, gy) = *selection_kind {
        selected_cell.0 = Some((gx, gy));
    } else if selection_kind.is_changed() {
        selected_cell.0 = None;
    }
}

/// Zone type display name for the cell info panel.
fn zone_label(zone: ZoneType) -> &'static str {
    match zone {
        ZoneType::None => "Unzoned",
        ZoneType::ResidentialLow => "Low-Density Residential",
        ZoneType::ResidentialMedium => "Medium-Density Residential",
        ZoneType::ResidentialHigh => "High-Density Residential",
        ZoneType::CommercialLow => "Low-Density Commercial",
        ZoneType::CommercialHigh => "High-Density Commercial",
        ZoneType::Industrial => "Industrial",
        ZoneType::Office => "Office",
        ZoneType::MixedUse => "Mixed-Use",
    }
}

/// Describes a pollution/noise level using a human-readable label and color.
fn level_label(value: u8) -> (&'static str, egui::Color32) {
    if value == 0 {
        ("None", egui::Color32::from_rgb(50, 200, 50))
    } else if value <= 15 {
        ("Low", egui::Color32::from_rgb(80, 200, 80))
    } else if value <= 40 {
        ("Moderate", egui::Color32::from_rgb(220, 180, 50))
    } else if value <= 70 {
        ("High", egui::Color32::from_rgb(220, 120, 50))
    } else {
        ("Very High", egui::Color32::from_rgb(220, 50, 50))
    }
}

/// Key service categories to show in the nearby services section.
fn is_key_service(st: ServiceType) -> bool {
    matches!(
        st,
        ServiceType::FireStation
            | ServiceType::FireHouse
            | ServiceType::FireHQ
            | ServiceType::PoliceStation
            | ServiceType::PoliceKiosk
            | ServiceType::PoliceHQ
            | ServiceType::Hospital
            | ServiceType::MedicalClinic
            | ServiceType::MedicalCenter
            | ServiceType::ElementarySchool
            | ServiceType::HighSchool
            | ServiceType::University
            | ServiceType::Library
            | ServiceType::Kindergarten
            | ServiceType::SmallPark
            | ServiceType::LargePark
            | ServiceType::Playground
            | ServiceType::BusDepot
            | ServiceType::TrainStation
            | ServiceType::SubwayStation
    )
}

/// Displays the empty cell info panel when a cell without a building is selected.
///
/// Shows elevation, land value, zone type, pollution, noise, and nearby services.
#[allow(clippy::too_many_arguments)]
pub fn cell_info_panel_ui(
    mut contexts: EguiContexts,
    selected_cell: Res<SelectedCell>,
    selected_building: Res<SelectedBuilding>,
    grid: Res<WorldGrid>,
    pollution: Res<PollutionGrid>,
    land_value: Res<LandValueGrid>,
    noise: Res<NoisePollutionGrid>,
    services: Query<&ServiceBuilding>,
) {
    // Don't show if a building is selected (the building inspector takes precedence)
    if selected_building.0.is_some() {
        return;
    }

    let Some((gx, gy)) = selected_cell.0 else {
        return;
    };

    if !grid.in_bounds(gx, gy) {
        return;
    }

    let cell = grid.get(gx, gy);
    let idx = gy * GRID_WIDTH + gx;

    let elevation = cell.elevation;
    let lv = land_value.values.get(idx).copied().unwrap_or(0);
    let poll_level = pollution.levels.get(idx).copied().unwrap_or(0);
    let noise_level = noise.levels.get(idx).copied().unwrap_or(0);

    // Calculate the world-space position of this cell for service distance checks
    let (world_x, world_y) = WorldGrid::grid_to_world(gx, gy);

    // Gather nearby services within their coverage radius
    let mut covered_services: Vec<(ServiceType, f32)> = Vec::new();
    for service in &services {
        if !is_key_service(service.service_type) {
            continue;
        }
        let (sx, sy) = WorldGrid::grid_to_world(service.grid_x, service.grid_y);
        let dist = ((world_x - sx).powi(2) + (world_y - sy).powi(2)).sqrt();
        let radius = ServiceBuilding::coverage_radius(service.service_type);
        if dist <= radius {
            covered_services.push((service.service_type, dist));
        }
    }
    covered_services.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Deduplicate by service type, keeping the closest instance
    let mut seen_types = Vec::new();
    covered_services.retain(|(st, _)| {
        if seen_types.contains(st) {
            false
        } else {
            seen_types.push(*st);
            true
        }
    });

    let cell_type_label = match cell.cell_type {
        CellType::Grass => "Grass",
        CellType::Water => "Water",
        CellType::Road => "Road",
    };

    egui::Window::new("Cell Info")
        .default_width(260.0)
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading(format!("Cell ({}, {})", gx, gy));
            ui.label(format!("Terrain: {}", cell_type_label));
            ui.separator();

            egui::Grid::new("cell_info_grid")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    // Elevation
                    ui.label("Elevation:");
                    ui.label(format!("{:.1} m", elevation));
                    ui.end_row();

                    // Land value
                    ui.label("Land Value:");
                    let lv_color = if lv >= 180 {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else if lv >= 80 {
                        egui::Color32::from_rgb(220, 180, 50)
                    } else {
                        egui::Color32::from_rgb(180, 100, 60)
                    };
                    ui.colored_label(lv_color, format!("{}/255", lv));
                    ui.end_row();

                    // Zone type
                    ui.label("Zone:");
                    let zone_text = zone_label(cell.zone);
                    let zone_color = match cell.zone {
                        ZoneType::None => egui::Color32::from_rgb(160, 160, 160),
                        ZoneType::ResidentialLow
                        | ZoneType::ResidentialMedium
                        | ZoneType::ResidentialHigh => egui::Color32::from_rgb(50, 180, 50),
                        ZoneType::CommercialLow | ZoneType::CommercialHigh => {
                            egui::Color32::from_rgb(50, 80, 200)
                        }
                        ZoneType::Industrial => egui::Color32::from_rgb(200, 180, 30),
                        ZoneType::Office => egui::Color32::from_rgb(160, 130, 220),
                        ZoneType::MixedUse => egui::Color32::from_rgb(180, 100, 180),
                    };
                    ui.colored_label(zone_color, zone_text);
                    ui.end_row();

                    // Pollution
                    ui.label("Pollution:");
                    let (poll_text, poll_color) = level_label(poll_level);
                    ui.colored_label(poll_color, format!("{} ({}/255)", poll_text, poll_level));
                    ui.end_row();

                    // Noise
                    ui.label("Noise:");
                    let (noise_text, noise_color) = level_label(noise_level);
                    ui.colored_label(noise_color, format!("{} ({}/100)", noise_text, noise_level));
                    ui.end_row();
                });

            // Nearby service coverage
            ui.separator();
            ui.label("Service Coverage:");

            if covered_services.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(160, 160, 160),
                    "No services in range",
                );
            } else {
                egui::Grid::new("cell_services_grid")
                    .num_columns(2)
                    .spacing([16.0, 2.0])
                    .show(ui, |ui| {
                        for (st, dist) in &covered_services {
                            let cells_away = (dist / CELL_SIZE).round() as u32;
                            ui.colored_label(egui::Color32::from_rgb(80, 200, 140), st.name());
                            ui.colored_label(
                                egui::Color32::from_rgb(160, 160, 160),
                                format!("{} cells", cells_away),
                            );
                            ui.end_row();
                        }
                    });
            }
        });
}

/// Plugin that provides the empty cell info panel feature.
pub struct CellInfoPanelPlugin;

impl Plugin for CellInfoPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedCell>()
            .add_systems(
                Update,
                (update_selected_cell, cell_info_panel_ui)
                    .chain()
                    .run_if(in_state(SaveLoadState::Idle))
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_label_unzoned() {
        assert_eq!(zone_label(ZoneType::None), "Unzoned");
    }

    #[test]
    fn test_zone_label_all_variants() {
        assert_eq!(
            zone_label(ZoneType::ResidentialLow),
            "Low-Density Residential"
        );
        assert_eq!(
            zone_label(ZoneType::ResidentialMedium),
            "Medium-Density Residential"
        );
        assert_eq!(
            zone_label(ZoneType::ResidentialHigh),
            "High-Density Residential"
        );
        assert_eq!(
            zone_label(ZoneType::CommercialLow),
            "Low-Density Commercial"
        );
        assert_eq!(
            zone_label(ZoneType::CommercialHigh),
            "High-Density Commercial"
        );
        assert_eq!(zone_label(ZoneType::Industrial), "Industrial");
        assert_eq!(zone_label(ZoneType::Office), "Office");
        assert_eq!(zone_label(ZoneType::MixedUse), "Mixed-Use");
    }

    #[test]
    fn test_level_label_none() {
        let (text, _color) = level_label(0);
        assert_eq!(text, "None");
    }

    #[test]
    fn test_level_label_low() {
        let (text, _color) = level_label(10);
        assert_eq!(text, "Low");
    }

    #[test]
    fn test_level_label_moderate() {
        let (text, _color) = level_label(30);
        assert_eq!(text, "Moderate");
    }

    #[test]
    fn test_level_label_high() {
        let (text, _color) = level_label(50);
        assert_eq!(text, "High");
    }

    #[test]
    fn test_level_label_very_high() {
        let (text, _color) = level_label(80);
        assert_eq!(text, "Very High");
    }

    #[test]
    fn test_level_label_max() {
        let (text, _color) = level_label(255);
        assert_eq!(text, "Very High");
    }

    #[test]
    fn test_level_label_boundaries() {
        // Boundary at 15
        let (text, _) = level_label(15);
        assert_eq!(text, "Low");
        let (text, _) = level_label(16);
        assert_eq!(text, "Moderate");

        // Boundary at 40
        let (text, _) = level_label(40);
        assert_eq!(text, "Moderate");
        let (text, _) = level_label(41);
        assert_eq!(text, "High");

        // Boundary at 70
        let (text, _) = level_label(70);
        assert_eq!(text, "High");
        let (text, _) = level_label(71);
        assert_eq!(text, "Very High");
    }

    #[test]
    fn test_is_key_service() {
        assert!(is_key_service(ServiceType::FireStation));
        assert!(is_key_service(ServiceType::PoliceStation));
        assert!(is_key_service(ServiceType::Hospital));
        assert!(is_key_service(ServiceType::ElementarySchool));
        assert!(is_key_service(ServiceType::SmallPark));
        assert!(is_key_service(ServiceType::BusDepot));
        assert!(is_key_service(ServiceType::TrainStation));

        // Non-key services
        assert!(!is_key_service(ServiceType::Landfill));
        assert!(!is_key_service(ServiceType::Cemetery));
        assert!(!is_key_service(ServiceType::Incinerator));
        assert!(!is_key_service(ServiceType::CityHall));
    }

    #[test]
    fn test_selected_cell_default() {
        let sc = SelectedCell::default();
        assert!(sc.0.is_none());
    }
}

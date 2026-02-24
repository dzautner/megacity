//! Cell Tooltips on Hover (UX-006).
//!
//! When the player hovers over any grid cell for 500ms, a tooltip appears
//! showing relevant information:
//! - Cell type, zone, elevation
//! - For buildings: zone type, level, occupancy
//! - For roads: road type, traffic density
//!
//! The tooltip is hidden during drag operations and repositions 20px offset
//! from the cursor.

use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts};

use rendering::camera::{CameraDrag, LeftClickDrag};
use rendering::input::CursorGridPos;
use simulation::buildings::Building;
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::services::ServiceBuilding;
use simulation::SaveLoadState;
use simulation::traffic::TrafficGrid;
use simulation::utilities::UtilitySource;

/// How long (seconds) the cursor must hover the same cell before showing the tooltip.
const HOVER_DELAY: f32 = 0.5;

/// Pixel offset from the cursor to the tooltip.
const TOOLTIP_OFFSET: f32 = 20.0;

/// Tracks which cell is being hovered and for how long.
#[derive(Resource, Default)]
pub struct CellHoverState {
    /// The grid cell currently under the cursor.
    pub cell: Option<(i32, i32)>,
    /// Accumulated hover time on the current cell (seconds).
    pub elapsed: f32,
}

/// Plugin that registers the cell tooltip system and state.
pub struct CellTooltipPlugin;

impl Plugin for CellTooltipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CellHoverState>()
            .add_systems(
                Update,
                cell_tooltip_ui.run_if(in_state(SaveLoadState::Idle)),
            );
    }
}

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

fn road_type_label(rt: simulation::grid::RoadType) -> &'static str {
    match rt {
        simulation::grid::RoadType::Local => "Local Road",
        simulation::grid::RoadType::Avenue => "Avenue",
        simulation::grid::RoadType::Boulevard => "Boulevard",
        simulation::grid::RoadType::Highway => "Highway",
        simulation::grid::RoadType::OneWay => "One-Way Road",
        simulation::grid::RoadType::Path => "Pedestrian Path",
    }
}

fn traffic_label(density: u16) -> (&'static str, egui::Color32) {
    if density == 0 {
        ("None", egui::Color32::from_rgb(80, 200, 80))
    } else if density <= 5 {
        ("Light", egui::Color32::from_rgb(120, 200, 80))
    } else if density <= 12 {
        ("Moderate", egui::Color32::from_rgb(220, 180, 50))
    } else if density <= 20 {
        ("Heavy", egui::Color32::from_rgb(220, 120, 50))
    } else {
        ("Gridlock", egui::Color32::from_rgb(220, 50, 50))
    }
}

#[allow(clippy::too_many_arguments)]
fn cell_tooltip_ui(
    mut contexts: EguiContexts,
    cursor: Res<CursorGridPos>,
    grid: Res<WorldGrid>,
    traffic: Res<TrafficGrid>,
    camera_drag: Res<CameraDrag>,
    left_drag: Res<LeftClickDrag>,
    time: Res<Time>,
    mut hover: ResMut<CellHoverState>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    utilities: Query<&UtilitySource>,
) {
    // Suppress during any drag operation.
    if camera_drag.dragging || left_drag.is_dragging {
        hover.elapsed = 0.0;
        hover.cell = None;
        return;
    }

    if !cursor.valid {
        hover.elapsed = 0.0;
        hover.cell = None;
        return;
    }

    let gx = cursor.grid_x;
    let gy = cursor.grid_y;

    // Reset timer when cell changes.
    if hover.cell != Some((gx, gy)) {
        hover.cell = Some((gx, gy));
        hover.elapsed = 0.0;
    }

    hover.elapsed += time.delta_secs();

    if hover.elapsed < HOVER_DELAY {
        return;
    }

    let ux = gx as usize;
    let uy = gy as usize;

    if !grid.in_bounds(ux, uy) {
        return;
    }

    let cell = grid.get(ux, uy);
    let ctx = contexts.ctx_mut();

    // Need a pointer position for placement.
    let Some(pointer_pos) = ctx.pointer_hover_pos() else {
        return;
    };

    let offset = egui::vec2(TOOLTIP_OFFSET, TOOLTIP_OFFSET);
    let label_pos = pointer_pos + offset;

    egui::Area::new(egui::Id::new("cell_hover_tooltip"))
        .fixed_pos(egui::pos2(label_pos.x, label_pos.y))
        .interactable(false)
        .order(egui::Order::Tooltip)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 220))
                .show(ui, |ui| {
                    ui.set_max_width(220.0);

                    // --- Cell type header ---
                    let type_str = match cell.cell_type {
                        CellType::Grass => "Grass",
                        CellType::Water => "Water",
                        CellType::Road => "Road",
                    };
                    ui.label(
                        egui::RichText::new(type_str)
                            .strong()
                            .size(13.0)
                            .color(egui::Color32::WHITE),
                    );

                    ui.separator();

                    // --- Elevation ---
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Elevation:")
                                .size(11.0)
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:.1}m", cell.elevation * 100.0))
                                .size(11.0)
                                .color(egui::Color32::WHITE),
                        );
                    });

                    // --- Zone (only for non-water cells) ---
                    if cell.cell_type != CellType::Water {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Zone:")
                                    .size(11.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                            ui.label(
                                egui::RichText::new(zone_label(cell.zone))
                                    .size(11.0)
                                    .color(egui::Color32::WHITE),
                            );
                        });
                    }

                    // --- Road info ---
                    if cell.cell_type == CellType::Road {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Road:")
                                    .size(11.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                            ui.label(
                                egui::RichText::new(road_type_label(cell.road_type))
                                    .size(11.0)
                                    .color(egui::Color32::WHITE),
                            );
                        });

                        // Traffic density
                        let density = traffic.get(ux, uy);
                        let (traffic_str, traffic_color) = traffic_label(density);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Traffic:")
                                    .size(11.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                            ui.label(
                                egui::RichText::new(traffic_str)
                                    .size(11.0)
                                    .color(traffic_color),
                            );
                        });
                    }

                    // --- Building info ---
                    if let Some(building_entity) = cell.building_id {
                        // Zoned building
                        if let Ok(building) = buildings.get(building_entity) {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} (L{})",
                                    zone_label(building.zone_type),
                                    building.level
                                ))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(180, 220, 255)),
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Occupancy:")
                                        .size(11.0)
                                        .color(egui::Color32::LIGHT_GRAY),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}/{}",
                                        building.occupants, building.capacity
                                    ))
                                    .size(11.0)
                                    .color(egui::Color32::WHITE),
                                );
                            });
                        }

                        // Service building
                        if let Ok(service) = services.get(building_entity) {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(service.service_type.name())
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(180, 220, 255)),
                            );
                        }

                        // Utility building
                        if let Ok(utility) = utilities.get(building_entity) {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(utility.utility_type.name())
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(180, 220, 255)),
                            );
                        }
                    }
                });
        });
}

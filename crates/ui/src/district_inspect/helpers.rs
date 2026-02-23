//! Pure helper functions for district inspection (testable without ECS).

use bevy_egui::egui;

use simulation::config::CELL_SIZE;
use simulation::districts::DistrictMap;

/// Convert a grid cell (gx, gy) to a world-space center coordinate.
pub fn grid_to_world_center(gx: usize, gy: usize) -> (f32, f32) {
    let wx = gx as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    let wy = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    (wx, wy)
}

/// Check if a service building's coverage radius overlaps with a grid cell.
pub fn service_covers_cell(
    service_x: usize,
    service_y: usize,
    radius: f32,
    cell_x: usize,
    cell_y: usize,
) -> bool {
    let (swx, swy) = grid_to_world_center(service_x, service_y);
    let (cwx, cwy) = grid_to_world_center(cell_x, cell_y);
    let dx = swx - cwx;
    let dy = swy - cwy;
    dx * dx + dy * dy <= radius * radius
}

/// Format a happiness value as a colored label descriptor.
pub fn happiness_label(happiness: f32) -> &'static str {
    if happiness >= 80.0 {
        "Excellent"
    } else if happiness >= 60.0 {
        "Good"
    } else if happiness >= 40.0 {
        "Fair"
    } else if happiness >= 20.0 {
        "Poor"
    } else {
        "Critical"
    }
}

/// Color for a happiness value.
pub fn happiness_color(happiness: f32) -> egui::Color32 {
    if happiness >= 80.0 {
        egui::Color32::from_rgb(50, 200, 50) // green
    } else if happiness >= 60.0 {
        egui::Color32::from_rgb(120, 200, 50) // light green
    } else if happiness >= 40.0 {
        egui::Color32::from_rgb(220, 220, 50) // yellow
    } else if happiness >= 20.0 {
        egui::Color32::from_rgb(220, 150, 50) // orange
    } else {
        egui::Color32::from_rgb(220, 50, 50) // red
    }
}

/// Get district index from a player-defined district map for a grid cell,
/// or fall back to the automatic statistical district.
pub fn resolve_district_index(district_map: &DistrictMap, gx: usize, gy: usize) -> Option<usize> {
    district_map.get_district_index_at(gx, gy)
}

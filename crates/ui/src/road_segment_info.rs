//! Road Segment Info Panel (UX-064).
//!
//! When a road segment is selected via the Inspect tool, displays:
//! - Road type and width (lane count)
//! - Segment length in meters
//! - Current traffic volume (sum across rasterized cells)
//! - Congestion level as LOS grade A-F
//! - Connected intersection nodes (start/end node positions)
//! - Monthly maintenance cost estimate for this segment

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::input::{ActiveTool, CursorGridPos};
use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, WorldGrid};
use simulation::road_maintenance::{RoadConditionGrid, RoadMaintenanceBudget};
use simulation::road_segments::{RoadSegmentStore, SegmentId};
use simulation::traffic::TrafficGrid;

/// Resource tracking the currently selected road segment.
#[derive(Resource, Default)]
pub struct SelectedRoadSegment(pub Option<SegmentId>);

/// Precomputed info for the selected road segment, refreshed each frame.
#[derive(Resource, Default)]
pub struct RoadSegmentInfoCache {
    pub road_type_label: String,
    pub lane_count: u8,
    pub length_meters: f32,
    pub traffic_volume: u32,
    pub congestion: f32,
    pub los_grade: char,
    pub start_node_pos: [f32; 2],
    pub end_node_pos: [f32; 2],
    pub monthly_maintenance: f64,
    pub avg_condition: f32,
    pub cell_count: usize,
    pub valid: bool,
}

/// System that detects road cell clicks and selects the corresponding road segment.
///
/// Runs on Update. When the user left-clicks while in Inspect mode and the cursor
/// is over a road cell, we find the road segment whose rasterized cells contain
/// that grid coordinate and store its ID in `SelectedRoadSegment`.
pub fn detect_road_segment_selection(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    grid: Res<WorldGrid>,
    segments: Res<RoadSegmentStore>,
    mut selected: ResMut<SelectedRoadSegment>,
) {
    if !buttons.just_pressed(MouseButton::Left) || !cursor.valid {
        return;
    }

    // Only detect in Inspect mode
    if *tool != ActiveTool::Inspect {
        selected.0 = None;
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    let cell = grid.get(gx, gy);

    // If the cell has a building, the building inspector takes precedence
    if cell.building_id.is_some() {
        selected.0 = None;
        return;
    }

    // Only select on road cells
    if cell.cell_type != CellType::Road {
        selected.0 = None;
        return;
    }

    // Find a segment whose rasterized cells include (gx, gy)
    let found = segments
        .segments
        .iter()
        .find(|s| s.rasterized_cells.contains(&(gx, gy)))
        .map(|s| s.id);

    selected.0 = found;
}

/// System that refreshes the info cache for the selected road segment.
pub fn refresh_road_segment_info(
    selected: Res<SelectedRoadSegment>,
    segments: Res<RoadSegmentStore>,
    traffic: Res<TrafficGrid>,
    condition_grid: Res<RoadConditionGrid>,
    maint_budget: Res<RoadMaintenanceBudget>,
    mut cache: ResMut<RoadSegmentInfoCache>,
) {
    let Some(seg_id) = selected.0 else {
        cache.valid = false;
        return;
    };

    let Some(segment) = segments.get_segment(seg_id) else {
        cache.valid = false;
        return;
    };

    // Road type label
    cache.road_type_label = road_type_label(segment.road_type);
    cache.lane_count = segment.road_type.lane_count();

    // Length in meters: arc_length is in world units, CELL_SIZE = 16.0 world units.
    // We treat 1 cell (16 world units) as ~16 meters for display.
    cache.length_meters = segment.arc_length;

    // Traffic volume: sum density across all rasterized cells
    let mut total_traffic: u32 = 0;
    let mut total_congestion: f32 = 0.0;
    let mut total_condition: f32 = 0.0;
    let cell_count = segment.rasterized_cells.len();

    for &(cx, cy) in &segment.rasterized_cells {
        total_traffic += traffic.get(cx, cy) as u32;
        total_congestion += traffic.congestion_level(cx, cy);
        total_condition += condition_grid.get(cx, cy) as f32;
    }

    cache.traffic_volume = total_traffic;
    cache.cell_count = cell_count;

    // Average congestion across the segment
    cache.congestion = if cell_count > 0 {
        total_congestion / cell_count as f32
    } else {
        0.0
    };

    // LOS grade: A (free flow) to F (breakdown)
    cache.los_grade = congestion_to_los(cache.congestion);

    // Average road condition
    cache.avg_condition = if cell_count > 0 {
        total_condition / cell_count as f32
    } else {
        0.0
    };

    // Connected intersection nodes
    if let Some(start_node) = segments.get_node(segment.start_node) {
        cache.start_node_pos = [start_node.position.x, start_node.position.y];
    }
    if let Some(end_node) = segments.get_node(segment.end_node) {
        cache.end_node_pos = [end_node.position.x, end_node.position.y];
    }

    // Monthly maintenance cost for this segment
    // Based on cell count * cost_per_cell * budget_level
    cache.monthly_maintenance =
        cell_count as f64 * maint_budget.cost_per_cell * maint_budget.budget_level as f64;

    cache.valid = true;
}

/// System that renders the Road Segment Info Panel using egui.
pub fn road_segment_info_ui(mut contexts: EguiContexts, cache: Res<RoadSegmentInfoCache>) {
    if !cache.valid {
        return;
    }

    egui::Window::new("Road Segment Info")
        .default_width(280.0)
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading(&cache.road_type_label);
            ui.separator();

            egui::Grid::new("road_seg_overview")
                .num_columns(2)
                .show(ui, |ui| {
                    // Road type and width
                    ui.label("Lanes:");
                    ui.label(format!("{}", cache.lane_count));
                    ui.end_row();

                    // Length in meters
                    ui.label("Length:");
                    ui.label(format!("{:.0} m", cache.length_meters));
                    ui.end_row();

                    // Traffic volume
                    ui.label("Traffic Volume:");
                    ui.label(format!("{}", cache.traffic_volume));
                    ui.end_row();

                    // Congestion level with LOS grade
                    ui.label("Congestion:");
                    let los_color = los_color(cache.los_grade);
                    ui.colored_label(
                        los_color,
                        format!("LOS {} ({:.0}%)", cache.los_grade, cache.congestion * 100.0),
                    );
                    ui.end_row();

                    // Road condition
                    ui.label("Condition:");
                    let cond_pct = cache.avg_condition / 255.0 * 100.0;
                    let cond_color = if cond_pct >= 60.0 {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else if cond_pct >= 30.0 {
                        egui::Color32::from_rgb(220, 180, 50)
                    } else {
                        egui::Color32::from_rgb(220, 50, 50)
                    };
                    ui.colored_label(cond_color, format!("{:.0}%", cond_pct));
                    ui.end_row();

                    // Maintenance cost
                    ui.label("Maintenance:");
                    ui.label(format!("${:.0}/mo", cache.monthly_maintenance));
                    ui.end_row();

                    // Cells covered
                    ui.label("Grid Cells:");
                    ui.label(format!("{}", cache.cell_count));
                    ui.end_row();
                });

            // Connected intersections
            ui.separator();
            ui.heading("Connected Nodes");
            egui::Grid::new("road_seg_nodes")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Start:");
                    let (sx, sy) =
                        world_to_grid_display(cache.start_node_pos[0], cache.start_node_pos[1]);
                    ui.label(format!("({}, {})", sx, sy));
                    ui.end_row();

                    ui.label("End:");
                    let (ex, ey) =
                        world_to_grid_display(cache.end_node_pos[0], cache.end_node_pos[1]);
                    ui.label(format!("({}, {})", ex, ey));
                    ui.end_row();
                });
        });
}

/// Convert world coordinates to grid coordinates for display.
fn world_to_grid_display(wx: f32, wy: f32) -> (i32, i32) {
    let gx = (wx / CELL_SIZE).floor() as i32;
    let gy = (wy / CELL_SIZE).floor() as i32;
    (gx, gy)
}

/// Map a road type to a human-readable label.
fn road_type_label(rt: simulation::grid::RoadType) -> String {
    use simulation::grid::RoadType;
    match rt {
        RoadType::Local => "Local Road".to_string(),
        RoadType::Avenue => "Avenue".to_string(),
        RoadType::Boulevard => "Boulevard".to_string(),
        RoadType::Highway => "Highway".to_string(),
        RoadType::OneWay => "One-Way Road".to_string(),
        RoadType::Path => "Pedestrian Path".to_string(),
    }
}

/// Map congestion level (0.0 - 1.0) to Level of Service grade A-F.
fn congestion_to_los(congestion: f32) -> char {
    if congestion < 0.15 {
        'A'
    } else if congestion < 0.30 {
        'B'
    } else if congestion < 0.45 {
        'C'
    } else if congestion < 0.65 {
        'D'
    } else if congestion < 0.85 {
        'E'
    } else {
        'F'
    }
}

/// Color for Level of Service grade.
fn los_color(grade: char) -> egui::Color32 {
    match grade {
        'A' => egui::Color32::from_rgb(50, 200, 50),  // green
        'B' => egui::Color32::from_rgb(120, 200, 50), // light green
        'C' => egui::Color32::from_rgb(220, 220, 50), // yellow
        'D' => egui::Color32::from_rgb(220, 150, 50), // orange
        'E' => egui::Color32::from_rgb(220, 80, 50),  // red-orange
        'F' => egui::Color32::from_rgb(220, 50, 50),  // red
        _ => egui::Color32::GRAY,
    }
}

pub struct RoadSegmentInfoPlugin;

impl Plugin for RoadSegmentInfoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedRoadSegment>()
            .init_resource::<RoadSegmentInfoCache>()
            .add_systems(
                Update,
                (
                    detect_road_segment_selection,
                    refresh_road_segment_info,
                    road_segment_info_ui,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::grid::RoadType;

    #[test]
    fn test_congestion_to_los_free_flow() {
        assert_eq!(congestion_to_los(0.0), 'A');
        assert_eq!(congestion_to_los(0.10), 'A');
    }

    #[test]
    fn test_congestion_to_los_moderate() {
        assert_eq!(congestion_to_los(0.15), 'B');
        assert_eq!(congestion_to_los(0.29), 'B');
    }

    #[test]
    fn test_congestion_to_los_approaching_unstable() {
        assert_eq!(congestion_to_los(0.30), 'C');
        assert_eq!(congestion_to_los(0.44), 'C');
    }

    #[test]
    fn test_congestion_to_los_unstable() {
        assert_eq!(congestion_to_los(0.45), 'D');
        assert_eq!(congestion_to_los(0.64), 'D');
    }

    #[test]
    fn test_congestion_to_los_at_capacity() {
        assert_eq!(congestion_to_los(0.65), 'E');
        assert_eq!(congestion_to_los(0.84), 'E');
    }

    #[test]
    fn test_congestion_to_los_breakdown() {
        assert_eq!(congestion_to_los(0.85), 'F');
        assert_eq!(congestion_to_los(1.0), 'F');
    }

    #[test]
    fn test_road_type_labels() {
        assert_eq!(road_type_label(RoadType::Local), "Local Road");
        assert_eq!(road_type_label(RoadType::Avenue), "Avenue");
        assert_eq!(road_type_label(RoadType::Boulevard), "Boulevard");
        assert_eq!(road_type_label(RoadType::Highway), "Highway");
        assert_eq!(road_type_label(RoadType::OneWay), "One-Way Road");
        assert_eq!(road_type_label(RoadType::Path), "Pedestrian Path");
    }

    #[test]
    fn test_world_to_grid_display() {
        // CELL_SIZE = 16.0
        let (gx, gy) = world_to_grid_display(48.0, 32.0);
        assert_eq!(gx, 3);
        assert_eq!(gy, 2);
    }

    #[test]
    fn test_world_to_grid_display_origin() {
        let (gx, gy) = world_to_grid_display(0.0, 0.0);
        assert_eq!(gx, 0);
        assert_eq!(gy, 0);
    }

    #[test]
    fn test_los_color_all_grades() {
        // Ensure all grades produce a valid color (not GRAY)
        for grade in ['A', 'B', 'C', 'D', 'E', 'F'] {
            let color = los_color(grade);
            assert_ne!(
                color,
                egui::Color32::GRAY,
                "Grade {} should have a color",
                grade
            );
        }
    }

    #[test]
    fn test_los_color_unknown_grade() {
        assert_eq!(los_color('X'), egui::Color32::GRAY);
    }

    #[test]
    fn test_road_segment_info_cache_default() {
        let cache = RoadSegmentInfoCache::default();
        assert!(!cache.valid);
        assert_eq!(cache.traffic_volume, 0);
        assert_eq!(cache.congestion, 0.0);
        assert_eq!(cache.monthly_maintenance, 0.0);
    }

    #[test]
    fn test_selected_road_segment_default() {
        let selected = SelectedRoadSegment::default();
        assert!(selected.0.is_none());
    }

    #[test]
    fn test_lane_count_matches_road_type() {
        assert_eq!(RoadType::Local.lane_count(), 2);
        assert_eq!(RoadType::Avenue.lane_count(), 4);
        assert_eq!(RoadType::Boulevard.lane_count(), 6);
        assert_eq!(RoadType::Highway.lane_count(), 4);
        assert_eq!(RoadType::OneWay.lane_count(), 2);
        assert_eq!(RoadType::Path.lane_count(), 0);
    }
}

//! Enhanced Click-to-Select (UX-009).
//!
//! Provides priority-based entity selection when the player left-clicks in
//! Inspect mode. Selection priority: citizens > buildings > road segments > cells.
//!
//! This module exports a [`SelectionKind`] resource that tracks what type of
//! entity was selected on the last click, enabling downstream UI panels to
//! display the appropriate info panel without conflicting with each other.

use bevy::prelude::*;

use simulation::buildings::Building;
use simulation::citizen::{Citizen, Position};
use simulation::config::CELL_SIZE;
use simulation::grid::WorldGrid;
use simulation::road_segments::{RoadSegmentStore, SegmentId};
use simulation::services::ServiceBuilding;

use crate::camera::LeftClickDrag;
use crate::input::{ActiveTool, CursorGridPos, SelectedBuilding};

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// What type of entity was selected on the last inspect-mode click.
///
/// Downstream UI panels (citizen info, building inspector, road segment info,
/// cell info, district inspect) should check this resource to decide whether
/// to show their panel. Only the panel matching the current `SelectionKind`
/// should be active.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub enum SelectionKind {
    /// No selection (initial state or click on void).
    #[default]
    None,
    /// A citizen entity was selected.
    Citizen(Entity),
    /// A building entity was selected (zone building or service/utility).
    Building(Entity),
    /// A road segment was selected by its ID.
    RoadSegment(SegmentId),
    /// An empty cell was selected at the given grid coordinates.
    Cell(usize, usize),
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Core priority-based selection system.
///
/// Runs on left-click when in Inspect mode. Checks in priority order:
/// 1. Citizens near the click position (within 2 cells radius)
/// 2. Buildings at the clicked grid cell (including multi-cell footprints)
/// 3. Road segments whose Bezier curve passes near the click position
/// 4. Fallback: empty cell selection
#[allow(clippy::too_many_arguments)]
pub fn enhanced_select_system(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    left_drag: Res<LeftClickDrag>,
    grid: Res<WorldGrid>,
    segments: Res<RoadSegmentStore>,
    citizens: Query<(Entity, &Position), With<Citizen>>,
    buildings: Query<(Entity, &Building)>,
    services: Query<(Entity, &ServiceBuilding)>,
    mut selection_kind: ResMut<SelectionKind>,
    mut selected_building: ResMut<SelectedBuilding>,
) {
    // Only respond to fresh left-clicks
    if !buttons.just_pressed(MouseButton::Left) || !cursor.valid {
        return;
    }

    // Suppress when camera is panning
    if left_drag.is_dragging {
        return;
    }

    // Only apply in Inspect mode
    if *tool != ActiveTool::Inspect {
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    // World-space click position (XZ plane, Y is up in 3D but grid uses x/z)
    let click_world = cursor.world_pos; // Vec2(world_x, world_z)

    // -----------------------------------------------------------------------
    // Priority 1: Citizens
    // -----------------------------------------------------------------------
    if let Some(citizen_entity) = find_nearest_citizen(&citizens, click_world) {
        *selection_kind = SelectionKind::Citizen(citizen_entity);
        selected_building.0 = None; // Clear building selection
        return;
    }

    // -----------------------------------------------------------------------
    // Priority 2: Buildings (zone buildings, service buildings, utilities)
    // -----------------------------------------------------------------------
    // First check the grid cell directly for a building_id
    let cell = grid.get(gx, gy);
    if let Some(building_entity) = cell.building_id {
        *selection_kind = SelectionKind::Building(building_entity);
        selected_building.0 = Some(building_entity);
        return;
    }

    // Also check multi-cell service buildings whose footprint covers (gx, gy)
    // but whose building_id might be stored on a different cell.
    if let Some(service_entity) = find_service_building_at(&services, gx, gy) {
        *selection_kind = SelectionKind::Building(service_entity);
        selected_building.0 = Some(service_entity);
        return;
    }

    // Also check zone buildings that might occupy the clicked cell
    if let Some(building_entity) = find_zone_building_at(&buildings, gx, gy) {
        *selection_kind = SelectionKind::Building(building_entity);
        selected_building.0 = Some(building_entity);
        return;
    }

    // -----------------------------------------------------------------------
    // Priority 3: Road segments
    // -----------------------------------------------------------------------
    if let Some(segment_id) = find_nearest_road_segment(&segments, click_world, gx, gy) {
        *selection_kind = SelectionKind::RoadSegment(segment_id);
        selected_building.0 = None;
        return;
    }

    // -----------------------------------------------------------------------
    // Priority 4: Empty cell
    // -----------------------------------------------------------------------
    if grid.in_bounds(gx, gy) {
        *selection_kind = SelectionKind::Cell(gx, gy);
        selected_building.0 = None;
    } else {
        *selection_kind = SelectionKind::None;
        selected_building.0 = None;
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Find the nearest citizen within a small radius of the click position.
/// Returns `Some(entity)` if a citizen is close enough, `None` otherwise.
fn find_nearest_citizen(
    citizens: &Query<(Entity, &Position), With<Citizen>>,
    click_world: Vec2,
) -> Option<Entity> {
    // Search radius: 2 cells (32 world units)
    let radius = CELL_SIZE * 2.0;
    let radius_sq = radius * radius;

    let mut best: Option<(Entity, f32)> = None;

    for (entity, pos) in citizens.iter() {
        let dx = pos.x - click_world.x;
        let dy = pos.y - click_world.y;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq < radius_sq && (best.is_none() || dist_sq < best.unwrap().1) {
            best = Some((entity, dist_sq));
        }
    }

    best.map(|(entity, _)| entity)
}

/// Find a service building whose footprint covers grid cell (gx, gy).
fn find_service_building_at(
    services: &Query<(Entity, &ServiceBuilding)>,
    gx: usize,
    gy: usize,
) -> Option<Entity> {
    for (entity, service) in services.iter() {
        let (fw, fh) = ServiceBuilding::footprint(service.service_type);
        let sx = service.grid_x;
        let sy = service.grid_y;

        if gx >= sx && gx < sx + fw && gy >= sy && gy < sy + fh {
            return Some(entity);
        }
    }
    None
}

/// Find a zone building at the given grid cell by querying Building components.
fn find_zone_building_at(
    buildings: &Query<(Entity, &Building)>,
    gx: usize,
    gy: usize,
) -> Option<Entity> {
    for (entity, building) in buildings.iter() {
        if building.grid_x == gx && building.grid_y == gy {
            return Some(entity);
        }
    }
    None
}

/// Find the nearest road segment to the click position.
///
/// First checks if the clicked grid cell is a road cell and finds the segment
/// that rasterized it. If not, samples nearby segments' Bezier curves to find
/// the closest one within a threshold distance.
fn find_nearest_road_segment(
    segments: &RoadSegmentStore,
    click_world: Vec2,
    gx: usize,
    gy: usize,
) -> Option<SegmentId> {
    // Fast path: check if (gx, gy) is in a segment's rasterized cells
    for segment in &segments.segments {
        if segment.rasterized_cells.contains(&(gx, gy)) {
            return Some(segment.id);
        }
    }

    // Slow path: find the nearest segment by sampling Bezier curves
    // Only search within a reasonable distance (1.5 cells = 24 world units)
    let threshold = CELL_SIZE * 1.5;
    let threshold_sq = threshold * threshold;

    let mut best: Option<(SegmentId, f32)> = None;

    for segment in &segments.segments {
        // Quick bounding-box reject: skip segments far from the click
        let min_x = segment
            .p0
            .x
            .min(segment.p1.x)
            .min(segment.p2.x)
            .min(segment.p3.x)
            - threshold;
        let max_x = segment
            .p0
            .x
            .max(segment.p1.x)
            .max(segment.p2.x)
            .max(segment.p3.x)
            + threshold;
        let min_y = segment
            .p0
            .y
            .min(segment.p1.y)
            .min(segment.p2.y)
            .min(segment.p3.y)
            - threshold;
        let max_y = segment
            .p0
            .y
            .max(segment.p1.y)
            .max(segment.p2.y)
            .max(segment.p3.y)
            + threshold;

        if click_world.x < min_x
            || click_world.x > max_x
            || click_world.y < min_y
            || click_world.y > max_y
        {
            continue;
        }

        // Sample the Bezier curve and find the closest point
        let dist_sq = min_distance_to_segment_sq(segment, click_world);
        if dist_sq < threshold_sq && (best.is_none() || dist_sq < best.unwrap().1) {
            best = Some((segment.id, dist_sq));
        }
    }

    best.map(|(id, _)| id)
}

/// Compute the minimum squared distance from a point to a Bezier road segment
/// by sampling the curve at multiple points.
fn min_distance_to_segment_sq(
    segment: &simulation::road_segments::RoadSegment,
    point: Vec2,
) -> f32 {
    let samples = 32;
    let mut min_dist_sq = f32::MAX;

    for i in 0..=samples {
        let t = i as f32 / samples as f32;
        let curve_pt = segment.evaluate(t);
        let dist_sq = (curve_pt - point).length_squared();
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
        }
    }

    min_dist_sq
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EnhancedSelectPlugin;

impl Plugin for EnhancedSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionKind>()
            .add_systems(Update, enhanced_select_system);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_kind_default() {
        let kind = SelectionKind::default();
        assert_eq!(kind, SelectionKind::None);
    }

    #[test]
    fn test_selection_kind_citizen() {
        let entity = Entity::from_raw(42);
        let kind = SelectionKind::Citizen(entity);
        assert!(matches!(kind, SelectionKind::Citizen(_)));
    }

    #[test]
    fn test_selection_kind_building() {
        let entity = Entity::from_raw(7);
        let kind = SelectionKind::Building(entity);
        assert!(matches!(kind, SelectionKind::Building(_)));
    }

    #[test]
    fn test_selection_kind_road_segment() {
        let kind = SelectionKind::RoadSegment(SegmentId(5));
        assert!(matches!(kind, SelectionKind::RoadSegment(_)));
    }

    #[test]
    fn test_selection_kind_cell() {
        let kind = SelectionKind::Cell(10, 20);
        assert_eq!(kind, SelectionKind::Cell(10, 20));
    }

    #[test]
    fn test_min_distance_to_straight_segment() {
        use simulation::road_segments::{RoadSegment, SegmentNodeId};

        // Create a straight segment from (0,0) to (100,0)
        let segment = RoadSegment {
            id: SegmentId(0),
            start_node: SegmentNodeId(0),
            end_node: SegmentNodeId(1),
            p0: Vec2::new(0.0, 0.0),
            p1: Vec2::new(33.3, 0.0),
            p2: Vec2::new(66.6, 0.0),
            p3: Vec2::new(100.0, 0.0),
            road_type: simulation::grid::RoadType::Local,
            arc_length: 100.0,
            rasterized_cells: vec![],
        };

        // Point directly on the segment midpoint
        let dist_sq = min_distance_to_segment_sq(&segment, Vec2::new(50.0, 0.0));
        assert!(dist_sq < 1.0, "Point on segment should be very close");

        // Point 10 units away perpendicular to the segment
        let dist_sq = min_distance_to_segment_sq(&segment, Vec2::new(50.0, 10.0));
        assert!(dist_sq < 110.0, "Point near segment should be close-ish");
        assert!(dist_sq > 90.0, "Point 10 away should be ~100 sq dist");
    }
}

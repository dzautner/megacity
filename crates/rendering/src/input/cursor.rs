use bevy::prelude::*;

use simulation::grid::WorldGrid;
use simulation::road_segments::RoadSegmentStore;

use super::types::{
    ActiveTool, CursorGridPos, GridSnap, IntersectionSnap, StatusMessage, INTERSECTION_SNAP_RADIUS,
};

/// Each frame, check if the cursor is near an existing segment node (intersection)
/// and update `IntersectionSnap` accordingly.
pub fn update_intersection_snap(
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    segments: Res<RoadSegmentStore>,
    mut snap: ResMut<IntersectionSnap>,
) {
    snap.snapped_pos = None;

    if !cursor.valid {
        return;
    }

    // Only snap for road tools
    let is_road_tool = matches!(
        *tool,
        ActiveTool::Road
            | ActiveTool::RoadAvenue
            | ActiveTool::RoadBoulevard
            | ActiveTool::RoadHighway
            | ActiveTool::RoadOneWay
            | ActiveTool::RoadPath
    );
    if !is_road_tool {
        return;
    }

    let cursor_pos = cursor.world_pos;
    let mut best_dist = INTERSECTION_SNAP_RADIUS;
    let mut best_pos: Option<Vec2> = None;

    for node in &segments.nodes {
        let dist = (node.position - cursor_pos).length();
        if dist < best_dist {
            best_dist = dist;
            best_pos = Some(node.position);
        }
    }

    snap.snapped_pos = best_pos;
}

pub fn tick_status_message(time: Res<Time>, mut status: ResMut<StatusMessage>) {
    if status.timer > 0.0 {
        status.timer -= time.delta_secs();
    }
}

pub fn update_cursor_grid_pos(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut cursor: ResMut<CursorGridPos>,
    grid: Res<WorldGrid>,
    grid_snap: Res<GridSnap>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.get_single() else {
        return;
    };

    if let Some(screen_pos) = window.cursor_position() {
        // Ray-plane intersection against Y=0 ground plane
        if let Ok(ray) = camera.viewport_to_world(cam_transform, screen_pos) {
            if ray.direction.y.abs() > 0.001 {
                let t = -ray.origin.y / ray.direction.y;
                if t > 0.0 {
                    let hit = ray.origin + ray.direction * t;
                    // 3D: hit.x -> grid X, hit.z -> grid Y
                    let (gx, gy) = WorldGrid::world_to_grid(hit.x, hit.z);

                    // When grid snap is enabled, snap world_pos to the cell center
                    if grid_snap.enabled && gx >= 0 && gy >= 0 {
                        let (cx, cz) = WorldGrid::grid_to_world(gx as usize, gy as usize);
                        cursor.world_pos = Vec2::new(cx, cz);
                    } else {
                        cursor.world_pos = Vec2::new(hit.x, hit.z);
                    }

                    cursor.grid_x = gx;
                    cursor.grid_y = gy;
                    cursor.valid = gx >= 0 && gy >= 0 && grid.in_bounds(gx as usize, gy as usize);
                    return;
                }
            }
        }
        cursor.valid = false;
    } else {
        cursor.valid = false;
    }
}

/// Approximate arc length of a cubic Bezier by sampling.
pub(crate) fn estimate_arc_length(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    let steps = 64;
    let mut length = 0.0_f32;
    let mut prev = p0;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        let pt = u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3;
        length += (pt - prev).length();
        prev = pt;
    }
    length
}

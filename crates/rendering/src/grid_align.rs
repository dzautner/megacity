//! Grid-cell center alignment for road snapping (UX fix for #1226).
//!
//! Parallel and angle snapping can produce sub-pixel world coordinates that
//! misalign with the grid used by the zoning system.  This module provides a
//! final alignment pass that snaps road endpoint coordinates to the nearest
//! grid-cell center when the offset is within `CELL_SIZE / 2`, guaranteeing
//! that rasterized road cells always produce valid zone frontage.

use bevy::prelude::*;

use simulation::grid::WorldGrid;

use crate::angle_snap::AngleSnapState;
use crate::input::{ActiveTool, CursorGridPos, DrawPhase, IntersectionSnap, RoadDrawState};

// ---------------------------------------------------------------------------
// Public helper
// ---------------------------------------------------------------------------

/// Snap a world-space coordinate to the nearest grid-cell center.
///
/// The cell center for grid coordinate `(col, row)` is:
///   `(col * CELL_SIZE + CELL_SIZE / 2, row * CELL_SIZE + CELL_SIZE / 2)`
///
/// If the point is already within `CELL_SIZE / 2` of a cell center (i.e. it
/// lies inside that cell), it is moved to that center.  Since every point in
/// the world is always within some cell, this is effectively an unconditional
/// snap -- but the intent is to correct sub-pixel drift introduced by
/// snapping helpers while leaving non-snapped positions untouched (those are
/// already grid-aligned by `update_cursor_grid_pos` when grid-snap is on).
pub fn snap_to_cell_center(pos: Vec2) -> Vec2 {
    let (gx, gy) = WorldGrid::world_to_grid(pos.x, pos.y);
    // Clamp to valid range (non-negative) before converting
    if gx < 0 || gy < 0 {
        return pos;
    }
    let (cx, cy) = WorldGrid::grid_to_world(gx as usize, gy as usize);
    Vec2::new(cx, cy)
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn is_road_tool(tool: &ActiveTool) -> bool {
    matches!(
        tool,
        ActiveTool::Road
            | ActiveTool::RoadAvenue
            | ActiveTool::RoadBoulevard
            | ActiveTool::RoadHighway
            | ActiveTool::RoadOneWay
            | ActiveTool::RoadPath
    )
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Align `CursorGridPos.world_pos` to the nearest grid-cell center when a
/// road tool is active.  This runs **after** parallel snap and angle snap have
/// modified the cursor, but **before** `handle_tool_input` commits a segment.
pub fn align_cursor_to_grid(tool: Res<ActiveTool>, mut cursor: ResMut<CursorGridPos>) {
    if !cursor.valid || !is_road_tool(&tool) {
        return;
    }
    cursor.world_pos = snap_to_cell_center(cursor.world_pos);
}

/// Align the angle-snap result to the nearest grid-cell center so that
/// angle-snapped endpoints also produce correct zone frontage.
pub fn align_angle_snap_to_grid(
    tool: Res<ActiveTool>,
    draw_state: Res<RoadDrawState>,
    mut angle_snap: ResMut<AngleSnapState>,
) {
    if !angle_snap.active || !is_road_tool(&tool) {
        return;
    }
    if draw_state.phase != DrawPhase::PlacedStart {
        return;
    }
    angle_snap.snapped_pos = snap_to_cell_center(angle_snap.snapped_pos);
}

/// Align the intersection-snap result to the nearest grid-cell center so
/// that snapping to an existing node also produces grid-aligned endpoints.
pub fn align_intersection_snap_to_grid(tool: Res<ActiveTool>, mut snap: ResMut<IntersectionSnap>) {
    if !is_road_tool(&tool) {
        return;
    }
    if let Some(pos) = snap.snapped_pos {
        snap.snapped_pos = Some(snap_to_cell_center(pos));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::config::CELL_SIZE;

    #[test]
    fn test_snap_to_cell_center_exact() {
        // A point already at a cell center should not move.
        let center = Vec2::new(CELL_SIZE * 0.5, CELL_SIZE * 0.5); // cell (0,0)
        assert_eq!(snap_to_cell_center(center), center);
    }

    #[test]
    fn test_snap_to_cell_center_offset() {
        // A point slightly off the cell (0,0) center should snap back.
        let off = Vec2::new(CELL_SIZE * 0.5 + 0.3, CELL_SIZE * 0.5 - 0.7);
        let snapped = snap_to_cell_center(off);
        let expected = Vec2::new(CELL_SIZE * 0.5, CELL_SIZE * 0.5);
        assert!((snapped - expected).length() < f32::EPSILON);
    }

    #[test]
    fn test_snap_to_cell_center_different_cell() {
        // Cell (3, 5): center should be (3*16+8, 5*16+8) = (56, 88)
        let pos = Vec2::new(55.2, 87.9);
        let snapped = snap_to_cell_center(pos);
        assert!((snapped.x - 56.0).abs() < f32::EPSILON);
        assert!((snapped.y - 88.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_snap_to_cell_center_negative_coords() {
        // Negative coords should pass through unchanged.
        let neg = Vec2::new(-5.0, -10.0);
        assert_eq!(snap_to_cell_center(neg), neg);
    }

    #[test]
    fn test_snap_to_cell_center_boundary() {
        // Right at the cell boundary: floor-based grid assignment means
        // (16.0, 16.0) -> cell (1,1) -> center (24, 24)
        let boundary = Vec2::new(CELL_SIZE, CELL_SIZE);
        let snapped = snap_to_cell_center(boundary);
        let expected = Vec2::new(CELL_SIZE + CELL_SIZE * 0.5, CELL_SIZE + CELL_SIZE * 0.5);
        assert!((snapped - expected).length() < f32::EPSILON);
    }
}

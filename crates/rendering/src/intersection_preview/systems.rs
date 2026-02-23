//! Plugin and ECS systems for intersection preview detection and rendering.

use bevy::prelude::*;

use simulation::road_segments::RoadSegmentStore;

use crate::input::{ActiveTool, CursorGridPos, DrawPhase, IntersectionSnap, RoadDrawState};

use super::geometry::find_classified_intersections;
use super::types::{IntersectionKind, IntersectionPreviewState, GIZMO_Y};

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct IntersectionPreviewPlugin;

impl Plugin for IntersectionPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IntersectionPreviewState>().add_systems(
            Update,
            (
                compute_intersection_preview
                    .after(crate::input::update_cursor_grid_pos)
                    .after(crate::angle_snap::update_angle_snap)
                    .after(crate::input::update_intersection_snap),
                draw_intersection_preview_markers.after(compute_intersection_preview),
            ),
        );
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Each frame, compute intersection points between the preview road and
/// existing segments, classify them, and store the results.
#[allow(clippy::too_many_arguments)]
fn compute_intersection_preview(
    draw_state: Res<RoadDrawState>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    angle_snap: Res<crate::angle_snap::AngleSnapState>,
    snap: Res<IntersectionSnap>,
    segment_store: Res<RoadSegmentStore>,
    mut state: ResMut<IntersectionPreviewState>,
) {
    state.intersections.clear();

    // Only compute when actively drawing a road
    if draw_state.phase != DrawPhase::PlacedStart || !cursor.valid {
        return;
    }

    // Only for road tools
    if tool.road_type().is_none() {
        return;
    }

    // No segments to intersect with
    if segment_store.segments.is_empty() {
        return;
    }

    let start = draw_state.start_pos;
    // Use the same end-point logic as draw_bezier_preview
    let end = if let Some(snapped) = snap.snapped_pos {
        snapped
    } else if angle_snap.active {
        angle_snap.snapped_pos
    } else {
        cursor.world_pos
    };

    // Skip if road is too short
    if (end - start).length() < 1.0 {
        return;
    }

    // Build Bezier control points (straight line, same as draw_bezier_preview)
    let p0 = start;
    let p3 = end;
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    state.intersections = find_classified_intersections(p0, p1, p2, p3, &segment_store);
}

/// Draw diamond-shaped markers at each detected intersection point.
/// Green = new node, Yellow = snap to existing node.
fn draw_intersection_preview_markers(
    state: Res<IntersectionPreviewState>,
    draw_state: Res<RoadDrawState>,
    mut gizmos: Gizmos,
) {
    if draw_state.phase != DrawPhase::PlacedStart {
        return;
    }

    for intersection in &state.intersections {
        let pt = intersection.position;
        let pos = Vec3::new(pt.x, GIZMO_Y + 0.1, pt.y);

        let (fill_color, outline_color) = match intersection.kind {
            IntersectionKind::NewNode => (
                Color::srgba(0.1, 0.9, 0.2, 0.85), // Green fill
                Color::srgba(0.2, 1.0, 0.3, 1.0),  // Bright green outline
            ),
            IntersectionKind::SnapToExisting => (
                Color::srgba(1.0, 0.85, 0.1, 0.85), // Yellow fill
                Color::srgba(1.0, 0.95, 0.3, 1.0),  // Bright yellow outline
            ),
        };

        let diamond_size = 6.0;

        // Diamond shape (rotated square)
        let top = pos + Vec3::new(0.0, 0.0, -diamond_size);
        let right = pos + Vec3::new(diamond_size, 0.0, 0.0);
        let bottom = pos + Vec3::new(0.0, 0.0, diamond_size);
        let left = pos + Vec3::new(-diamond_size, 0.0, 0.0);

        // Outline
        gizmos.line(top, right, outline_color);
        gizmos.line(right, bottom, outline_color);
        gizmos.line(bottom, left, outline_color);
        gizmos.line(left, top, outline_color);

        // Inner diamond (smaller, simulates fill)
        let inner_size = diamond_size * 0.5;
        let i_top = pos + Vec3::new(0.0, 0.0, -inner_size);
        let i_right = pos + Vec3::new(inner_size, 0.0, 0.0);
        let i_bottom = pos + Vec3::new(0.0, 0.0, inner_size);
        let i_left = pos + Vec3::new(-inner_size, 0.0, 0.0);

        gizmos.line(i_top, i_right, fill_color);
        gizmos.line(i_right, i_bottom, fill_color);
        gizmos.line(i_bottom, i_left, fill_color);
        gizmos.line(i_left, i_top, fill_color);

        // Cross lines inside for extra visibility
        gizmos.line(top, bottom, fill_color);
        gizmos.line(left, right, fill_color);

        // Circle around diamond for emphasis
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            diamond_size * 1.3,
            outline_color,
        );
    }
}

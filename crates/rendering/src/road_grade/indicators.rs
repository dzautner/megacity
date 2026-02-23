//! Core system that draws road grade and elevation indicators using gizmos
//! during road preview (freeform Bezier drawing).

use bevy::prelude::*;

use simulation::grid::{CellType, WorldGrid};

use crate::input::{ActiveTool, CursorGridPos, DrawPhase, RoadDrawState};

use super::constants::{
    COLOR_BRIDGE, COLOR_TUNNEL, ELEVATION_DISPLAY_SCALE, ELEVATION_SAMPLE_INTERVAL, GIZMO_Y,
    HILL_ELEVATION_THRESHOLD, INDICATOR_RADIUS,
};
use super::helpers::{
    approximate_arc_length, evaluate_bezier, grade_to_color, is_road_tool, sample_cell_type_at,
    sample_elevation_at,
};

/// Draws road grade and elevation indicators using gizmos during road preview.
pub fn draw_road_grade_indicators(
    draw_state: Res<RoadDrawState>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    grid: Res<WorldGrid>,
    mut gizmos: Gizmos,
) {
    // Only active during road placement preview
    if draw_state.phase != DrawPhase::PlacedStart || !cursor.valid {
        return;
    }

    // Only for road tools
    if !is_road_tool(&tool) {
        return;
    }

    let start = draw_state.start_pos;
    let end = cursor.world_pos;

    // Build Bezier control points (same as cursor_preview::draw_bezier_preview)
    let p0 = start;
    let p3 = end;
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    // Compute arc length to determine number of samples
    let arc_length = approximate_arc_length(p0, p1, p2, p3);
    if arc_length < 1.0 {
        return;
    }

    // Number of elevation label samples based on interval
    let num_samples = ((arc_length / ELEVATION_SAMPLE_INTERVAL).ceil() as usize).max(2);

    // Fine-grained sampling for continuous grade coloring and bridge/tunnel detection
    let fine_steps = ((arc_length / 2.0).ceil() as usize).clamp(16, 256);

    draw_grade_colored_segments(p0, p1, p2, p3, fine_steps, &grid, &mut gizmos);
    draw_elevation_markers(
        p0,
        p1,
        p2,
        p3,
        num_samples,
        fine_steps,
        arc_length,
        &grid,
        &mut gizmos,
    );
}

/// Draw grade-colored line segments and bridge/tunnel indicators along the curve.
#[allow(clippy::too_many_arguments)]
fn draw_grade_colored_segments(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    fine_steps: usize,
    grid: &WorldGrid,
    gizmos: &mut Gizmos,
) {
    let mut fine_prev_pt = evaluate_bezier(p0, p1, p2, p3, 0.0);
    let mut fine_prev_elev = sample_elevation_at(grid, fine_prev_pt);

    for i in 1..=fine_steps {
        let t = i as f32 / fine_steps as f32;
        let pt = evaluate_bezier(p0, p1, p2, p3, t);
        let seg_len = (pt - fine_prev_pt).length();

        let elev = sample_elevation_at(grid, pt);

        // Grade between consecutive fine samples
        let grade = if seg_len > 0.01 {
            ((elev - fine_prev_elev) * ELEVATION_DISPLAY_SCALE).abs() / seg_len
        } else {
            0.0
        };

        // Color the segment by grade
        let grade_color = grade_to_color(grade);

        let prev_3d = Vec3::new(fine_prev_pt.x, GIZMO_Y, fine_prev_pt.y);
        let curr_3d = Vec3::new(pt.x, GIZMO_Y, pt.y);
        gizmos.line(prev_3d, curr_3d, grade_color);

        // Check for bridge (water crossing)
        let cell_type = sample_cell_type_at(grid, pt);
        if cell_type == CellType::Water {
            let center = Vec3::new(pt.x, GIZMO_Y + 0.5, pt.y);
            gizmos.circle(
                Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                INDICATOR_RADIUS,
                COLOR_BRIDGE,
            );
        }

        // Check for tunnel (hill crossing)
        if elev > HILL_ELEVATION_THRESHOLD {
            let center = Vec3::new(pt.x, GIZMO_Y + 0.5, pt.y);
            gizmos.circle(
                Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                INDICATOR_RADIUS * 0.8,
                COLOR_TUNNEL,
            );
        }

        fine_prev_pt = pt;
        fine_prev_elev = elev;
    }
}

/// Draw elevation diamond markers at regular intervals along the curve.
#[allow(clippy::too_many_arguments)]
fn draw_elevation_markers(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    num_samples: usize,
    walk_steps: usize,
    arc_length: f32,
    grid: &WorldGrid,
    gizmos: &mut Gizmos,
) {
    let label_step = if num_samples > 1 {
        arc_length / (num_samples - 1) as f32
    } else {
        arc_length
    };
    let mut next_label_dist = 0.0_f32;
    let mut label_index = 0_usize;
    let mut prev_elevation: Option<f32> = None;
    let mut prev_world_dist: f32 = 0.0;

    let mut sample_cum_dist = 0.0_f32;
    let mut sample_prev_pt = evaluate_bezier(p0, p1, p2, p3, 0.0);

    for i in 0..=walk_steps {
        let t = i as f32 / walk_steps as f32;
        let pt = evaluate_bezier(p0, p1, p2, p3, t);

        if i > 0 {
            sample_cum_dist += (pt - sample_prev_pt).length();
        }
        sample_prev_pt = pt;

        if sample_cum_dist >= next_label_dist || i == 0 {
            let elev = sample_elevation_at(grid, pt);
            let display_elev = elev * ELEVATION_DISPLAY_SCALE;

            // Elevation marker position
            let center = Vec3::new(pt.x, GIZMO_Y + 1.0, pt.y);

            // Grade color for the marker
            let grade = if let Some(pe) = prev_elevation {
                let dist_delta = sample_cum_dist - prev_world_dist;
                if dist_delta > 0.01 {
                    ((elev - pe) * ELEVATION_DISPLAY_SCALE).abs() / dist_delta
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let marker_color = grade_to_color(grade);

            // Draw a small diamond marker at the elevation sample point
            let size = 2.5;
            let up = Vec3::new(0.0, 0.0, size);
            let right = Vec3::new(size, 0.0, 0.0);
            gizmos.line(center - up, center + right, marker_color);
            gizmos.line(center + right, center + up, marker_color);
            gizmos.line(center + up, center - right, marker_color);
            gizmos.line(center - right, center - up, marker_color);

            // Draw a small vertical line to indicate height proportional to elevation
            let height_line_top = Vec3::new(pt.x, GIZMO_Y + 1.0 + display_elev * 0.1, pt.y);
            gizmos.line(
                Vec3::new(pt.x, GIZMO_Y, pt.y),
                height_line_top,
                marker_color,
            );

            prev_elevation = Some(elev);
            prev_world_dist = sample_cum_dist;
            label_index += 1;
            next_label_dist = label_index as f32 * label_step;
        }
    }
}

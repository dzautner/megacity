//! Road angle snapping: when Shift is held during freeform road drawing,
//! the end-point snaps to the nearest 15-degree increment from the start point.

use bevy::prelude::*;

use crate::input::{ActiveTool, CursorGridPos, DrawPhase, RoadDrawState};

/// Snap increment in degrees.
const SNAP_INCREMENT_DEG: f32 = 15.0;

/// Resource tracking the angle-snap state each frame.
#[derive(Resource, Default)]
pub struct AngleSnapState {
    /// Whether angle snapping is currently active (Shift held + road drawing).
    pub active: bool,
    /// The snapped world position (valid only when `active` is true).
    pub snapped_pos: Vec2,
    /// The snapped angle in degrees (0 = +X axis, counter-clockwise).
    pub snapped_angle_deg: f32,
}

/// Snap `angle_rad` to the nearest `SNAP_INCREMENT_DEG`-degree increment.
fn snap_angle(angle_rad: f32) -> f32 {
    let deg = angle_rad.to_degrees();
    let snapped_deg = (deg / SNAP_INCREMENT_DEG).round() * SNAP_INCREMENT_DEG;
    snapped_deg.to_radians()
}

/// System that computes the angle-snapped cursor position each frame.
/// Runs every frame; only activates when Shift is held during freeform road drawing.
pub fn update_angle_snap(
    keys: Res<ButtonInput<KeyCode>>,
    cursor: Res<CursorGridPos>,
    draw_state: Res<RoadDrawState>,
    tool: Res<ActiveTool>,
    mut snap: ResMut<AngleSnapState>,
) {
    let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    let is_road_tool = matches!(
        *tool,
        ActiveTool::Road
            | ActiveTool::RoadAvenue
            | ActiveTool::RoadBoulevard
            | ActiveTool::RoadHighway
            | ActiveTool::RoadOneWay
            | ActiveTool::RoadPath
    );

    // Only active when: shift held, road tool selected, start point placed, cursor valid
    if !shift_held || !is_road_tool || draw_state.phase != DrawPhase::PlacedStart || !cursor.valid {
        snap.active = false;
        return;
    }

    let start = draw_state.start_pos;
    let raw_end = cursor.world_pos;
    let delta = raw_end - start;
    let distance = delta.length();

    if distance < 0.001 {
        snap.active = false;
        return;
    }

    // Compute angle from start to cursor (atan2 gives angle from +X axis)
    let raw_angle = delta.y.atan2(delta.x);
    let snapped_angle = snap_angle(raw_angle);

    // Recompute end point at snapped angle, same distance
    let snapped_end = start + Vec2::new(snapped_angle.cos(), snapped_angle.sin()) * distance;

    snap.active = true;
    snap.snapped_pos = snapped_end;
    snap.snapped_angle_deg = snapped_angle.to_degrees();
}

/// Draw a visual indicator when angle snapping is active:
/// - An arc from the start point showing the snapped angle
/// - A label showing the angle in degrees
pub fn draw_angle_snap_indicator(
    snap: Res<AngleSnapState>,
    draw_state: Res<RoadDrawState>,
    mut gizmos: Gizmos,
) {
    if !snap.active || draw_state.phase != DrawPhase::PlacedStart {
        return;
    }

    let start = draw_state.start_pos;
    let end = snap.snapped_pos;
    let y = 0.8; // slightly above ground and road preview

    // Draw snapped guide line (dashed effect via segments)
    let guide_color = Color::srgba(0.3, 1.0, 0.6, 0.6);
    let delta = end - start;
    let distance = delta.length();
    let direction = delta / distance;

    // Draw dashed guide line extending beyond the cursor
    let dash_len = 8.0;
    let gap_len = 6.0;
    let total_len = distance + 80.0; // extend past cursor
    let mut t = 0.0;
    while t < total_len {
        let seg_start = t;
        let seg_end = (t + dash_len).min(total_len);
        let p0 = start + direction * seg_start;
        let p1 = start + direction * seg_end;
        gizmos.line(
            Vec3::new(p0.x, y, p0.y),
            Vec3::new(p1.x, y, p1.y),
            guide_color,
        );
        t += dash_len + gap_len;
    }

    // Draw a small arc near the start showing the snapped angle
    let arc_radius = 24.0_f32.min(distance * 0.3);
    let arc_color = Color::srgba(1.0, 1.0, 0.3, 0.8);
    let arc_segments = 16;
    let snapped_rad = snap.snapped_angle_deg.to_radians();

    // Draw arc from 0 degrees to the snapped angle
    let arc_start_angle = 0.0_f32;
    let arc_end_angle = snapped_rad;
    let (a0, a1) = if arc_start_angle <= arc_end_angle {
        (arc_start_angle, arc_end_angle)
    } else {
        (arc_end_angle, arc_start_angle)
    };

    // Only draw arc if angle is non-zero
    if (a1 - a0).abs() > 0.01 {
        let mut prev_pt = Vec3::new(
            start.x + arc_radius * a0.cos(),
            y,
            start.y + arc_radius * a0.sin(),
        );
        for i in 1..=arc_segments {
            let frac = i as f32 / arc_segments as f32;
            let angle = a0 + (a1 - a0) * frac;
            let pt = Vec3::new(
                start.x + arc_radius * angle.cos(),
                y,
                start.y + arc_radius * angle.sin(),
            );
            gizmos.line(prev_pt, pt, arc_color);
            prev_pt = pt;
        }
    }

    // Draw angle tick mark at the snapped position (small cross)
    let tick_size = 4.0;
    let end_3d = Vec3::new(end.x, y, end.y);
    gizmos.line(
        end_3d + Vec3::new(-tick_size, 0.0, -tick_size),
        end_3d + Vec3::new(tick_size, 0.0, tick_size),
        Color::srgba(1.0, 1.0, 0.3, 0.9),
    );
    gizmos.line(
        end_3d + Vec3::new(-tick_size, 0.0, tick_size),
        end_3d + Vec3::new(tick_size, 0.0, -tick_size),
        Color::srgba(1.0, 1.0, 0.3, 0.9),
    );
}

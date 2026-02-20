//! Road Parallel Snapping (UX-026)
//!
//! When drawing a road near an existing road, offers snap to maintain a
//! constant offset. Useful for one-way pairs or parallel service roads.
//!
//! The system detects nearby road segments, projects the cursor onto the
//! nearest segment, then offsets perpendicular by a configurable distance.
//! Visual guide lines show the parallel alignment using Bevy gizmos.
//!
//! Toggle with **P** key. Adjust offset with **[** / **]** keys.

use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::RoadType;
use simulation::road_segments::{RoadSegmentStore, SegmentId};

use crate::input::{ActiveTool, CursorGridPos, DrawPhase, RoadDrawState};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum distance (world units) from cursor to a segment for parallel snap
/// to activate.
const SNAP_DETECTION_RADIUS: f32 = 80.0;

/// Height above ground for gizmo rendering.
const GIZMO_Y: f32 = 0.6;

/// Number of line segments used to draw gizmo guide curves.
const GUIDE_SEGMENTS: usize = 48;

/// Default offset multiplier (how many combined half-widths apart).
const DEFAULT_OFFSET_MULTIPLIER: f32 = 2.5;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Configuration and live state for parallel road snapping.
#[derive(Resource)]
pub struct ParallelSnapState {
    /// Whether parallel snap is enabled (toggled with P key).
    pub enabled: bool,
    /// Offset multiplier: final offset = multiplier * (half_width_existing + half_width_new).
    pub offset_multiplier: f32,
    /// The snapped world position (if snap is active this frame).
    pub snapped_pos: Option<Vec2>,
    /// The reference segment used for snapping.
    pub ref_segment_id: Option<SegmentId>,
    /// The parameter t on the reference segment closest to cursor.
    pub ref_t: f32,
    /// Which side of the segment the cursor is on (+1 or -1).
    pub side: f32,
}

impl Default for ParallelSnapState {
    fn default() -> Self {
        Self {
            enabled: true,
            offset_multiplier: DEFAULT_OFFSET_MULTIPLIER,
            snapped_pos: None,
            ref_segment_id: None,
            ref_t: 0.0,
            side: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ParallelSnapPlugin;

impl Plugin for ParallelSnapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParallelSnapState>().add_systems(
            Update,
            (
                toggle_parallel_snap,
                adjust_parallel_offset,
                compute_parallel_snap.after(crate::input::update_cursor_grid_pos),
                apply_parallel_snap_to_cursor
                    .after(compute_parallel_snap)
                    .before(crate::input::handle_tool_input),
                draw_parallel_snap_guide,
            ),
        );
    }
}

// ---------------------------------------------------------------------------
// Helper: road visual half-width (matches road_render.rs values)
// ---------------------------------------------------------------------------

fn road_half_width(road_type: RoadType) -> f32 {
    match road_type {
        RoadType::Path => 1.5,
        RoadType::OneWay => 3.0,
        RoadType::Local => 4.0,
        RoadType::Avenue => 6.0,
        RoadType::Boulevard => 8.0,
        RoadType::Highway => 10.0,
    }
}

/// Map the active tool to a RoadType (if it is a road tool).
fn active_road_type(tool: &ActiveTool) -> Option<RoadType> {
    match tool {
        ActiveTool::Road => Some(RoadType::Local),
        ActiveTool::RoadAvenue => Some(RoadType::Avenue),
        ActiveTool::RoadBoulevard => Some(RoadType::Boulevard),
        ActiveTool::RoadHighway => Some(RoadType::Highway),
        ActiveTool::RoadOneWay => Some(RoadType::OneWay),
        ActiveTool::RoadPath => Some(RoadType::Path),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Toggle parallel snap with the P key.
pub fn toggle_parallel_snap(keys: Res<ButtonInput<KeyCode>>, mut snap: ResMut<ParallelSnapState>) {
    if keys.just_pressed(KeyCode::KeyP) {
        snap.enabled = !snap.enabled;
        if !snap.enabled {
            snap.snapped_pos = None;
            snap.ref_segment_id = None;
        }
    }
}

/// Adjust the parallel offset distance with [ and ] keys.
pub fn adjust_parallel_offset(
    keys: Res<ButtonInput<KeyCode>>,
    mut snap: ResMut<ParallelSnapState>,
) {
    if keys.just_pressed(KeyCode::BracketLeft) {
        snap.offset_multiplier = (snap.offset_multiplier - 0.5).max(1.0);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        snap.offset_multiplier = (snap.offset_multiplier + 0.5).min(6.0);
    }
}

/// Core parallel snap computation: find nearest segment, project cursor,
/// compute perpendicular offset, and store the snapped position.
pub fn compute_parallel_snap(
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    draw_state: Res<RoadDrawState>,
    segments: Res<RoadSegmentStore>,
    mut snap: ResMut<ParallelSnapState>,
) {
    // Clear previous snap result
    snap.snapped_pos = None;
    snap.ref_segment_id = None;

    if !snap.enabled || !cursor.valid {
        return;
    }

    // Only snap for road tools
    let new_road_type = match active_road_type(&tool) {
        Some(rt) => rt,
        None => return,
    };

    // Active during both Idle (placing start) and PlacedStart (placing end) phases
    if draw_state.phase != DrawPhase::PlacedStart && draw_state.phase != DrawPhase::Idle {
        return;
    }

    if segments.segments.is_empty() {
        return;
    }

    let cursor_pos = cursor.world_pos;

    // Find the nearest point across all segments
    let mut best_dist = SNAP_DETECTION_RADIUS;
    let mut best_seg_idx: Option<usize> = None;
    let mut best_t: f32 = 0.0;

    for (idx, segment) in segments.segments.iter().enumerate() {
        let samples = 32;
        for i in 0..=samples {
            let t = i as f32 / samples as f32;
            let pt = segment.evaluate(t);
            let dist = (pt - cursor_pos).length();
            if dist < best_dist {
                best_dist = dist;
                best_seg_idx = Some(idx);
                best_t = t;
            }
        }
    }

    let seg_idx = match best_seg_idx {
        Some(idx) => idx,
        None => return,
    };

    let segment = &segments.segments[seg_idx];

    // Refine t with Newton-like iterations for better precision
    let mut t = best_t;
    for _ in 0..4 {
        let pt = segment.evaluate(t);
        let tan = segment.tangent(t);
        let len_sq = tan.length_squared();
        if len_sq < 1e-8 {
            break;
        }
        let diff = cursor_pos - pt;
        let dt = diff.dot(tan) / len_sq;
        t = (t + dt).clamp(0.0, 1.0);
    }
    let closest_pt = segment.evaluate(t);

    // Compute perpendicular direction
    let tangent = segment.tangent(t);
    let tangent_norm = tangent.normalize_or_zero();
    if tangent_norm.length_squared() < 0.5 {
        return;
    }
    // Perpendicular: rotate 90 degrees CCW in 2D
    let perp = Vec2::new(-tangent_norm.y, tangent_norm.x);

    // Determine which side the cursor is on
    let to_cursor = cursor_pos - closest_pt;
    let side = if to_cursor.dot(perp) >= 0.0 {
        1.0
    } else {
        -1.0
    };

    // Compute offset distance
    let existing_hw = road_half_width(segment.road_type);
    let new_hw = road_half_width(new_road_type);
    let offset = snap.offset_multiplier * (existing_hw + new_hw);

    // Snapped position
    let snapped = closest_pt + perp * side * offset;

    // Only snap if cursor is close enough to the proposed snap line
    let snap_threshold = CELL_SIZE * 2.5;
    if (snapped - cursor_pos).length() < snap_threshold {
        snap.snapped_pos = Some(snapped);
        snap.ref_segment_id = Some(segment.id);
        snap.ref_t = t;
        snap.side = side;
    }
}

/// When parallel snap is active, override `CursorGridPos.world_pos` so that
/// `handle_tool_input` and `draw_bezier_preview` automatically use the
/// snapped position.
pub fn apply_parallel_snap_to_cursor(
    snap: Res<ParallelSnapState>,
    tool: Res<ActiveTool>,
    mut cursor: ResMut<CursorGridPos>,
) {
    if !snap.enabled {
        return;
    }

    // Only apply for road tools
    if active_road_type(&tool).is_none() {
        return;
    }

    if let Some(snapped) = snap.snapped_pos {
        cursor.world_pos = snapped;
    }
}

/// Draw gizmo guide lines and snap indicator when parallel snap is active.
pub fn draw_parallel_snap_guide(
    snap: Res<ParallelSnapState>,
    tool: Res<ActiveTool>,
    segments: Res<RoadSegmentStore>,
    mut gizmos: Gizmos,
) {
    if !snap.enabled {
        return;
    }

    let snapped = match snap.snapped_pos {
        Some(pos) => pos,
        None => return,
    };

    let seg_id = match snap.ref_segment_id {
        Some(id) => id,
        None => return,
    };

    let segment = match segments.get_segment(seg_id) {
        Some(s) => s,
        None => return,
    };

    let new_road_type = match active_road_type(&tool) {
        Some(rt) => rt,
        None => return,
    };

    let existing_hw = road_half_width(segment.road_type);
    let new_hw = road_half_width(new_road_type);
    let offset = snap.offset_multiplier * (existing_hw + new_hw);
    let side = snap.side;

    // Colors
    let guide_color = Color::srgba(0.2, 0.8, 1.0, 0.7);
    let ref_color = Color::srgba(0.2, 0.8, 1.0, 0.3);
    let snap_marker_color = Color::srgba(1.0, 0.9, 0.2, 0.9);
    let connector_color = Color::srgba(0.6, 0.6, 0.6, 0.5);

    // Draw the parallel guide line along the full segment length
    let mut prev_offset: Option<Vec3> = None;
    let mut prev_ref: Option<Vec3> = None;

    for i in 0..=GUIDE_SEGMENTS {
        let t = i as f32 / GUIDE_SEGMENTS as f32;
        let pt = segment.evaluate(t);
        let tan = segment.tangent(t).normalize_or_zero();
        let perp = Vec2::new(-tan.y, tan.x);

        let offset_pt = pt + perp * side * offset;
        let offset_3d = Vec3::new(offset_pt.x, GIZMO_Y, offset_pt.y);
        let ref_3d = Vec3::new(pt.x, GIZMO_Y, pt.y);

        if let Some(prev) = prev_offset {
            gizmos.line(prev, offset_3d, guide_color);
        }
        if let Some(prev) = prev_ref {
            gizmos.line(prev, ref_3d, ref_color);
        }

        prev_offset = Some(offset_3d);
        prev_ref = Some(ref_3d);
    }

    // Draw perpendicular connector from reference point to snap point
    let ref_pt = segment.evaluate(snap.ref_t);
    let ref_3d = Vec3::new(ref_pt.x, GIZMO_Y, ref_pt.y);
    let snap_3d = Vec3::new(snapped.x, GIZMO_Y, snapped.y);
    gizmos.line(ref_3d, snap_3d, connector_color);

    // Draw snap indicator circle
    gizmos.circle(
        Isometry3d::new(snap_3d, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        6.0,
        snap_marker_color,
    );

    // Draw diamond marker at snap position
    let diamond_size = 4.0;
    let tan_at_snap = segment.tangent(snap.ref_t).normalize_or_zero();
    let along = Vec3::new(tan_at_snap.x, 0.0, tan_at_snap.y) * diamond_size;
    let across_dir = Vec2::new(-tan_at_snap.y, tan_at_snap.x);
    let across = Vec3::new(across_dir.x, 0.0, across_dir.y) * diamond_size;

    let top = snap_3d + along;
    let bottom = snap_3d - along;
    let left = snap_3d - across;
    let right = snap_3d + across;

    gizmos.line(top, right, snap_marker_color);
    gizmos.line(right, bottom, snap_marker_color);
    gizmos.line(bottom, left, snap_marker_color);
    gizmos.line(left, top, snap_marker_color);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_road_half_width_values() {
        assert!((road_half_width(RoadType::Path) - 1.5).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::OneWay) - 3.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Local) - 4.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Avenue) - 6.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Boulevard) - 8.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Highway) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_active_road_type_mapping() {
        assert_eq!(active_road_type(&ActiveTool::Road), Some(RoadType::Local));
        assert_eq!(
            active_road_type(&ActiveTool::RoadAvenue),
            Some(RoadType::Avenue)
        );
        assert_eq!(
            active_road_type(&ActiveTool::RoadBoulevard),
            Some(RoadType::Boulevard)
        );
        assert_eq!(
            active_road_type(&ActiveTool::RoadHighway),
            Some(RoadType::Highway)
        );
        assert_eq!(
            active_road_type(&ActiveTool::RoadOneWay),
            Some(RoadType::OneWay)
        );
        assert_eq!(
            active_road_type(&ActiveTool::RoadPath),
            Some(RoadType::Path)
        );
        assert_eq!(active_road_type(&ActiveTool::Inspect), None);
        assert_eq!(active_road_type(&ActiveTool::Bulldoze), None);
    }

    #[test]
    fn test_default_snap_state() {
        let state = ParallelSnapState::default();
        assert!(state.enabled);
        assert!((state.offset_multiplier - DEFAULT_OFFSET_MULTIPLIER).abs() < f32::EPSILON);
        assert!(state.snapped_pos.is_none());
        assert!(state.ref_segment_id.is_none());
    }
}

//! Parallel Road Drawing Mode (UX-021)
//!
//! When enabled, drawing a road automatically creates a parallel road at a
//! fixed offset.  Useful for creating divided highways, one-way street pairs,
//! and dual carriageways.
//!
//! Toggle with **Alt+P**.  The offset is based on the road type's visual width.
//! Both roads are created as separate segments.
//!
//! A gizmo preview of the parallel road is drawn while placing the end point.

use bevy::prelude::*;

use simulation::app_state::AppState;
use simulation::config::CELL_SIZE;
use simulation::economy::CityBudget;
use simulation::grid::{RoadType, WorldGrid};
use simulation::road_segments::{RoadSegmentStore, SegmentId};
use simulation::roads::RoadNetwork;

use crate::cursor_preview::{bezier_eval, bezier_normal};
use crate::input::{ActiveTool, CursorGridPos, DrawPhase, IntersectionSnap, RoadDrawState};
use crate::terrain_render::{mark_chunk_dirty_at, ChunkDirty, TerrainChunk};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default offset multiplier: final offset = multiplier * 2 * half_width.
const DEFAULT_OFFSET_MULTIPLIER: f32 = 2.5;

/// Height above ground for gizmo rendering.
const GIZMO_Y: f32 = 0.5;

/// Number of segments for the gizmo preview curve.
const PREVIEW_SEGMENTS: usize = 48;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Configuration and state for parallel road drawing mode.
#[derive(Resource)]
pub struct ParallelDrawState {
    /// Whether parallel drawing is enabled (toggled with Alt+P).
    pub enabled: bool,
    /// Offset multiplier: final offset = multiplier * 2 * road_half_width.
    pub offset_multiplier: f32,
    /// Tracks the last segment count so we can detect newly placed segments.
    pub last_segment_count: usize,
    /// The last segment ID we created a parallel for, to avoid duplicates.
    pub last_parallel_source: Option<SegmentId>,
}

impl Default for ParallelDrawState {
    fn default() -> Self {
        Self {
            enabled: false,
            offset_multiplier: DEFAULT_OFFSET_MULTIPLIER,
            last_segment_count: 0,
            last_parallel_source: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ParallelDrawPlugin;

impl Plugin for ParallelDrawPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParallelDrawState>().add_systems(
            Update,
            (
                toggle_parallel_draw,
                create_parallel_segment.after(crate::input::handle_tool_input),
                draw_parallel_preview.after(crate::input::handle_tool_input),
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}

// ---------------------------------------------------------------------------
// Helper: road visual half-width
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

/// Compute the perpendicular offset for a straight road between two points.
/// Returns the offset vector (perpendicular to the road direction).
fn compute_offset_vector(start: Vec2, end: Vec2, road_type: RoadType, multiplier: f32) -> Vec2 {
    let dir = (end - start).normalize_or_zero();
    // Perpendicular: rotate 90 degrees CCW in 2D
    let perp = Vec2::new(-dir.y, dir.x);
    let half_w = road_half_width(road_type);
    perp * multiplier * 2.0 * half_w
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Toggle parallel drawing with Alt+P.
fn toggle_parallel_draw(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ParallelDrawState>,
    segments: Res<RoadSegmentStore>,
) {
    let alt_held = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    if alt_held && keys.just_pressed(KeyCode::KeyP) {
        state.enabled = !state.enabled;
        // Sync segment count to avoid creating parallels for existing segments
        state.last_segment_count = segments.segments.len();
        state.last_parallel_source = None;
    }
}

/// After a new road segment is placed, automatically create a parallel segment.
#[allow(clippy::too_many_arguments)]
fn create_parallel_segment(
    mut state: ResMut<ParallelDrawState>,
    mut segments: ResMut<RoadSegmentStore>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut budget: ResMut<CityBudget>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    mut commands: Commands,
) {
    if !state.enabled {
        state.last_segment_count = segments.segments.len();
        return;
    }

    let current_count = segments.segments.len();
    if current_count <= state.last_segment_count {
        state.last_segment_count = current_count;
        return;
    }

    // A new segment was added; get the most recently added one
    let new_segment = match segments.segments.last() {
        Some(s) => s,
        None => return,
    };

    // Avoid creating a parallel for our own parallel segments
    if Some(new_segment.id) == state.last_parallel_source {
        state.last_segment_count = segments.segments.len();
        return;
    }

    let source_id = new_segment.id;
    let road_type = new_segment.road_type;
    let start = new_segment.p0;
    let end = new_segment.p3;

    // Compute the perpendicular offset
    let offset = compute_offset_vector(start, end, road_type, state.offset_multiplier);

    let parallel_start = start + offset;
    let parallel_end = end + offset;

    // Check length
    if (parallel_end - parallel_start).length() < CELL_SIZE {
        state.last_segment_count = segments.segments.len();
        return;
    }

    // Check cost
    let approx_cells = ((parallel_end - parallel_start).length() / CELL_SIZE).ceil() as usize;
    let total_cost = road_type.cost() * approx_cells as f64;
    if budget.treasury < total_cost {
        // Not enough money for the parallel road — skip silently
        state.last_segment_count = segments.segments.len();
        return;
    }

    // Create the parallel segment
    let (_par_id, cells) = segments.add_straight_segment(
        parallel_start,
        parallel_end,
        road_type,
        24.0,
        &mut grid,
        &mut roads,
    );

    let actual_cost = road_type.cost() * cells.len() as f64;
    budget.treasury -= actual_cost;

    // Mark dirty chunks
    for &(cx, cy) in &cells {
        mark_chunk_dirty_at(cx, cy, &chunks, &mut commands);
    }

    // Track what we just created so we don't recurse
    state.last_parallel_source = Some(source_id);
    state.last_segment_count = segments.segments.len();
}

/// Draw a gizmo preview of the parallel road while in PlacedStart phase.
#[allow(clippy::too_many_arguments)]
fn draw_parallel_preview(
    state: Res<ParallelDrawState>,
    draw_state: Res<RoadDrawState>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    angle_snap: Res<crate::angle_snap::AngleSnapState>,
    snap: Res<IntersectionSnap>,
    mut gizmos: Gizmos,
) {
    if !state.enabled {
        return;
    }

    if draw_state.phase != DrawPhase::PlacedStart || !cursor.valid {
        return;
    }

    let road_type = match tool.road_type() {
        Some(rt) => rt,
        None => return,
    };

    let start = draw_state.start_pos;
    let end = if let Some(snapped) = snap.snapped_pos {
        snapped
    } else if angle_snap.active {
        angle_snap.snapped_pos
    } else {
        cursor.world_pos
    };

    if (end - start).length() < CELL_SIZE {
        return;
    }

    // Compute offset
    let offset = compute_offset_vector(start, end, road_type, state.offset_multiplier);
    let par_start = start + offset;
    let par_end = end + offset;

    // Bezier control points for the parallel road (straight line)
    let p0 = par_start;
    let p3 = par_end;
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    let half_w = road_half_width(road_type);
    let y = GIZMO_Y;

    // Colors — slightly different from the main preview to distinguish
    let center_color = Color::srgba(0.3, 1.0, 0.6, 0.4);
    let edge_color = Color::srgba(0.3, 1.0, 0.6, 0.7);
    let fill_color = Color::srgba(0.3, 0.7, 0.4, 0.15);

    // Sample curve points
    let mut centers: Vec<Vec3> = Vec::with_capacity(PREVIEW_SEGMENTS + 1);
    let mut lefts: Vec<Vec3> = Vec::with_capacity(PREVIEW_SEGMENTS + 1);
    let mut rights: Vec<Vec3> = Vec::with_capacity(PREVIEW_SEGMENTS + 1);

    for i in 0..=PREVIEW_SEGMENTS {
        let t = i as f32 / PREVIEW_SEGMENTS as f32;
        let pt = bezier_eval(p0, p1, p2, p3, t);
        let n = bezier_normal(p0, p1, p2, p3, t);
        let left = pt + n * half_w;
        let right = pt - n * half_w;
        centers.push(Vec3::new(pt.x, y, pt.y));
        lefts.push(Vec3::new(left.x, y, left.y));
        rights.push(Vec3::new(right.x, y, right.y));
    }

    // Draw center line
    for i in 0..PREVIEW_SEGMENTS {
        gizmos.line(centers[i], centers[i + 1], center_color);
    }

    // Draw left and right edge lines
    for i in 0..PREVIEW_SEGMENTS {
        gizmos.line(lefts[i], lefts[i + 1], edge_color);
        gizmos.line(rights[i], rights[i + 1], edge_color);
    }

    // Draw cross-hatching
    let fill_step = 4;
    for i in (0..=PREVIEW_SEGMENTS).step_by(fill_step) {
        gizmos.line(lefts[i], rights[i], fill_color);
    }

    // Draw start marker (green circle)
    let start_3d = Vec3::new(par_start.x, y, par_start.y);
    gizmos.circle(
        Isometry3d::new(start_3d, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        half_w,
        Color::srgba(0.2, 0.9, 0.4, 0.8),
    );

    // Draw end marker
    let end_3d = Vec3::new(par_end.x, y, par_end.y);
    gizmos.circle(
        Isometry3d::new(end_3d, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        half_w,
        Color::srgba(0.3, 1.0, 0.6, 0.8),
    );

    // Draw offset connector lines at start and end
    let connector_color = Color::srgba(0.6, 0.8, 0.6, 0.4);
    let main_start_3d = Vec3::new(start.x, y, start.y);
    let main_end_3d = Vec3::new(end.x, y, end.y);
    gizmos.line(main_start_3d, start_3d, connector_color);
    gizmos.line(main_end_3d, end_3d, connector_color);

    // Draw "parallel" indicator label position (small diamond at midpoint)
    let mid_main = (start + end) * 0.5;
    let mid_par = (par_start + par_end) * 0.5;
    let mid_connector_main = Vec3::new(mid_main.x, y + 0.1, mid_main.y);
    let mid_connector_par = Vec3::new(mid_par.x, y + 0.1, mid_par.y);
    gizmos.line(mid_connector_main, mid_connector_par, connector_color);
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
        assert!((road_half_width(RoadType::Local) - 4.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Avenue) - 6.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Boulevard) - 8.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Highway) - 10.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::OneWay) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_offset_vector_horizontal() {
        // Horizontal road: offset should be vertical
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(100.0, 0.0);
        let offset = compute_offset_vector(start, end, RoadType::Local, 2.5);
        // Perpendicular to (1,0) is (0,1), times 2.5 * 2 * 4.0 = 20.0
        assert!((offset.x).abs() < 1e-4);
        assert!((offset.y - 20.0).abs() < 1e-4);
    }

    #[test]
    fn test_compute_offset_vector_vertical() {
        // Vertical road: offset should be horizontal
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(0.0, 100.0);
        let offset = compute_offset_vector(start, end, RoadType::Local, 2.5);
        // Perpendicular to (0,1) is (-1,0), times 20.0
        assert!((offset.x - (-20.0)).abs() < 1e-4);
        assert!((offset.y).abs() < 1e-4);
    }

    #[test]
    fn test_compute_offset_vector_diagonal() {
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(100.0, 100.0);
        let offset = compute_offset_vector(start, end, RoadType::Local, 2.5);
        // Length should be 2.5 * 2 * 4.0 = 20.0
        assert!((offset.length() - 20.0).abs() < 1e-3);
    }

    #[test]
    fn test_compute_offset_vector_highway() {
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(100.0, 0.0);
        let offset = compute_offset_vector(start, end, RoadType::Highway, 2.5);
        // Highway half_width = 10.0, so offset = 2.5 * 2 * 10.0 = 50.0
        assert!((offset.y - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_default_state() {
        let state = ParallelDrawState::default();
        assert!(!state.enabled);
        assert!((state.offset_multiplier - DEFAULT_OFFSET_MULTIPLIER).abs() < f32::EPSILON);
        assert_eq!(state.last_segment_count, 0);
        assert!(state.last_parallel_source.is_none());
    }
}

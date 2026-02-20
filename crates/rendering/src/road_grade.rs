//! Road grade and elevation indicators shown during road preview.
//!
//! When the player is placing a road (freeform Bezier drawing, `DrawPhase::PlacedStart`),
//! this module overlays:
//!
//! - **Elevation numbers** at regular intervals along the preview curve
//! - **Grade color coding**: green (0-3%), yellow (3-6%), red (6%+)
//! - **Bridge indicator** where the road crosses water cells
//! - **Tunnel indicator** where the road goes through elevated terrain (hill)

use bevy::prelude::*;

use simulation::grid::{CellType, WorldGrid};

use crate::input::{ActiveTool, CursorGridPos, DrawPhase, RoadDrawState};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Distance between elevation sample points along the preview curve (world units).
const ELEVATION_SAMPLE_INTERVAL: f32 = 32.0;

/// Height offset for gizmo rendering above the ground plane.
const GIZMO_Y: f32 = 1.0;

/// Elevation scale factor: terrain elevation is [0,1], we scale for display.
/// In a 256x256 grid with CELL_SIZE=16, max world height is ~40m conceptually.
const ELEVATION_DISPLAY_SCALE: f32 = 40.0;

/// Grade thresholds (as fractions, not percentages).
const GRADE_LOW_THRESHOLD: f32 = 0.03;
const GRADE_MEDIUM_THRESHOLD: f32 = 0.06;

/// Radius of bridge/tunnel indicator circles.
const INDICATOR_RADIUS: f32 = 4.0;

/// Minimum hill elevation threshold for tunnel detection.
/// Cells with elevation above this are considered hills where tunnels would be needed.
const HILL_ELEVATION_THRESHOLD: f32 = 0.70;

// ---------------------------------------------------------------------------
// Grade colors
// ---------------------------------------------------------------------------

/// Green: gentle grade (0-3%).
const COLOR_GRADE_LOW: Color = Color::srgba(0.2, 0.85, 0.2, 0.9);

/// Yellow: moderate grade (3-6%).
const COLOR_GRADE_MEDIUM: Color = Color::srgba(0.9, 0.85, 0.1, 0.9);

/// Red: steep grade (6%+).
const COLOR_GRADE_HIGH: Color = Color::srgba(0.95, 0.15, 0.1, 0.9);

/// Blue: bridge indicator (water crossing).
const COLOR_BRIDGE: Color = Color::srgba(0.2, 0.5, 0.95, 0.9);

/// Orange: tunnel indicator (hill crossing).
const COLOR_TUNNEL: Color = Color::srgba(0.9, 0.5, 0.1, 0.9);

// ---------------------------------------------------------------------------
// Core system
// ---------------------------------------------------------------------------

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

    let mut fine_prev_pt = evaluate_bezier(p0, p1, p2, p3, 0.0);
    let mut fine_prev_elev = sample_elevation_at(&grid, fine_prev_pt);

    for i in 1..=fine_steps {
        let t = i as f32 / fine_steps as f32;
        let pt = evaluate_bezier(p0, p1, p2, p3, t);
        let seg_len = (pt - fine_prev_pt).length();

        let elev = sample_elevation_at(&grid, pt);

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
        let cell_type = sample_cell_type_at(&grid, pt);
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

    // Draw elevation markers at regular intervals along the curve
    let label_step = if num_samples > 1 {
        arc_length / (num_samples - 1) as f32
    } else {
        arc_length
    };
    let mut next_label_dist = 0.0_f32;
    let mut label_index = 0_usize;
    let mut prev_elevation: Option<f32> = None;
    let mut prev_world_dist: f32 = 0.0;

    let walk_steps = fine_steps;
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
            let elev = sample_elevation_at(&grid, pt);
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

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Check if the active tool is a road drawing tool.
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

/// Map a grade value to a color using the three-tier color coding.
///
/// - Green for 0-3% grade
/// - Yellow for 3-6% grade
/// - Red for 6%+ grade
///
/// Interpolates smoothly between tiers.
pub fn grade_to_color(grade: f32) -> Color {
    if grade <= GRADE_LOW_THRESHOLD {
        COLOR_GRADE_LOW
    } else if grade <= GRADE_MEDIUM_THRESHOLD {
        // Interpolate green -> yellow
        let t = (grade - GRADE_LOW_THRESHOLD) / (GRADE_MEDIUM_THRESHOLD - GRADE_LOW_THRESHOLD);
        lerp_color(COLOR_GRADE_LOW, COLOR_GRADE_MEDIUM, t)
    } else {
        // Interpolate yellow -> red (capped at 12%)
        let t = ((grade - GRADE_MEDIUM_THRESHOLD) / GRADE_MEDIUM_THRESHOLD).min(1.0);
        lerp_color(COLOR_GRADE_MEDIUM, COLOR_GRADE_HIGH, t)
    }
}

/// Linearly interpolate between two colors in sRGB space.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    let t = t.clamp(0.0, 1.0);
    Color::srgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    )
}

/// Sample the terrain elevation at a world position by looking up the grid cell.
fn sample_elevation_at(grid: &WorldGrid, world_pos: Vec2) -> f32 {
    let (gx, gy) = WorldGrid::world_to_grid(world_pos.x, world_pos.y);
    if gx >= 0 && gy >= 0 && grid.in_bounds(gx as usize, gy as usize) {
        grid.get(gx as usize, gy as usize).elevation
    } else {
        0.0
    }
}

/// Sample the cell type at a world position.
fn sample_cell_type_at(grid: &WorldGrid, world_pos: Vec2) -> CellType {
    let (gx, gy) = WorldGrid::world_to_grid(world_pos.x, world_pos.y);
    if gx >= 0 && gy >= 0 && grid.in_bounds(gx as usize, gy as usize) {
        grid.get(gx as usize, gy as usize).cell_type
    } else {
        CellType::Grass
    }
}

/// Evaluate a cubic Bezier curve at parameter t.
fn evaluate_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;
    p0 * mt3 + p1 * 3.0 * mt2 * t + p2 * 3.0 * mt * t2 + p3 * t3
}

/// Approximate arc length of a cubic Bezier by sampling.
fn approximate_arc_length(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    let steps = 64;
    let mut length = 0.0_f32;
    let mut prev = p0;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let pt = evaluate_bezier(p0, p1, p2, p3, t);
        length += (pt - prev).length();
        prev = pt;
    }
    length
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_to_color_low() {
        // 0% grade should be green
        let c = grade_to_color(0.0);
        let s = c.to_srgba();
        assert!(
            s.green > s.red,
            "0% grade should be green, got r={} g={}",
            s.red,
            s.green
        );
    }

    #[test]
    fn test_grade_to_color_medium() {
        // 4.5% grade should be between green and yellow (yellowish)
        let c = grade_to_color(0.045);
        let s = c.to_srgba();
        assert!(
            s.green > 0.5,
            "4.5% grade green should be > 0.5, got {}",
            s.green
        );
        assert!(s.red > 0.3, "4.5% grade red should be > 0.3, got {}", s.red);
    }

    #[test]
    fn test_grade_to_color_high() {
        // 10% grade should be red
        let c = grade_to_color(0.10);
        let s = c.to_srgba();
        assert!(
            s.red > s.green,
            "10% grade should be red, got r={} g={}",
            s.red,
            s.green
        );
    }

    #[test]
    fn test_grade_to_color_at_thresholds() {
        // At exactly 3% threshold, should be green
        let c_low = grade_to_color(GRADE_LOW_THRESHOLD);
        let s_low = c_low.to_srgba();
        assert!(
            s_low.green > s_low.red,
            "at 3% threshold should still be green"
        );

        // At exactly 6% threshold, should be yellow
        let c_med = grade_to_color(GRADE_MEDIUM_THRESHOLD);
        let s_med = c_med.to_srgba();
        assert!(s_med.red > 0.5, "at 6% threshold red should be > 0.5");
        assert!(s_med.green > 0.5, "at 6% threshold green should be > 0.5");
    }

    #[test]
    fn test_lerp_color_endpoints() {
        let a = Color::srgba(0.0, 0.0, 0.0, 1.0);
        let b = Color::srgba(1.0, 1.0, 1.0, 1.0);

        let start = lerp_color(a, b, 0.0).to_srgba();
        assert!((start.red - 0.0).abs() < 1e-5);

        let end = lerp_color(a, b, 1.0).to_srgba();
        assert!((end.red - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_lerp_color_midpoint() {
        let a = Color::srgba(0.0, 0.0, 0.0, 1.0);
        let b = Color::srgba(1.0, 1.0, 1.0, 1.0);
        let mid = lerp_color(a, b, 0.5).to_srgba();
        assert!((mid.red - 0.5).abs() < 1e-5);
        assert!((mid.green - 0.5).abs() < 1e-5);
        assert!((mid.blue - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_lerp_color_clamps() {
        let a = Color::srgba(0.2, 0.3, 0.4, 1.0);
        let b = Color::srgba(0.8, 0.7, 0.6, 1.0);

        let below = lerp_color(a, b, -1.0).to_srgba();
        let at_zero = lerp_color(a, b, 0.0).to_srgba();
        assert!((below.red - at_zero.red).abs() < 1e-5);

        let above = lerp_color(a, b, 2.0).to_srgba();
        let at_one = lerp_color(a, b, 1.0).to_srgba();
        assert!((above.red - at_one.red).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_bezier_endpoints() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(10.0, 0.0);
        let p2 = Vec2::new(20.0, 0.0);
        let p3 = Vec2::new(30.0, 0.0);

        let start = evaluate_bezier(p0, p1, p2, p3, 0.0);
        assert!((start - p0).length() < 1e-5);

        let end = evaluate_bezier(p0, p1, p2, p3, 1.0);
        assert!((end - p3).length() < 1e-5);
    }

    #[test]
    fn test_approximate_arc_length_straight_line() {
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(100.0, 0.0);
        let p1 = Vec2::new(33.33, 0.0);
        let p2 = Vec2::new(66.67, 0.0);

        let length = approximate_arc_length(p0, p1, p2, p3);
        assert!(
            (length - 100.0).abs() < 1.0,
            "straight line arc length should be ~100, got {length}"
        );
    }

    #[test]
    fn test_approximate_arc_length_zero() {
        let p = Vec2::new(50.0, 50.0);
        let length = approximate_arc_length(p, p, p, p);
        assert!(
            length < 0.01,
            "zero-length curve should have ~0 arc length, got {length}"
        );
    }

    #[test]
    fn test_sample_elevation_at_in_bounds() {
        let mut grid = WorldGrid::new(16, 16);
        grid.get_mut(5, 5).elevation = 0.75;
        let (wx, wy) = WorldGrid::grid_to_world(5, 5);
        let elev = sample_elevation_at(&grid, Vec2::new(wx, wy));
        assert!(
            (elev - 0.75).abs() < 1e-5,
            "should sample correct elevation, got {elev}"
        );
    }

    #[test]
    fn test_sample_elevation_at_out_of_bounds() {
        let grid = WorldGrid::new(16, 16);
        let elev = sample_elevation_at(&grid, Vec2::new(-100.0, -100.0));
        assert!(
            (elev - 0.0).abs() < 1e-5,
            "out-of-bounds should return 0.0, got {elev}"
        );
    }

    #[test]
    fn test_sample_cell_type_at_water() {
        let mut grid = WorldGrid::new(16, 16);
        grid.get_mut(3, 7).cell_type = CellType::Water;
        let (wx, wy) = WorldGrid::grid_to_world(3, 7);
        let ct = sample_cell_type_at(&grid, Vec2::new(wx, wy));
        assert_eq!(ct, CellType::Water);
    }

    #[test]
    fn test_sample_cell_type_at_out_of_bounds() {
        let grid = WorldGrid::new(16, 16);
        let ct = sample_cell_type_at(&grid, Vec2::new(-50.0, -50.0));
        assert_eq!(ct, CellType::Grass, "out-of-bounds should default to Grass");
    }

    #[test]
    fn test_is_road_tool() {
        assert!(is_road_tool(&ActiveTool::Road));
        assert!(is_road_tool(&ActiveTool::RoadAvenue));
        assert!(is_road_tool(&ActiveTool::RoadBoulevard));
        assert!(is_road_tool(&ActiveTool::RoadHighway));
        assert!(is_road_tool(&ActiveTool::RoadOneWay));
        assert!(is_road_tool(&ActiveTool::RoadPath));
        assert!(!is_road_tool(&ActiveTool::Bulldoze));
        assert!(!is_road_tool(&ActiveTool::Inspect));
    }

    #[test]
    fn test_grade_color_monotonic_red() {
        let low = grade_to_color(0.01).to_srgba();
        let high = grade_to_color(0.12).to_srgba();
        assert!(
            high.red > low.red,
            "higher grade should have more red: low.r={} high.r={}",
            low.red,
            high.red
        );
    }

    #[test]
    fn test_grade_color_low_is_green_dominant() {
        let c = grade_to_color(0.01).to_srgba();
        assert!(
            c.green > c.red && c.green > c.blue,
            "low grade should be green dominant"
        );
    }
}

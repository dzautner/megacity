//! Helper functions for road grade computation: Bezier evaluation, grid sampling,
//! color interpolation, and tool identification.

use bevy::prelude::*;

use simulation::grid::{CellType, WorldGrid};

use crate::input::ActiveTool;

use super::constants::{
    COLOR_GRADE_HIGH, COLOR_GRADE_LOW, COLOR_GRADE_MEDIUM, GRADE_LOW_THRESHOLD,
    GRADE_MEDIUM_THRESHOLD,
};

// ---------------------------------------------------------------------------
// Tool identification
// ---------------------------------------------------------------------------

/// Check if the active tool is a road drawing tool.
pub(crate) fn is_road_tool(tool: &ActiveTool) -> bool {
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
// Color helpers
// ---------------------------------------------------------------------------

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
pub(crate) fn lerp_color(a: Color, b: Color, t: f32) -> Color {
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

// ---------------------------------------------------------------------------
// Grid sampling
// ---------------------------------------------------------------------------

/// Sample the terrain elevation at a world position by looking up the grid cell.
pub(crate) fn sample_elevation_at(grid: &WorldGrid, world_pos: Vec2) -> f32 {
    let (gx, gy) = WorldGrid::world_to_grid(world_pos.x, world_pos.y);
    if gx >= 0 && gy >= 0 && grid.in_bounds(gx as usize, gy as usize) {
        grid.get(gx as usize, gy as usize).elevation
    } else {
        0.0
    }
}

/// Sample the cell type at a world position.
pub(crate) fn sample_cell_type_at(grid: &WorldGrid, world_pos: Vec2) -> CellType {
    let (gx, gy) = WorldGrid::world_to_grid(world_pos.x, world_pos.y);
    if gx >= 0 && gy >= 0 && grid.in_bounds(gx as usize, gy as usize) {
        grid.get(gx as usize, gy as usize).cell_type
    } else {
        CellType::Grass
    }
}

// ---------------------------------------------------------------------------
// Bezier helpers
// ---------------------------------------------------------------------------

/// Evaluate a cubic Bezier curve at parameter t.
pub(crate) fn evaluate_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;
    p0 * mt3 + p1 * 3.0 * mt2 * t + p2 * 3.0 * mt * t2 + p3 * t3
}

/// Approximate arc length of a cubic Bezier by sampling.
pub(crate) fn approximate_arc_length(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
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

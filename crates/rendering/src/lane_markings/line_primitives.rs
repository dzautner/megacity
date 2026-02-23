//! Low-level line-strip geometry emitters along cubic Bezier curves.
//!
//! Provides [`emit_dashed_line`] and [`emit_solid_line`] which append
//! triangle-list vertices/indices into caller-provided buffers, plus the
//! Bezier evaluation helpers they rely on.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Bezier helpers
// ---------------------------------------------------------------------------

/// Evaluate a cubic Bezier at parameter `t` in [0, 1].
pub(crate) fn eval_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;
    p0 * mt3 + p1 * 3.0 * mt2 * t + p2 * 3.0 * mt * t2 + p3 * t3
}

/// Tangent of a cubic Bezier at parameter `t`.
pub(crate) fn bezier_tangent(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;
    (p1 - p0) * 3.0 * mt2 + (p2 - p1) * 6.0 * mt * t + (p3 - p2) * 3.0 * t2
}

/// Get the road surface half-width for a given road type.
pub(crate) fn road_half_width(road_type: simulation::grid::RoadType) -> f32 {
    match road_type {
        simulation::grid::RoadType::Path => 1.5,
        simulation::grid::RoadType::OneWay => 3.0,
        simulation::grid::RoadType::Local => 4.0,
        simulation::grid::RoadType::Avenue => 6.0,
        simulation::grid::RoadType::Boulevard => 8.0,
        simulation::grid::RoadType::Highway => 10.0,
    }
}

// ---------------------------------------------------------------------------
// Line primitives along Bezier curves
// ---------------------------------------------------------------------------

/// Emit a **dashed** line strip along the Bezier at a perpendicular offset.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_dashed_line(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    perp_offset: f32,
    half_width: f32,
    y: f32,
    arc_length: f32,
    step: f32,
    total_steps: usize,
    dash_len: f32,
    gap_len: f32,
    color: [f32; 4],
    trim_start: f32,
    trim_end: f32,
) {
    let period = dash_len + gap_len;
    let mut cum_len = 0.0_f32;
    let mut prev_pt = eval_bezier(p0, p1, p2, p3, 0.0);
    let mut in_dash = false;
    let mut dash_start: Option<(Vec2, Vec2)> = None;
    let trim_end_dist = arc_length - trim_end;

    for i in 0..=total_steps {
        let t = (i as f32 * step / arc_length).min(1.0);
        let pt = eval_bezier(p0, p1, p2, p3, t);
        let tangent = bezier_tangent(p0, p1, p2, p3, t).normalize_or_zero();
        let perp = Vec2::new(-tangent.y, tangent.x);

        if i > 0 {
            cum_len += (pt - prev_pt).length();
        }
        prev_pt = pt;

        let in_trim = cum_len < trim_start || cum_len > trim_end_dist;
        let phase = cum_len % period;
        let should_dash = phase < dash_len && !in_trim;

        let center = pt + perp * perp_offset;
        let left = center - perp * half_width;
        let right = center + perp * half_width;

        if should_dash && !in_dash {
            dash_start = Some((left, right));
            in_dash = true;
        } else if in_dash && (!should_dash || i == total_steps) {
            if let Some((sl, sr)) = dash_start {
                let vi = positions.len() as u32;
                positions.push([sl.x, y, sl.y]);
                positions.push([sr.x, y, sr.y]);
                positions.push([right.x, y, right.y]);
                positions.push([left.x, y, left.y]);
                normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
                colors.extend_from_slice(&[color; 4]);
                uvs.extend_from_slice(&[[0.0, 0.0]; 4]);
                indices.push(vi);
                indices.push(vi + 2);
                indices.push(vi + 1);
                indices.push(vi);
                indices.push(vi + 3);
                indices.push(vi + 2);
            }
            in_dash = false;
            dash_start = None;
        }
    }
}

/// Emit a **solid** (continuous) line strip along the Bezier at a perpendicular offset.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_solid_line(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    perp_offset: f32,
    half_width: f32,
    y: f32,
    arc_length: f32,
    total_steps: usize,
    color: [f32; 4],
    trim_start: f32,
    trim_end: f32,
) {
    let segments = total_steps.min(64);
    let trim_end_dist = arc_length - trim_end;
    let mut cum_len = 0.0_f32;
    let mut prev_pt = eval_bezier(p0, p1, p2, p3, 0.0);
    let mut prev_emitted = false;

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let pt = eval_bezier(p0, p1, p2, p3, t);
        let tangent = bezier_tangent(p0, p1, p2, p3, t).normalize_or_zero();
        let perp = Vec2::new(-tangent.y, tangent.x);

        if i > 0 {
            cum_len += (pt - prev_pt).length();
        }
        prev_pt = pt;

        if cum_len < trim_start || cum_len > trim_end_dist {
            prev_emitted = false;
            continue;
        }

        let center = pt + perp * perp_offset;
        let left = center - perp * half_width;
        let right = center + perp * half_width;

        positions.push([left.x, y, left.y]);
        positions.push([right.x, y, right.y]);
        normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 2]);
        colors.extend_from_slice(&[color; 2]);
        uvs.extend_from_slice(&[[0.0, t]; 2]);

        if prev_emitted {
            let base = (positions.len() as u32) - 4;
            indices.push(base);
            indices.push(base + 2);
            indices.push(base + 1);
            indices.push(base + 1);
            indices.push(base + 2);
            indices.push(base + 3);
        }
        prev_emitted = true;
    }
}

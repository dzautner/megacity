use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::{RoadType, WorldGrid};

use super::bezier::{bezier_tangent, evaluate_bezier};

/// Sample terrain elevation at a world XZ position, returning a Y value.
fn sample_terrain_y(grid: &WorldGrid, world_x: f32, world_z: f32) -> f32 {
    let gx = (world_x / CELL_SIZE).floor() as i32;
    let gy = (world_z / CELL_SIZE).floor() as i32;
    let gx = (gx as usize).min(grid.width.saturating_sub(1));
    let gy = (gy as usize).min(grid.height.saturating_sub(1));
    grid.elevation_y(gx, gy)
}

/// Add dashed center line and lane markings along the curve.
/// `y_offset` is a small offset above terrain for z-fighting avoidance.
#[allow(clippy::too_many_arguments)]
pub fn add_curve_lane_markings(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    road_type: RoadType,
    road_half_w: f32,
    arc_length: f32,
    y_offset: f32,
    _sample_count: usize,
    trim_start: f32,
    trim_end: f32,
    grid: &WorldGrid,
) {
    let line_hw = 0.12; // line half-width
    let dash_len = 3.0;
    let gap_len = 4.0;
    let white: [f32; 4] = [1.0, 1.0, 1.0, 0.45];
    let yellow: [f32; 4] = [0.80, 0.70, 0.12, 0.65];

    let step = 0.5_f32;
    let total_steps = (arc_length / step).ceil() as usize;

    match road_type {
        RoadType::Local | RoadType::OneWay => {
            generate_dashed_line_along_curve(
                positions,
                normals,
                colors,
                uvs,
                indices,
                p0,
                p1,
                p2,
                p3,
                0.0,
                line_hw,
                y_offset,
                arc_length,
                step,
                total_steps,
                dash_len,
                gap_len,
                white,
                trim_start,
                trim_end,
                grid,
            );
        }
        RoadType::Avenue => {
            let offset = 0.3;
            generate_solid_line_along_curve(
                positions,
                normals,
                colors,
                uvs,
                indices,
                p0,
                p1,
                p2,
                p3,
                -offset,
                line_hw,
                y_offset,
                arc_length,
                step,
                total_steps,
                yellow,
                trim_start,
                trim_end,
                grid,
            );
            generate_solid_line_along_curve(
                positions,
                normals,
                colors,
                uvs,
                indices,
                p0,
                p1,
                p2,
                p3,
                offset,
                line_hw,
                y_offset,
                arc_length,
                step,
                total_steps,
                yellow,
                trim_start,
                trim_end,
                grid,
            );
        }
        RoadType::Boulevard => {
            let median_hw = 1.5;
            generate_solid_line_along_curve(
                positions,
                normals,
                colors,
                uvs,
                indices,
                p0,
                p1,
                p2,
                p3,
                0.0,
                median_hw,
                y_offset - 0.01,
                arc_length,
                step,
                total_steps,
                [0.30, 0.42, 0.25, 1.0],
                trim_start,
                trim_end,
                grid,
            );
            let lane_w = road_half_w * 0.45;
            for &off in &[-lane_w, lane_w] {
                generate_dashed_line_along_curve(
                    positions,
                    normals,
                    colors,
                    uvs,
                    indices,
                    p0,
                    p1,
                    p2,
                    p3,
                    off,
                    line_hw,
                    y_offset,
                    arc_length,
                    step,
                    total_steps,
                    dash_len,
                    gap_len,
                    [1.0, 1.0, 1.0, 0.35],
                    trim_start,
                    trim_end,
                    grid,
                );
            }
        }
        RoadType::Highway => {
            generate_solid_line_along_curve(
                positions,
                normals,
                colors,
                uvs,
                indices,
                p0,
                p1,
                p2,
                p3,
                0.0,
                0.4,
                y_offset + 0.02,
                arc_length,
                step,
                total_steps,
                [0.55, 0.53, 0.50, 1.0],
                trim_start,
                trim_end,
                grid,
            );
            let lane_w = road_half_w * 0.45;
            for &off in &[-lane_w, lane_w] {
                generate_dashed_line_along_curve(
                    positions,
                    normals,
                    colors,
                    uvs,
                    indices,
                    p0,
                    p1,
                    p2,
                    p3,
                    off,
                    line_hw * 1.5,
                    y_offset,
                    arc_length,
                    step,
                    total_steps,
                    dash_len,
                    gap_len,
                    [1.0, 1.0, 1.0, 0.55],
                    trim_start,
                    trim_end,
                    grid,
                );
            }
            for &edge in &[-road_half_w + 0.5, road_half_w - 0.5] {
                generate_solid_line_along_curve(
                    positions,
                    normals,
                    colors,
                    uvs,
                    indices,
                    p0,
                    p1,
                    p2,
                    p3,
                    edge,
                    line_hw * 1.5,
                    y_offset,
                    arc_length,
                    step,
                    total_steps,
                    [1.0, 1.0, 1.0, 0.65],
                    trim_start,
                    trim_end,
                    grid,
                );
            }
        }
        RoadType::Path => {}
    }
}

/// Generate a dashed line along a Bezier curve at a perpendicular offset.
/// `y_offset` is added to the sampled terrain Y for each quad.
#[allow(clippy::too_many_arguments)]
fn generate_dashed_line_along_curve(
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
    y_offset: f32,
    arc_length: f32,
    step: f32,
    total_steps: usize,
    dash_len: f32,
    gap_len: f32,
    color: [f32; 4],
    trim_start: f32,
    trim_end: f32,
    grid: &WorldGrid,
) {
    let period = dash_len + gap_len;
    let mut cum_len = 0.0_f32;
    let mut prev_pt = evaluate_bezier(p0, p1, p2, p3, 0.0);
    let mut in_dash = false;
    let mut dash_start_pts: Option<(Vec2, Vec2, f32)> = None;
    let trim_end_dist = arc_length - trim_end;

    for i in 0..=total_steps {
        let t = (i as f32 * step / arc_length).min(1.0);
        let pt = evaluate_bezier(p0, p1, p2, p3, t);
        let tangent = bezier_tangent(p0, p1, p2, p3, t).normalize_or_zero();
        let perp = Vec2::new(-tangent.y, tangent.x);

        if i > 0 {
            cum_len += (pt - prev_pt).length();
        }
        prev_pt = pt;

        let in_trim_zone = cum_len < trim_start || cum_len > trim_end_dist;

        let phase = cum_len % period;
        let should_be_dash = phase < dash_len && !in_trim_zone;

        let center = pt + perp * perp_offset;
        let left = center - perp * half_width;
        let right = center + perp * half_width;
        let y = sample_terrain_y(grid, pt.x, pt.y) + y_offset;

        if should_be_dash && !in_dash {
            dash_start_pts = Some((left, right, y));
            in_dash = true;
        } else if in_dash && (!should_be_dash || i == total_steps) {
            if let Some((sl, sr, sy)) = dash_start_pts {
                let vi = positions.len() as u32;
                let avg_y = (sy + y) * 0.5;
                positions.push([sl.x, avg_y, sl.y]);
                positions.push([sr.x, avg_y, sr.y]);
                positions.push([right.x, avg_y, right.y]);
                positions.push([left.x, avg_y, left.y]);
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
            dash_start_pts = None;
        }
    }
}

/// Generate a solid continuous line along a Bezier curve at a perpendicular offset.
/// `y_offset` is added to the sampled terrain Y for each vertex.
#[allow(clippy::too_many_arguments)]
fn generate_solid_line_along_curve(
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
    y_offset: f32,
    arc_length: f32,
    _step: f32,
    total_steps: usize,
    color: [f32; 4],
    trim_start: f32,
    trim_end: f32,
    grid: &WorldGrid,
) {
    let segments = total_steps.min(64);
    let trim_end_dist = arc_length - trim_end;
    let mut cum_len = 0.0_f32;
    let mut prev_pt = evaluate_bezier(p0, p1, p2, p3, 0.0);
    let mut prev_emitted = false;

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let pt = evaluate_bezier(p0, p1, p2, p3, t);
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
        let y = sample_terrain_y(grid, pt.x, pt.y) + y_offset;

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

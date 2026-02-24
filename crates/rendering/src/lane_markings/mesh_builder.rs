//! Per-segment lane-marking mesh construction.
//!
//! [`build_lane_marking_mesh`] produces the triangle-list [`Mesh`] containing
//! center lines, lane dividers, edge lines and barriers appropriate for the
//! given [`RoadType`].

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;

use simulation::grid::RoadType;

use super::line_primitives::{emit_dashed_line, emit_solid_line};
use super::{BARRIER_COLOR, DASH_LEN, GAP_LEN, LINE_HW, WHITE, YELLOW, Y_MARKING};

/// Build all lane-marking geometry for one road segment.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_lane_marking_mesh(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    road_type: RoadType,
    road_half_w: f32,
    arc_length: f32,
    trim_start: f32,
    trim_end: f32,
) -> Mesh {
    let step = 0.5_f32;
    let total_steps = (arc_length / step).ceil() as usize;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    match road_type {
        RoadType::Local | RoadType::OneWay => {
            // Dashed white center line for two-lane local/one-way roads.
            emit_dashed_line(
                &mut positions,
                &mut normals,
                &mut colors,
                &mut uvs,
                &mut indices,
                p0,
                p1,
                p2,
                p3,
                0.0,
                LINE_HW,
                Y_MARKING,
                arc_length,
                step,
                total_steps,
                DASH_LEN,
                GAP_LEN,
                WHITE,
                trim_start,
                trim_end,
            );
        }
        RoadType::Avenue => {
            // Double yellow center lines (solid).
            let offset = 0.3;
            emit_solid_line(
                &mut positions,
                &mut normals,
                &mut colors,
                &mut uvs,
                &mut indices,
                p0,
                p1,
                p2,
                p3,
                -offset,
                LINE_HW,
                Y_MARKING,
                arc_length,
                total_steps,
                YELLOW,
                trim_start,
                trim_end,
            );
            emit_solid_line(
                &mut positions,
                &mut normals,
                &mut colors,
                &mut uvs,
                &mut indices,
                p0,
                p1,
                p2,
                p3,
                offset,
                LINE_HW,
                Y_MARKING,
                arc_length,
                total_steps,
                YELLOW,
                trim_start,
                trim_end,
            );
            // Solid white edge lines.
            for &edge in &[-(road_half_w - 0.5), road_half_w - 0.5] {
                emit_solid_line(
                    &mut positions,
                    &mut normals,
                    &mut colors,
                    &mut uvs,
                    &mut indices,
                    p0,
                    p1,
                    p2,
                    p3,
                    edge,
                    LINE_HW,
                    Y_MARKING,
                    arc_length,
                    total_steps,
                    [1.0, 1.0, 1.0, 0.45],
                    trim_start,
                    trim_end,
                );
            }
        }
        RoadType::Boulevard => {
            // Solid yellow center line.
            emit_solid_line(
                &mut positions,
                &mut normals,
                &mut colors,
                &mut uvs,
                &mut indices,
                p0,
                p1,
                p2,
                p3,
                0.0,
                LINE_HW * 1.5,
                Y_MARKING,
                arc_length,
                total_steps,
                YELLOW,
                trim_start,
                trim_end,
            );
            // Dashed white lane dividers at inner lane boundaries.
            let lane_offset = road_half_w * 0.45;
            for &off in &[-lane_offset, lane_offset] {
                emit_dashed_line(
                    &mut positions,
                    &mut normals,
                    &mut colors,
                    &mut uvs,
                    &mut indices,
                    p0,
                    p1,
                    p2,
                    p3,
                    off,
                    LINE_HW,
                    Y_MARKING,
                    arc_length,
                    step,
                    total_steps,
                    DASH_LEN,
                    GAP_LEN,
                    [1.0, 1.0, 1.0, 0.45],
                    trim_start,
                    trim_end,
                );
            }
            // Solid white edge lines.
            for &edge in &[-(road_half_w - 0.5), road_half_w - 0.5] {
                emit_solid_line(
                    &mut positions,
                    &mut normals,
                    &mut colors,
                    &mut uvs,
                    &mut indices,
                    p0,
                    p1,
                    p2,
                    p3,
                    edge,
                    LINE_HW,
                    Y_MARKING,
                    arc_length,
                    total_steps,
                    [1.0, 1.0, 1.0, 0.45],
                    trim_start,
                    trim_end,
                );
            }
        }
        RoadType::Highway => {
            // Concrete jersey barrier in centre.
            emit_solid_line(
                &mut positions,
                &mut normals,
                &mut colors,
                &mut uvs,
                &mut indices,
                p0,
                p1,
                p2,
                p3,
                0.0,
                0.4,
                Y_MARKING + 0.02,
                arc_length,
                total_steps,
                BARRIER_COLOR,
                trim_start,
                trim_end,
            );
            // Dashed white lane dividers.
            let lane_offset = road_half_w * 0.45;
            for &off in &[-lane_offset, lane_offset] {
                emit_dashed_line(
                    &mut positions,
                    &mut normals,
                    &mut colors,
                    &mut uvs,
                    &mut indices,
                    p0,
                    p1,
                    p2,
                    p3,
                    off,
                    LINE_HW * 1.5,
                    Y_MARKING,
                    arc_length,
                    step,
                    total_steps,
                    DASH_LEN,
                    GAP_LEN,
                    WHITE,
                    trim_start,
                    trim_end,
                );
            }
            // Solid white edge lines.
            for &edge in &[-(road_half_w - 0.5), road_half_w - 0.5] {
                emit_solid_line(
                    &mut positions,
                    &mut normals,
                    &mut colors,
                    &mut uvs,
                    &mut indices,
                    p0,
                    p1,
                    p2,
                    p3,
                    edge,
                    LINE_HW * 1.5,
                    Y_MARKING,
                    arc_length,
                    total_steps,
                    [1.0, 1.0, 1.0, 0.65],
                    trim_start,
                    trim_end,
                );
            }
        }
        // Path: no markings (filtered earlier).
        _ => {}
    }

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

use bevy::prelude::*;
use bevy::render::mesh::Indices;

use simulation::colorblind::ColorblindMode;
use simulation::config::{CELL_SIZE, CHUNK_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{CellType, WorldGrid};
use simulation::network_viz::NetworkVizData;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::weather::Season;

use crate::overlay::OverlayMode;

use super::coloring::{apply_overlay, blend_dual_overlays, coast_tint, terrain_color};
use super::road_markings::add_road_markings;
use super::types::{DualOverlayInfo, OverlayGrids};

pub(super) fn chunk_world_pos(cx: usize, cy: usize) -> (f32, f32) {
    let wx = cx as f32 * CHUNK_SIZE as f32 * CELL_SIZE;
    let wz = cy as f32 * CHUNK_SIZE as f32 * CELL_SIZE;
    (wx, wz)
}

#[allow(clippy::too_many_arguments)]
pub fn build_chunk_mesh(
    grid: &WorldGrid,
    roads: &RoadNetwork,
    _segments: &RoadSegmentStore,
    cx: usize,
    cy: usize,
    overlay: &OverlayMode,
    overlay_grids: &OverlayGrids,
    season: Season,
    cb_mode: ColorblindMode,
    network_viz: &NetworkVizData,
    dual: &DualOverlayInfo,
) -> Mesh {
    let cells_in_chunk = CHUNK_SIZE * CHUNK_SIZE;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(cells_in_chunk * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(cells_in_chunk * 4);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(cells_in_chunk * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(cells_in_chunk * 6);

    let base_gx = cx * CHUNK_SIZE;
    let base_gy = cy * CHUNK_SIZE;

    for ly in 0..CHUNK_SIZE {
        for lx in 0..CHUNK_SIZE {
            let gx = base_gx + lx;
            let gy = base_gy + ly;

            if gx >= GRID_WIDTH || gy >= GRID_HEIGHT {
                continue;
            }

            let cell = grid.get(gx, gy);
            let snow_depth = overlay_grids.snow.map(|sg| sg.get(gx, gy)).unwrap_or(0.0);
            let base_color = terrain_color(cell, gx, gy, season, snow_depth, cb_mode);
            let color = if dual.is_active(overlay) {
                let primary_color = apply_overlay(
                    base_color,
                    cell,
                    gx,
                    gy,
                    grid,
                    overlay,
                    overlay_grids,
                    cb_mode,
                    network_viz,
                );
                let secondary_color = apply_overlay(
                    base_color,
                    cell,
                    gx,
                    gy,
                    grid,
                    &dual.secondary,
                    overlay_grids,
                    cb_mode,
                    network_viz,
                );
                blend_dual_overlays(
                    primary_color,
                    secondary_color,
                    gx,
                    &dual.mode,
                    dual.blend_factor,
                )
            } else {
                apply_overlay(
                    base_color,
                    cell,
                    gx,
                    gy,
                    grid,
                    overlay,
                    overlay_grids,
                    cb_mode,
                    network_viz,
                )
            };

            let x0 = lx as f32 * CELL_SIZE;
            let z0 = ly as f32 * CELL_SIZE;
            let x1 = (lx + 1) as f32 * CELL_SIZE;
            let z1 = (ly + 1) as f32 * CELL_SIZE;
            let y = 0.0;

            let c: [f32; 4] = color.to_srgba().to_f32_array();

            // Cheap coastline blending: tint cells adjacent to water
            let c = if cell.cell_type != CellType::Road {
                coast_tint(grid, gx, gy, c, cell.cell_type)
            } else {
                c
            };

            // 4 vertices, 2 triangles per cell
            let vi = positions.len() as u32;
            positions.push([x0, y, z0]);
            positions.push([x1, y, z0]);
            positions.push([x1, y, z1]);
            positions.push([x0, y, z1]);
            normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
            colors.extend_from_slice(&[c; 4]);

            indices.push(vi);
            indices.push(vi + 2);
            indices.push(vi + 1);
            indices.push(vi);
            indices.push(vi + 3);
            indices.push(vi + 2);

            // Road surface and markings
            if cell.cell_type == CellType::Road && *overlay == OverlayMode::None {
                add_road_markings(
                    &mut positions,
                    &mut normals,
                    &mut colors,
                    &mut indices,
                    grid,
                    roads,
                    gx,
                    gy,
                    lx,
                    ly,
                    cell.road_type,
                );
            }
        }
    }

    let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; positions.len()];
    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD
            | bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_quad_3d(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    x0: f32,
    z0: f32,
    x1: f32,
    z1: f32,
    y: f32,
    color: [f32; 4],
) {
    let vi = positions.len() as u32;
    positions.push([x0, y, z0]);
    positions.push([x1, y, z0]);
    positions.push([x1, y, z1]);
    positions.push([x0, y, z1]);
    normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
    colors.push(color);
    colors.push(color);
    colors.push(color);
    colors.push(color);
    indices.push(vi);
    indices.push(vi + 2);
    indices.push(vi + 1);
    indices.push(vi);
    indices.push(vi + 3);
    indices.push(vi + 2);
}

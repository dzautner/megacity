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

/// Compute a per-cell Y height. Each cell quad uses the cell's elevation for
/// its four corners, interpolated with neighbors to create smooth slopes.
/// Water cells use a flat water level.
fn cell_corner_heights(grid: &WorldGrid, gx: usize, gy: usize) -> [f32; 4] {
    let y = grid.elevation_y(gx, gy);
    // For a smooth mesh, average with diagonal neighbors at each corner.
    // Corners: [top-left, top-right, bottom-right, bottom-left]
    //   tl = avg(cell, left, up, up-left)
    //   tr = avg(cell, right, up, up-right)
    //   br = avg(cell, right, down, down-right)
    //   bl = avg(cell, left, down, down-left)
    let e = |gx: usize, gy: usize| -> f32 { grid.elevation_y(gx, gy) };

    let has_left = gx > 0;
    let has_right = gx + 1 < GRID_WIDTH;
    let has_up = gy > 0;
    let has_down = gy + 1 < GRID_HEIGHT;

    let el = if has_left { e(gx - 1, gy) } else { y };
    let er = if has_right { e(gx + 1, gy) } else { y };
    let eu = if has_up { e(gx, gy - 1) } else { y };
    let ed = if has_down { e(gx, gy + 1) } else { y };

    let eul = if has_up && has_left {
        e(gx - 1, gy - 1)
    } else {
        y
    };
    let eur = if has_up && has_right {
        e(gx + 1, gy - 1)
    } else {
        y
    };
    let edl = if has_down && has_left {
        e(gx - 1, gy + 1)
    } else {
        y
    };
    let edr = if has_down && has_right {
        e(gx + 1, gy + 1)
    } else {
        y
    };

    let tl = (y + el + eu + eul) * 0.25;
    let tr = (y + er + eu + eur) * 0.25;
    let br = (y + er + ed + edr) * 0.25;
    let bl = (y + el + ed + edl) * 0.25;

    [tl, tr, br, bl]
}

/// Compute a face normal from three positions.
fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let nx = u[1] * v[2] - u[2] * v[1];
    let ny = u[2] * v[0] - u[0] * v[2];
    let nz = u[0] * v[1] - u[1] * v[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-8 {
        [0.0, 1.0, 0.0]
    } else {
        [nx / len, ny / len, nz / len]
    }
}

/// Average normal for a vertex shared by two triangles of a quad.
fn avg_normal(n1: [f32; 3], n2: [f32; 3]) -> [f32; 3] {
    let x = n1[0] + n2[0];
    let y = n1[1] + n2[1];
    let z = n1[2] + n2[2];
    let len = (x * x + y * y + z * z).sqrt();
    if len < 1e-8 {
        [0.0, 1.0, 0.0]
    } else {
        [x / len, y / len, z / len]
    }
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

            // Heightmap: compute per-corner Y from elevation grid
            let [y_tl, y_tr, y_br, y_bl] = cell_corner_heights(grid, gx, gy);

            let c: [f32; 4] = color.to_srgba().to_f32_array();

            // Cheap coastline blending: tint cells adjacent to water
            let c = if cell.cell_type != CellType::Road {
                coast_tint(grid, gx, gy, c, cell.cell_type)
            } else {
                c
            };

            // 4 vertices: TL, TR, BR, BL
            let vi = positions.len() as u32;
            let p_tl = [x0, y_tl, z0];
            let p_tr = [x1, y_tr, z0];
            let p_br = [x1, y_br, z1];
            let p_bl = [x0, y_bl, z1];

            positions.push(p_tl);
            positions.push(p_tr);
            positions.push(p_br);
            positions.push(p_bl);

            // Compute normals from the two triangles of the quad
            let n1 = face_normal(p_tl, p_br, p_tr); // tri 0: TL->BR->TR
            let n2 = face_normal(p_tl, p_bl, p_br); // tri 1: TL->BL->BR

            // Average normals at shared vertices for smooth shading
            let n_tl = avg_normal(n1, n2);
            let n_br = avg_normal(n1, n2);
            normals.push(n_tl);  // TL (shared by both tris)
            normals.push(n1);    // TR (only tri 0)
            normals.push(n_br);  // BR (shared by both tris)
            normals.push(n2);    // BL (only tri 1)

            colors.extend_from_slice(&[c; 4]);

            // Two triangles: TL-BR-TR and TL-BL-BR
            indices.push(vi);
            indices.push(vi + 2);
            indices.push(vi + 1);
            indices.push(vi);
            indices.push(vi + 3);
            indices.push(vi + 2);

            // Road surface and markings
            if cell.cell_type == CellType::Road && *overlay == OverlayMode::None {
                let road_y = grid.elevation_y(gx, gy);
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
                    road_y,
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

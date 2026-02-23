use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{CellType, RoadType, WorldGrid};
use simulation::roads::RoadNetwork;

use super::lane_markings::{
    draw_curbs, draw_horizontal_lane_markings, draw_vertical_lane_markings,
};
use super::mesh::push_quad_3d;

fn count_road_neighbors_8(grid: &WorldGrid, gx: usize, gy: usize) -> usize {
    let mut count = 0;
    for &(dx, dy) in &[
        (-1isize, -1isize),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ] {
        let nx = gx as isize + dx;
        let ny = gy as isize + dy;
        if nx >= 0
            && ny >= 0
            && (nx as usize) < GRID_WIDTH
            && (ny as usize) < GRID_HEIGHT
            && grid.get(nx as usize, ny as usize).cell_type == CellType::Road
        {
            count += 1;
        }
    }
    count
}

#[allow(clippy::too_many_arguments)]
pub(super) fn add_road_markings(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    grid: &WorldGrid,
    roads: &RoadNetwork,
    gx: usize,
    gy: usize,
    lx: usize,
    ly: usize,
    road_type: RoadType,
) {
    if road_type == RoadType::Path {
        return;
    }

    let is_intersection = roads
        .intersections
        .contains(&simulation::roads::RoadNode(gx, gy));

    let has_left = gx > 0 && grid.get(gx - 1, gy).cell_type == CellType::Road;
    let has_right = gx + 1 < GRID_WIDTH && grid.get(gx + 1, gy).cell_type == CellType::Road;
    let has_up = gy + 1 < GRID_HEIGHT && grid.get(gx, gy + 1).cell_type == CellType::Road;
    let has_down = gy > 0 && grid.get(gx, gy - 1).cell_type == CellType::Road;

    let x_base = lx as f32 * CELL_SIZE;
    let z_base = ly as f32 * CELL_SIZE;
    let cx = x_base + CELL_SIZE * 0.5;
    let cz = z_base + CELL_SIZE * 0.5;

    // Road surface width varies by type (in world units, CELL_SIZE = 16)
    let road_half_w: f32 = match road_type {
        RoadType::Local | RoadType::OneWay => 3.5,
        RoadType::Avenue => 5.0,
        RoadType::Boulevard => 7.0,
        RoadType::Highway => 7.5,
        RoadType::Path => 3.0,
    };

    // Asphalt color — dark like real roads
    let noise = ((gx.wrapping_mul(3571).wrapping_add(gy.wrapping_mul(2143))) % 100) as f32 / 100.0;
    let av = (noise - 0.5) * 0.02;
    let asphalt: [f32; 4] = match road_type {
        RoadType::Highway => [0.10 + av, 0.10 + av, 0.12 + av, 1.0],
        RoadType::Boulevard => [0.13 + av, 0.13 + av, 0.15 + av, 1.0],
        RoadType::Avenue => [0.16 + av, 0.16 + av, 0.18 + av, 1.0],
        _ => [0.20 + av, 0.20 + av, 0.22 + av, 1.0],
    };

    let y_road = 0.03;
    let y_mark = 0.06;
    let y_curb = 0.12;

    let is_horizontal = has_left || has_right;
    let is_vertical = has_up || has_down;

    // Global world coordinates for continuous dash patterns
    let world_x = gx as f32 * CELL_SIZE;
    let world_z = gy as f32 * CELL_SIZE;

    // Dense area detection: count all 8 neighbors
    let road_neighbors_8 = count_road_neighbors_8(grid, gx, gy);
    let is_dense = road_neighbors_8 >= 6;

    // --- Asphalt road surface ---
    if is_intersection {
        draw_intersection_asphalt(
            positions, normals, colors, indices, x_base, z_base, y_road, asphalt,
        );
        if !is_dense {
            draw_crosswalks(
                positions, normals, colors, indices, x_base, z_base, y_mark, has_left, has_right,
                has_up, has_down,
            );
        }
    } else {
        draw_straight_asphalt(
            positions,
            normals,
            colors,
            indices,
            x_base,
            z_base,
            cx,
            cz,
            road_half_w,
            y_road,
            asphalt,
            is_horizontal,
            is_vertical,
            has_left,
            has_right,
            has_up,
            has_down,
        );

        // --- Lane markings (skip in dense areas) ---
        if !is_dense {
            if is_horizontal {
                draw_horizontal_lane_markings(
                    positions,
                    normals,
                    colors,
                    indices,
                    road_type,
                    x_base,
                    cx,
                    cz,
                    world_x,
                    road_half_w,
                    y_mark,
                    y_curb,
                );
            }
            if is_vertical {
                draw_vertical_lane_markings(
                    positions,
                    normals,
                    colors,
                    indices,
                    road_type,
                    z_base,
                    cx,
                    cz,
                    world_z,
                    road_half_w,
                    y_mark,
                    y_curb,
                );
            }
        }
    }

    // --- Curb edges where road meets non-road ---
    draw_curbs(
        positions,
        normals,
        colors,
        indices,
        x_base,
        z_base,
        cx,
        cz,
        road_half_w,
        y_curb,
        is_intersection,
        is_horizontal,
        is_vertical,
        has_left,
        has_right,
        has_up,
        has_down,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_intersection_asphalt(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    x_base: f32,
    z_base: f32,
    y_road: f32,
    asphalt: [f32; 4],
) {
    push_quad_3d(
        positions,
        normals,
        colors,
        indices,
        x_base,
        z_base,
        x_base + CELL_SIZE,
        z_base + CELL_SIZE,
        y_road,
        asphalt,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_crosswalks(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    x_base: f32,
    z_base: f32,
    y_mark: f32,
    has_left: bool,
    has_right: bool,
    has_up: bool,
    has_down: bool,
) {
    let stripe_w = 0.5;
    let stripe_gap = 1.2;
    let cw_color: [f32; 4] = [0.82, 0.82, 0.80, 0.75];
    let cw_inset = 1.5;

    if has_down {
        let zz = z_base + cw_inset;
        let mut sx = x_base + 1.5;
        while sx + stripe_w < x_base + CELL_SIZE - 1.5 {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                sx,
                zz,
                sx + stripe_w,
                zz + 2.0,
                y_mark,
                cw_color,
            );
            sx += stripe_w + stripe_gap;
        }
    }
    if has_up {
        let zz = z_base + CELL_SIZE - cw_inset - 2.0;
        let mut sx = x_base + 1.5;
        while sx + stripe_w < x_base + CELL_SIZE - 1.5 {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                sx,
                zz,
                sx + stripe_w,
                zz + 2.0,
                y_mark,
                cw_color,
            );
            sx += stripe_w + stripe_gap;
        }
    }
    if has_left {
        let xx = x_base + cw_inset;
        let mut sz = z_base + 1.5;
        while sz + stripe_w < z_base + CELL_SIZE - 1.5 {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                xx,
                sz,
                xx + 2.0,
                sz + stripe_w,
                y_mark,
                cw_color,
            );
            sz += stripe_w + stripe_gap;
        }
    }
    if has_right {
        let xx = x_base + CELL_SIZE - cw_inset - 2.0;
        let mut sz = z_base + 1.5;
        while sz + stripe_w < z_base + CELL_SIZE - 1.5 {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                xx,
                sz,
                xx + 2.0,
                sz + stripe_w,
                y_mark,
                cw_color,
            );
            sz += stripe_w + stripe_gap;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_straight_asphalt(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    x_base: f32,
    z_base: f32,
    cx: f32,
    cz: f32,
    road_half_w: f32,
    y_road: f32,
    asphalt: [f32; 4],
    is_horizontal: bool,
    is_vertical: bool,
    has_left: bool,
    has_right: bool,
    has_up: bool,
    has_down: bool,
) {
    if is_horizontal && !is_vertical {
        let z_top = if has_down { z_base } else { cz - road_half_w };
        let z_bot = if has_up {
            z_base + CELL_SIZE
        } else {
            cz + road_half_w
        };
        push_quad_3d(
            positions,
            normals,
            colors,
            indices,
            x_base,
            z_top,
            x_base + CELL_SIZE,
            z_bot,
            y_road,
            asphalt,
        );
    } else if is_vertical && !is_horizontal {
        let x_left = if has_left { x_base } else { cx - road_half_w };
        let x_right = if has_right {
            x_base + CELL_SIZE
        } else {
            cx + road_half_w
        };
        push_quad_3d(
            positions,
            normals,
            colors,
            indices,
            x_left,
            z_base,
            x_right,
            z_base + CELL_SIZE,
            y_road,
            asphalt,
        );
    } else {
        // Both horizontal and vertical (but not intersection) — draw both strips
        let z_top = if has_down { z_base } else { cz - road_half_w };
        let z_bot = if has_up {
            z_base + CELL_SIZE
        } else {
            cz + road_half_w
        };
        push_quad_3d(
            positions,
            normals,
            colors,
            indices,
            x_base,
            z_top,
            x_base + CELL_SIZE,
            z_bot,
            y_road,
            asphalt,
        );

        let x_left = if has_left { x_base } else { cx - road_half_w };
        let x_right = if has_right {
            x_base + CELL_SIZE
        } else {
            cx + road_half_w
        };
        push_quad_3d(
            positions,
            normals,
            colors,
            indices,
            x_left,
            z_base,
            x_right,
            z_base + CELL_SIZE,
            y_road,
            asphalt,
        );
    }
}

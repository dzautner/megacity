use simulation::config::CELL_SIZE;
use simulation::grid::RoadType;

use super::mesh::push_quad_3d;

/// Draw horizontal lane markings for different road types.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_horizontal_lane_markings(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    road_type: RoadType,
    x_base: f32,
    cx: f32,
    cz: f32,
    world_x: f32,
    road_half_w: f32,
    y_mark: f32,
    y_curb: f32,
) {
    let lw = 0.12; // line half-width
    let dash = 3.0;
    let gap = 4.0;
    let period = dash + gap;
    let x0 = x_base;
    let x1 = x_base + CELL_SIZE;
    let _ = cx; // center x not needed for horizontal — cz is used instead

    match road_type {
        RoadType::Local | RoadType::OneWay => {
            let mut sx = x0 - (world_x % period);
            while sx < x1 {
                let d0 = sx.max(x0);
                let d1 = (sx + dash).min(x1);
                if d1 > d0 {
                    push_quad_3d(
                        positions,
                        normals,
                        colors,
                        indices,
                        d0,
                        cz - lw,
                        d1,
                        cz + lw,
                        y_mark,
                        [0.95, 0.95, 0.90, 0.85],
                    );
                }
                sx += period;
            }
        }
        RoadType::Avenue => {
            let s = 0.2;
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x0,
                cz - s - lw,
                x1,
                cz - s + lw,
                y_mark,
                [0.90, 0.80, 0.15, 0.90],
            );
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x0,
                cz + s - lw,
                x1,
                cz + s + lw,
                y_mark,
                [0.90, 0.80, 0.15, 0.90],
            );
        }
        RoadType::Boulevard => {
            let lane_w = road_half_w * 0.5;
            let median_hw = 0.8;
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x0,
                cz - median_hw,
                x1,
                cz + median_hw,
                y_curb * 0.7,
                [0.35, 0.45, 0.30, 1.0],
            );
            for &off in &[-lane_w, lane_w] {
                let mut sx = x0 - (world_x % period);
                while sx < x1 {
                    let d0 = sx.max(x0);
                    let d1 = (sx + dash).min(x1);
                    if d1 > d0 {
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            d0,
                            cz + off - lw,
                            d1,
                            cz + off + lw,
                            y_mark,
                            [1.0, 1.0, 1.0, 0.35],
                        );
                    }
                    sx += period;
                }
            }
        }
        RoadType::Highway => {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x0,
                cz - 0.2,
                x1,
                cz + 0.2,
                y_mark,
                [0.80, 0.70, 0.12, 0.75],
            );
            let lane_w = road_half_w * 0.5;
            for &off in &[-lane_w, lane_w] {
                let mut sx = x0 - (world_x % period);
                while sx < x1 {
                    let d0 = sx.max(x0);
                    let d1 = (sx + dash).min(x1);
                    if d1 > d0 {
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            d0,
                            cz + off - lw,
                            d1,
                            cz + off + lw,
                            y_mark,
                            [0.95, 0.95, 0.90, 0.85],
                        );
                    }
                    sx += period;
                }
            }
            for &edge in &[-road_half_w + 0.3, road_half_w - 0.3] {
                push_quad_3d(
                    positions,
                    normals,
                    colors,
                    indices,
                    x0,
                    cz + edge - lw,
                    x1,
                    cz + edge + lw,
                    y_mark,
                    [1.0, 1.0, 1.0, 0.55],
                );
            }
        }
        RoadType::Path => {}
    }
}

/// Draw vertical lane markings for different road types.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_vertical_lane_markings(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    road_type: RoadType,
    z_base: f32,
    cx: f32,
    _cz: f32,
    world_z: f32,
    road_half_w: f32,
    y_mark: f32,
    y_curb: f32,
) {
    let lw = 0.12; // line half-width
    let dash = 3.0;
    let gap = 4.0;
    let period = dash + gap;
    let z0 = z_base;
    let z1 = z_base + CELL_SIZE;

    match road_type {
        RoadType::Local | RoadType::OneWay => {
            let mut sz = z0 - (world_z % period);
            while sz < z1 {
                let d0 = sz.max(z0);
                let d1 = (sz + dash).min(z1);
                if d1 > d0 {
                    push_quad_3d(
                        positions,
                        normals,
                        colors,
                        indices,
                        cx - lw,
                        d0,
                        cx + lw,
                        d1,
                        y_mark,
                        [0.95, 0.95, 0.90, 0.85],
                    );
                }
                sz += period;
            }
        }
        RoadType::Avenue => {
            let s = 0.2;
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                cx - s - lw,
                z0,
                cx - s + lw,
                z1,
                y_mark,
                [0.90, 0.80, 0.15, 0.90],
            );
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                cx + s - lw,
                z0,
                cx + s + lw,
                z1,
                y_mark,
                [0.90, 0.80, 0.15, 0.90],
            );
        }
        RoadType::Boulevard => {
            let lane_w = road_half_w * 0.5;
            let median_hw = 0.8;
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                cx - median_hw,
                z0,
                cx + median_hw,
                z1,
                y_curb * 0.7,
                [0.35, 0.45, 0.30, 1.0],
            );
            for &off in &[-lane_w, lane_w] {
                let mut sz = z0 - (world_z % period);
                while sz < z1 {
                    let d0 = sz.max(z0);
                    let d1 = (sz + dash).min(z1);
                    if d1 > d0 {
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            cx + off - lw,
                            d0,
                            cx + off + lw,
                            d1,
                            y_mark,
                            [1.0, 1.0, 1.0, 0.35],
                        );
                    }
                    sz += period;
                }
            }
        }
        RoadType::Highway => {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                cx - 0.2,
                z0,
                cx + 0.2,
                z1,
                y_mark,
                [0.80, 0.70, 0.12, 0.75],
            );
            let lane_w = road_half_w * 0.5;
            for &off in &[-lane_w, lane_w] {
                let mut sz = z0 - (world_z % period);
                while sz < z1 {
                    let d0 = sz.max(z0);
                    let d1 = (sz + dash).min(z1);
                    if d1 > d0 {
                        push_quad_3d(
                            positions,
                            normals,
                            colors,
                            indices,
                            cx + off - lw,
                            d0,
                            cx + off + lw,
                            d1,
                            y_mark,
                            [0.95, 0.95, 0.90, 0.85],
                        );
                    }
                    sz += period;
                }
            }
            for &edge in &[-road_half_w + 0.3, road_half_w - 0.3] {
                push_quad_3d(
                    positions,
                    normals,
                    colors,
                    indices,
                    cx + edge - lw,
                    z0,
                    cx + edge + lw,
                    z1,
                    y_mark,
                    [1.0, 1.0, 1.0, 0.55],
                );
            }
        }
        RoadType::Path => {}
    }
}

/// Draw curb edges where road meets non-road.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_curbs(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    x_base: f32,
    z_base: f32,
    cx: f32,
    cz: f32,
    road_half_w: f32,
    y_curb: f32,
    is_intersection: bool,
    is_horizontal: bool,
    is_vertical: bool,
    has_left: bool,
    has_right: bool,
    has_up: bool,
    has_down: bool,
) {
    let curb_w = 0.3;
    let curb_color: [f32; 4] = [0.62, 0.60, 0.57, 1.0];

    if is_intersection {
        if !has_left {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                z_base,
                x_base + curb_w,
                z_base + CELL_SIZE,
                y_curb,
                curb_color,
            );
        }
        if !has_right {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base + CELL_SIZE - curb_w,
                z_base,
                x_base + CELL_SIZE,
                z_base + CELL_SIZE,
                y_curb,
                curb_color,
            );
        }
        if !has_down {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                z_base,
                x_base + CELL_SIZE,
                z_base + curb_w,
                y_curb,
                curb_color,
            );
        }
        if !has_up {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                z_base + CELL_SIZE - curb_w,
                x_base + CELL_SIZE,
                z_base + CELL_SIZE,
                y_curb,
                curb_color,
            );
        }
    } else {
        if !has_left && is_vertical {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                cx - road_half_w - curb_w,
                z_base,
                cx - road_half_w,
                z_base + CELL_SIZE,
                y_curb,
                curb_color,
            );
        }
        if !has_right && is_vertical {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                cx + road_half_w,
                z_base,
                cx + road_half_w + curb_w,
                z_base + CELL_SIZE,
                y_curb,
                curb_color,
            );
        }
        if !has_down && is_horizontal {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                cz - road_half_w - curb_w,
                x_base + CELL_SIZE,
                cz - road_half_w,
                y_curb,
                curb_color,
            );
        }
        if !has_up && is_horizontal {
            push_quad_3d(
                positions,
                normals,
                colors,
                indices,
                x_base,
                cz + road_half_w,
                x_base + CELL_SIZE,
                cz + road_half_w + curb_w,
                y_curb,
                curb_color,
            );
        }
    }
}

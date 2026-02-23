use bevy::prelude::*;
use bevy::render::mesh::Indices;

use simulation::grid::RoadType;

use super::bezier::{bezier_tangent, evaluate_bezier};
use super::lane_markings::add_curve_lane_markings;

/// Tessellate a road segment into a triangle strip mesh with sidewalks and lane markings.
#[allow(clippy::too_many_arguments)]
pub fn tessellate_road_segment(
    p0: &Vec2,
    p1: &Vec2,
    p2: &Vec2,
    p3: &Vec2,
    road_type: RoadType,
    arc_length: f32,
    trim_start: f32,
    trim_end: f32,
) -> Mesh {
    let road_half_w: f32 = match road_type {
        RoadType::Path => 1.5,
        RoadType::OneWay => 3.0,
        RoadType::Local => 4.0,
        RoadType::Avenue => 6.0,
        RoadType::Boulevard => 8.0,
        RoadType::Highway => 10.0,
    };

    // Sidewalk width varies by road type
    let sidewalk_w: f32 = match road_type {
        RoadType::Path => 0.5,
        RoadType::OneWay | RoadType::Local => 2.0,
        RoadType::Avenue => 3.0,
        RoadType::Boulevard => 4.0,
        RoadType::Highway => 1.5, // narrow shoulder
    };
    let total_half_w = road_half_w + sidewalk_w;
    let curb_w: f32 = 0.4; // fixed narrow curb strip

    // Number of sample points along the curve
    let sample_count = ((arc_length / 4.0).ceil() as usize).clamp(8, 128);

    let y_sidewalk = 0.02;
    let y_road = 0.04;
    let y_mark = 0.07;

    // Asphalt color varies by road type (high contrast)
    let asphalt: [f32; 4] = match road_type {
        RoadType::Highway => [0.10, 0.10, 0.12, 1.0], // very dark — fresh asphalt
        RoadType::Boulevard => [0.16, 0.16, 0.20, 1.0],
        RoadType::Avenue => [0.22, 0.22, 0.25, 1.0],
        RoadType::Local | RoadType::OneWay => [0.32, 0.32, 0.34, 1.0], // lighter, worn
        RoadType::Path => [0.52, 0.47, 0.36, 1.0],                     // sandy/gravel
    };

    // Sidewalk color
    let sidewalk_color: [f32; 4] = match road_type {
        RoadType::Path => [0.42, 0.40, 0.34, 1.0], // dirt shoulder
        RoadType::Highway => [0.35, 0.35, 0.33, 1.0], // gravel shoulder
        _ => [0.58, 0.56, 0.53, 1.0],              // concrete sidewalk
    };

    // Curb color (border between sidewalk and road)
    let curb_color: [f32; 4] = [0.50, 0.48, 0.45, 1.0];

    // 6 vertices per sample: sidewalk_L, curb_L, road_L, road_R, curb_R, sidewalk_R
    let verts_per_sample: u32 = 6;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(sample_count * 6);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(sample_count * 6);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(sample_count * 6);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(sample_count * 6);
    let mut indices: Vec<u32> = Vec::with_capacity(sample_count * 18);

    let mut cumulative_len = 0.0_f32;
    let mut prev_pt = evaluate_bezier(*p0, *p1, *p2, *p3, 0.0);

    for i in 0..=sample_count {
        let t = i as f32 / sample_count as f32;
        let pt = evaluate_bezier(*p0, *p1, *p2, *p3, t);
        let tangent = bezier_tangent(*p0, *p1, *p2, *p3, t);

        if i > 0 {
            cumulative_len += (pt - prev_pt).length();
        }
        prev_pt = pt;

        let tan_norm = tangent.normalize_or_zero();
        let perp = Vec2::new(-tan_norm.y, tan_norm.x);

        let u = cumulative_len / arc_length.max(1.0);

        // 6 positions across the road width
        let sw_l = pt - perp * total_half_w;
        let curb_l = pt - perp * road_half_w;
        let road_l = pt - perp * (road_half_w - curb_w);
        let road_r = pt + perp * (road_half_w - curb_w);
        let curb_r = pt + perp * road_half_w;
        let sw_r = pt + perp * total_half_w;

        // Sidewalk left (at sidewalk height)
        positions.push([sw_l.x, y_sidewalk, sw_l.y]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(sidewalk_color);
        uvs.push([0.0, u]);

        // Curb left (at road height — transition)
        positions.push([curb_l.x, y_road, curb_l.y]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(curb_color);
        uvs.push([0.15, u]);

        // Road left inner
        positions.push([road_l.x, y_road, road_l.y]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(asphalt);
        uvs.push([0.3, u]);

        // Road right inner
        positions.push([road_r.x, y_road, road_r.y]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(asphalt);
        uvs.push([0.7, u]);

        // Curb right
        positions.push([curb_r.x, y_road, curb_r.y]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(curb_color);
        uvs.push([0.85, u]);

        // Sidewalk right
        positions.push([sw_r.x, y_sidewalk, sw_r.y]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(sidewalk_color);
        uvs.push([1.0, u]);

        // 5 quads across the width
        if i > 0 {
            let base = (i - 1) as u32 * verts_per_sample;
            let next = i as u32 * verts_per_sample;

            for j in 0..(verts_per_sample - 1) {
                indices.push(base + j);
                indices.push(next + j);
                indices.push(base + j + 1);

                indices.push(base + j + 1);
                indices.push(next + j);
                indices.push(next + j + 1);
            }
        }
    }

    // Lane markings (center dashes) — trimmed near junctions
    if road_type != RoadType::Path {
        add_curve_lane_markings(
            &mut positions,
            &mut normals,
            &mut colors,
            &mut uvs,
            &mut indices,
            *p0,
            *p1,
            *p2,
            *p3,
            road_type,
            road_half_w,
            arc_length,
            y_mark,
            sample_count,
            trim_start,
            trim_end,
        );
    }

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

use bevy::prelude::*;
use bevy::render::mesh::Indices;

use simulation::grid::RoadType;
use simulation::road_segments::{RoadSegmentStore, SegmentId};

/// Marker component for road segment mesh entities.
#[derive(Component)]
pub struct RoadSegmentMesh {
    pub segment_id: SegmentId,
}

/// Tracks the number of segments we've already spawned meshes for,
/// so we know when new segments appear.
#[derive(Default)]
pub struct LastSyncedCount(usize);

/// Synchronize road segment meshes with the RoadSegmentStore.
/// Spawns meshes for new segments, despawns for removed ones.
pub fn sync_road_segment_meshes(
    store: Res<RoadSegmentStore>,
    existing: Query<(Entity, &RoadSegmentMesh)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut local_count: Local<LastSyncedCount>,
) {
    if !store.is_changed() {
        return;
    }

    // Collect existing segment IDs
    let existing_ids: Vec<(Entity, SegmentId)> = existing
        .iter()
        .map(|(e, m)| (e, m.segment_id))
        .collect();

    // Despawn meshes for segments that no longer exist
    for (entity, id) in &existing_ids {
        if !store.segments.iter().any(|s| s.id == *id) {
            commands.entity(*entity).despawn();
        }
    }

    // Spawn meshes for new segments
    for segment in &store.segments {
        let already_exists = existing_ids.iter().any(|(_, id)| *id == segment.id);
        if already_exists {
            continue;
        }

        let mesh = tessellate_road_segment(
            &segment.p0,
            &segment.p1,
            &segment.p2,
            &segment.p3,
            segment.road_type,
            segment.arc_length,
        );

        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.9,
            ..default()
        });

        commands.spawn((
            RoadSegmentMesh {
                segment_id: segment.id,
            },
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::IDENTITY,
        ));
    }

    local_count.0 = store.segments.len();
}

/// Tessellate a road segment into a triangle strip mesh with lane markings.
fn tessellate_road_segment(
    p0: &Vec2,
    p1: &Vec2,
    p2: &Vec2,
    p3: &Vec2,
    road_type: RoadType,
    arc_length: f32,
) -> Mesh {
    let road_half_w: f32 = match road_type {
        RoadType::Local | RoadType::OneWay => 3.5,
        RoadType::Avenue => 5.0,
        RoadType::Boulevard => 7.0,
        RoadType::Highway => 7.5,
        RoadType::Path => 2.0,
    };

    // Number of sample points along the curve
    let sample_count = ((arc_length / 4.0).ceil() as usize).clamp(8, 128);

    let y_road = 0.04; // slightly above grid road surface
    let y_mark = 0.07;

    // Asphalt color
    let asphalt: [f32; 4] = match road_type {
        RoadType::Highway => [0.18, 0.18, 0.22, 1.0],
        RoadType::Path => [0.40, 0.36, 0.28, 1.0],
        _ => [0.25, 0.25, 0.28, 1.0],
    };

    // Edge darkening color
    let edge_color: [f32; 4] = [
        asphalt[0] * 0.7,
        asphalt[1] * 0.7,
        asphalt[2] * 0.7,
        1.0,
    ];

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(sample_count * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(sample_count * 4);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(sample_count * 4);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(sample_count * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(sample_count * 12);

    // Generate vertices along the curve
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
        // Perpendicular (rotate 90 degrees CCW in XZ plane)
        let perp = Vec2::new(-tan_norm.y, tan_norm.x);

        let u = cumulative_len / arc_length.max(1.0);

        // Left edge vertex (darkened)
        let left = pt - perp * road_half_w;
        positions.push([left.x, y_road, left.y]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(edge_color);
        uvs.push([0.0, u]);

        // Left inner vertex
        let left_inner = pt - perp * (road_half_w * 0.7);
        positions.push([left_inner.x, y_road, left_inner.y]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(asphalt);
        uvs.push([0.15, u]);

        // Right inner vertex
        let right_inner = pt + perp * (road_half_w * 0.7);
        positions.push([right_inner.x, y_road, right_inner.y]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(asphalt);
        uvs.push([0.85, u]);

        // Right edge vertex (darkened)
        let right = pt + perp * road_half_w;
        positions.push([right.x, y_road, right.y]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(edge_color);
        uvs.push([1.0, u]);

        // Build triangle strip indices
        if i > 0 {
            let base = (i - 1) as u32 * 4;
            let next = i as u32 * 4;

            // 3 quads across the width (left-edge, center, right-edge)
            for j in 0..3u32 {
                indices.push(base + j);
                indices.push(next + j);
                indices.push(base + j + 1);

                indices.push(base + j + 1);
                indices.push(next + j);
                indices.push(next + j + 1);
            }
        }
    }

    // Lane markings (center dashes)
    if road_type != RoadType::Path {
        add_curve_lane_markings(
            &mut positions,
            &mut normals,
            &mut colors,
            &mut uvs,
            &mut indices,
            *p0, *p1, *p2, *p3,
            road_type,
            road_half_w,
            arc_length,
            y_mark,
            sample_count,
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

/// Add dashed center line and lane markings along the curve.
#[allow(clippy::too_many_arguments)]
fn add_curve_lane_markings(
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
    y: f32,
    _sample_count: usize,
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
            // Dashed white center line
            generate_dashed_line_along_curve(
                positions, normals, colors, uvs, indices,
                p0, p1, p2, p3,
                0.0, // center offset
                line_hw, y, arc_length, step, total_steps,
                dash_len, gap_len, white,
            );
        }
        RoadType::Avenue => {
            // Double yellow center lines
            let offset = 0.2;
            generate_solid_line_along_curve(
                positions, normals, colors, uvs, indices,
                p0, p1, p2, p3,
                -offset, line_hw, y, arc_length, step, total_steps, yellow,
            );
            generate_solid_line_along_curve(
                positions, normals, colors, uvs, indices,
                p0, p1, p2, p3,
                offset, line_hw, y, arc_length, step, total_steps, yellow,
            );
        }
        RoadType::Boulevard => {
            // Green median
            let median_hw = 0.8;
            generate_solid_line_along_curve(
                positions, normals, colors, uvs, indices,
                p0, p1, p2, p3,
                0.0, median_hw, y - 0.01, arc_length, step, total_steps,
                [0.35, 0.45, 0.30, 1.0],
            );
            // Dashed white lane lines
            let lane_w = road_half_w * 0.5;
            for &off in &[-lane_w, lane_w] {
                generate_dashed_line_along_curve(
                    positions, normals, colors, uvs, indices,
                    p0, p1, p2, p3,
                    off, line_hw, y, arc_length, step, total_steps,
                    dash_len, gap_len, [1.0, 1.0, 1.0, 0.35],
                );
            }
        }
        RoadType::Highway => {
            // Solid yellow center
            generate_solid_line_along_curve(
                positions, normals, colors, uvs, indices,
                p0, p1, p2, p3,
                0.0, 0.2, y, arc_length, step, total_steps, yellow,
            );
            // Dashed white lane lines
            let lane_w = road_half_w * 0.5;
            for &off in &[-lane_w, lane_w] {
                generate_dashed_line_along_curve(
                    positions, normals, colors, uvs, indices,
                    p0, p1, p2, p3,
                    off, line_hw, y, arc_length, step, total_steps,
                    dash_len, gap_len, [1.0, 1.0, 1.0, 0.45],
                );
            }
            // Solid white edge lines
            for &edge in &[-road_half_w + 0.3, road_half_w - 0.3] {
                generate_solid_line_along_curve(
                    positions, normals, colors, uvs, indices,
                    p0, p1, p2, p3,
                    edge, line_hw, y, arc_length, step, total_steps,
                    [1.0, 1.0, 1.0, 0.55],
                );
            }
        }
        RoadType::Path => {}
    }
}

/// Generate a dashed line along a Bezier curve at a perpendicular offset.
#[allow(clippy::too_many_arguments)]
fn generate_dashed_line_along_curve(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,
    perp_offset: f32,
    half_width: f32,
    y: f32,
    arc_length: f32,
    step: f32,
    total_steps: usize,
    dash_len: f32,
    gap_len: f32,
    color: [f32; 4],
) {
    let period = dash_len + gap_len;
    let mut cum_len = 0.0_f32;
    let mut prev_pt = evaluate_bezier(p0, p1, p2, p3, 0.0);
    let mut in_dash = false;
    let mut dash_start_pts: Option<(Vec2, Vec2)> = None; // (left, right)

    for i in 0..=total_steps {
        let t = (i as f32 * step / arc_length).min(1.0);
        let pt = evaluate_bezier(p0, p1, p2, p3, t);
        let tangent = bezier_tangent(p0, p1, p2, p3, t).normalize_or_zero();
        let perp = Vec2::new(-tangent.y, tangent.x);

        if i > 0 {
            cum_len += (pt - prev_pt).length();
        }
        prev_pt = pt;

        let phase = cum_len % period;
        let should_be_dash = phase < dash_len;

        let center = pt + perp * perp_offset;
        let left = center - perp * half_width;
        let right = center + perp * half_width;

        if should_be_dash && !in_dash {
            // Start new dash
            dash_start_pts = Some((left, right));
            in_dash = true;
        } else if in_dash && (!should_be_dash || i == total_steps) {
            // End dash - emit quad
            if let Some((sl, sr)) = dash_start_pts {
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
            dash_start_pts = None;
        }
    }
}

/// Generate a solid continuous line along a Bezier curve at a perpendicular offset.
#[allow(clippy::too_many_arguments)]
fn generate_solid_line_along_curve(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,
    perp_offset: f32,
    half_width: f32,
    y: f32,
    _arc_length: f32,
    _step: f32,
    total_steps: usize,
    color: [f32; 4],
) {
    let segments = total_steps.min(64);
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let pt = evaluate_bezier(p0, p1, p2, p3, t);
        let tangent = bezier_tangent(p0, p1, p2, p3, t).normalize_or_zero();
        let perp = Vec2::new(-tangent.y, tangent.x);

        let center = pt + perp * perp_offset;
        let left = center - perp * half_width;
        let right = center + perp * half_width;

        positions.push([left.x, y, left.y]);
        positions.push([right.x, y, right.y]);
        normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 2]);
        colors.extend_from_slice(&[color; 2]);
        uvs.extend_from_slice(&[[0.0, t]; 2]);

        if i > 0 {
            let base = (positions.len() as u32) - 4;
            indices.push(base);
            indices.push(base + 2);
            indices.push(base + 1);
            indices.push(base + 1);
            indices.push(base + 2);
            indices.push(base + 3);
        }
    }
}

fn evaluate_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;
    p0 * mt3 + p1 * 3.0 * mt2 * t + p2 * 3.0 * mt * t2 + p3 * t3
}

fn bezier_tangent(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;
    (p1 - p0) * 3.0 * mt2 + (p2 - p1) * 6.0 * mt * t + (p3 - p2) * 3.0 * t2
}

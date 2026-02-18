use bevy::prelude::*;
use bevy::render::mesh::Indices;

use simulation::grid::RoadType;
use simulation::road_segments::{RoadSegmentStore, SegmentId, SegmentNodeId};

/// Marker component for road segment mesh entities.
#[derive(Component)]
pub struct RoadSegmentMesh {
    pub segment_id: SegmentId,
}

/// Tracks the number of segments we've already spawned meshes for,
/// so we know when new segments appear.
#[derive(Default)]
pub struct LastSyncedCount(usize);

/// Marker component for intersection fill mesh entities.
#[derive(Component)]
pub struct RoadIntersectionMesh;

/// Synchronize road segment meshes with the RoadSegmentStore.
/// Spawns meshes for new segments, despawns for removed ones.
/// Also generates intersection fill discs where segments meet.
pub fn sync_road_segment_meshes(
    store: Res<RoadSegmentStore>,
    existing: Query<(Entity, &RoadSegmentMesh)>,
    existing_intersections: Query<Entity, With<RoadIntersectionMesh>>,
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

    // Helper: check if a node is a junction (2+ connected segments)
    let is_junction = |node_id: SegmentNodeId| -> bool {
        store.get_node(node_id)
            .map(|n| n.connected_segments.len() >= 2)
            .unwrap_or(false)
    };

    // Spawn meshes for new segments
    for segment in &store.segments {
        let already_exists = existing_ids.iter().any(|(_, id)| *id == segment.id);
        if already_exists {
            continue;
        }

        // Compute trim distances based on junction status
        let road_half_w: f32 = match segment.road_type {
            RoadType::Path => 1.5,
            RoadType::OneWay => 3.0,
            RoadType::Local => 4.0,
            RoadType::Avenue => 6.0,
            RoadType::Boulevard => 8.0,
            RoadType::Highway => 10.0,
        };
        let trim_start = if is_junction(segment.start_node) { road_half_w * 1.2 } else { 0.0 };
        let trim_end = if is_junction(segment.end_node) { road_half_w * 1.2 } else { 0.0 };

        let mesh = tessellate_road_segment(
            &segment.p0,
            &segment.p1,
            &segment.p2,
            &segment.p3,
            segment.road_type,
            segment.arc_length,
            trim_start,
            trim_end,
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

    // Despawn old intersection meshes and regenerate
    for entity in &existing_intersections {
        commands.entity(entity).despawn();
    }

    // Generate intersection fill discs at junction nodes
    let intersection_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        ..default()
    });

    for node in &store.nodes {
        if node.connected_segments.len() < 2 {
            continue;
        }

        // Find the max road dimensions of connected segments
        let mut max_road_hw: f32 = 0.0;
        let mut max_total_hw: f32 = 0.0;
        let mut avg_asphalt = [0.0_f32; 4];
        let mut count = 0.0_f32;

        for &seg_id in &node.connected_segments {
            if let Some(seg) = store.get_segment(seg_id) {
                let rhw: f32 = match seg.road_type {
                    RoadType::Path => 1.5,
                    RoadType::OneWay => 3.0,
                    RoadType::Local => 4.0,
                    RoadType::Avenue => 6.0,
                    RoadType::Boulevard => 8.0,
                    RoadType::Highway => 10.0,
                };
                let sw: f32 = match seg.road_type {
                    RoadType::Path => 0.5,
                    RoadType::OneWay | RoadType::Local => 2.0,
                    RoadType::Avenue => 3.0,
                    RoadType::Boulevard => 4.0,
                    RoadType::Highway => 1.5,
                };
                let asph: [f32; 4] = match seg.road_type {
                    RoadType::Highway => [0.10, 0.10, 0.12, 1.0],
                    RoadType::Boulevard => [0.16, 0.16, 0.20, 1.0],
                    RoadType::Avenue => [0.22, 0.22, 0.25, 1.0],
                    RoadType::Local | RoadType::OneWay => [0.32, 0.32, 0.34, 1.0],
                    RoadType::Path => [0.52, 0.47, 0.36, 1.0],
                };
                max_road_hw = max_road_hw.max(rhw);
                max_total_hw = max_total_hw.max(rhw + sw);
                for j in 0..4 { avg_asphalt[j] += asph[j]; }
                count += 1.0;
            }
        }

        if count < 1.0 { continue; }
        for j in 0..4 { avg_asphalt[j] /= count; }

        let sidewalk_color: [f32; 4] = [0.58, 0.56, 0.53, 1.0];

        // Build a disc mesh: sidewalk disc + asphalt disc on top
        let mesh = build_intersection_disc(
            node.position,
            max_total_hw,
            max_road_hw,
            sidewalk_color,
            avg_asphalt,
        );

        commands.spawn((
            RoadIntersectionMesh,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(intersection_material.clone()),
            Transform::IDENTITY,
        ));
    }

    local_count.0 = store.segments.len();
}

/// Build a disc mesh for an intersection: sidewalk ring + asphalt center.
fn build_intersection_disc(
    center: Vec2,
    outer_radius: f32,
    inner_radius: f32,
    sidewalk_color: [f32; 4],
    asphalt_color: [f32; 4],
) -> Mesh {
    let disc_segments = 24;
    let y_sidewalk = 0.02;
    let y_road = 0.04;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Sidewalk disc (larger, lower)
    let base = positions.len() as u32;
    positions.push([center.x, y_sidewalk, center.y]);
    normals.push([0.0, 1.0, 0.0]);
    colors.push(sidewalk_color);
    uvs.push([0.5, 0.5]);

    for i in 0..=disc_segments {
        let angle = (i as f32 / disc_segments as f32) * std::f32::consts::TAU;
        let x = center.x + outer_radius * angle.cos();
        let z = center.y + outer_radius * angle.sin();
        positions.push([x, y_sidewalk, z]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(sidewalk_color);
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);

        if i > 0 {
            let vi = base + 1 + i as u32;
            indices.push(base);
            indices.push(vi - 1);
            indices.push(vi);
        }
    }

    // Asphalt disc (smaller, higher)
    let base2 = positions.len() as u32;
    positions.push([center.x, y_road, center.y]);
    normals.push([0.0, 1.0, 0.0]);
    colors.push(asphalt_color);
    uvs.push([0.5, 0.5]);

    for i in 0..=disc_segments {
        let angle = (i as f32 / disc_segments as f32) * std::f32::consts::TAU;
        let x = center.x + inner_radius * angle.cos();
        let z = center.y + inner_radius * angle.sin();
        positions.push([x, y_road, z]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(asphalt_color);
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);

        if i > 0 {
            let vi = base2 + 1 + i as u32;
            indices.push(base2);
            indices.push(vi - 1);
            indices.push(vi);
        }
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

/// Tessellate a road segment into a triangle strip mesh with sidewalks and lane markings.
fn tessellate_road_segment(
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
        RoadType::Highway => [0.10, 0.10, 0.12, 1.0],     // very dark — fresh asphalt
        RoadType::Boulevard => [0.16, 0.16, 0.20, 1.0],
        RoadType::Avenue => [0.22, 0.22, 0.25, 1.0],
        RoadType::Local | RoadType::OneWay => [0.32, 0.32, 0.34, 1.0], // lighter, worn
        RoadType::Path => [0.52, 0.47, 0.36, 1.0],         // sandy/gravel
    };

    // Sidewalk color
    let sidewalk_color: [f32; 4] = match road_type {
        RoadType::Path => [0.42, 0.40, 0.34, 1.0],  // dirt shoulder
        RoadType::Highway => [0.35, 0.35, 0.33, 1.0], // gravel shoulder
        _ => [0.58, 0.56, 0.53, 1.0],                 // concrete sidewalk
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
            *p0, *p1, *p2, *p3,
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
    trim_start: f32,
    trim_end: f32,
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
                dash_len, gap_len, white, trim_start, trim_end,
            );
        }
        RoadType::Avenue => {
            // Double yellow center lines
            let offset = 0.3;
            generate_solid_line_along_curve(
                positions, normals, colors, uvs, indices,
                p0, p1, p2, p3,
                -offset, line_hw, y, arc_length, step, total_steps, yellow,
                trim_start, trim_end,
            );
            generate_solid_line_along_curve(
                positions, normals, colors, uvs, indices,
                p0, p1, p2, p3,
                offset, line_hw, y, arc_length, step, total_steps, yellow,
                trim_start, trim_end,
            );
        }
        RoadType::Boulevard => {
            // Wide green median strip
            let median_hw = 1.5;
            generate_solid_line_along_curve(
                positions, normals, colors, uvs, indices,
                p0, p1, p2, p3,
                0.0, median_hw, y - 0.01, arc_length, step, total_steps,
                [0.30, 0.42, 0.25, 1.0], trim_start, trim_end,
            );
            // Dashed white lane lines
            let lane_w = road_half_w * 0.45;
            for &off in &[-lane_w, lane_w] {
                generate_dashed_line_along_curve(
                    positions, normals, colors, uvs, indices,
                    p0, p1, p2, p3,
                    off, line_hw, y, arc_length, step, total_steps,
                    dash_len, gap_len, [1.0, 1.0, 1.0, 0.35],
                    trim_start, trim_end,
                );
            }
        }
        RoadType::Highway => {
            // Concrete jersey barrier in center
            generate_solid_line_along_curve(
                positions, normals, colors, uvs, indices,
                p0, p1, p2, p3,
                0.0, 0.4, y + 0.02, arc_length, step, total_steps,
                [0.55, 0.53, 0.50, 1.0], trim_start, trim_end,
            );
            // Dashed white lane lines
            let lane_w = road_half_w * 0.45;
            for &off in &[-lane_w, lane_w] {
                generate_dashed_line_along_curve(
                    positions, normals, colors, uvs, indices,
                    p0, p1, p2, p3,
                    off, line_hw * 1.5, y, arc_length, step, total_steps,
                    dash_len, gap_len, [1.0, 1.0, 1.0, 0.55],
                    trim_start, trim_end,
                );
            }
            // Solid white edge lines (wider)
            for &edge in &[-road_half_w + 0.5, road_half_w - 0.5] {
                generate_solid_line_along_curve(
                    positions, normals, colors, uvs, indices,
                    p0, p1, p2, p3,
                    edge, line_hw * 1.5, y, arc_length, step, total_steps,
                    [1.0, 1.0, 1.0, 0.65], trim_start, trim_end,
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
    trim_start: f32,
    trim_end: f32,
) {
    let period = dash_len + gap_len;
    let mut cum_len = 0.0_f32;
    let mut prev_pt = evaluate_bezier(p0, p1, p2, p3, 0.0);
    let mut in_dash = false;
    let mut dash_start_pts: Option<(Vec2, Vec2)> = None;
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

        // Skip rendering in trimmed zones
        let in_trim_zone = cum_len < trim_start || cum_len > trim_end_dist;

        let phase = cum_len % period;
        let should_be_dash = phase < dash_len && !in_trim_zone;

        let center = pt + perp * perp_offset;
        let left = center - perp * half_width;
        let right = center + perp * half_width;

        if should_be_dash && !in_dash {
            dash_start_pts = Some((left, right));
            in_dash = true;
        } else if in_dash && (!should_be_dash || i == total_steps) {
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
    arc_length: f32,
    _step: f32,
    total_steps: usize,
    color: [f32; 4],
    trim_start: f32,
    trim_end: f32,
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

        // Skip rendering in trimmed zones
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

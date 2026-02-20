//! Road lane marking decals rendered as separate overlay meshes.
//!
//! Spawns thin geometry slightly above the road surface for center lines,
//! lane dividers, and edge markings.  Marking style varies per [`RoadType`]:
//!
//! | Road type  | Center         | Lane dividers | Edge lines |
//! |------------|----------------|---------------|------------|
//! | Path       | none           | none          | none       |
//! | Local      | none           | none          | none       |
//! | OneWay     | none           | none          | none       |
//! | Avenue     | double yellow  | none          | none       |
//! | Boulevard  | yellow center  | dashed white  | none       |
//! | Highway    | barrier        | dashed white  | solid white|

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;

use simulation::grid::RoadType;
use simulation::road_segments::{RoadSegmentStore, SegmentId, SegmentNodeId};

// ---------------------------------------------------------------------------
// Components & resources
// ---------------------------------------------------------------------------

/// Marker component for a lane-marking overlay entity tied to a road segment.
#[derive(Component)]
pub struct LaneMarkingMesh {
    pub segment_id: SegmentId,
}

/// Tracks the generation counter so we know when to rebuild.
#[derive(Default)]
pub struct LaneMarkingSyncState {
    generation: usize,
}

// ---------------------------------------------------------------------------
// Geometry constants
// ---------------------------------------------------------------------------

/// Y height for marking geometry (above road surface at 0.04).
const Y_MARKING: f32 = 0.06;

/// Half-width of a painted line.
const LINE_HW: f32 = 0.12;

/// Dash length for dashed markings.
const DASH_LEN: f32 = 3.0;

/// Gap length between dashes.
const GAP_LEN: f32 = 4.0;

/// White marking colour (semi-transparent).
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.55];

/// Yellow marking colour (semi-transparent).
const YELLOW: [f32; 4] = [0.80, 0.70, 0.12, 0.65];

/// Jersey barrier colour (concrete grey).
const BARRIER_COLOR: [f32; 4] = [0.55, 0.53, 0.50, 1.0];

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Synchronises lane-marking overlay meshes with the [`RoadSegmentStore`].
///
/// Runs every frame but short-circuits unless the store has changed.
#[allow(clippy::too_many_arguments)]
pub fn sync_lane_marking_meshes(
    store: Res<RoadSegmentStore>,
    existing: Query<(Entity, &LaneMarkingMesh)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut local_state: Local<LaneMarkingSyncState>,
) {
    if !store.is_changed() {
        return;
    }

    // Quick generation check to avoid redundant work.
    let generation = store.segments.len();
    if generation == local_state.generation && generation != 0 {
        // Also need to handle segment removals, so do a full diff below.
    }
    local_state.generation = generation;

    // Collect existing segment IDs.
    let existing_ids: Vec<(Entity, SegmentId)> =
        existing.iter().map(|(e, m)| (e, m.segment_id)).collect();

    // Despawn meshes whose segments have been removed.
    for (entity, id) in &existing_ids {
        if !store.segments.iter().any(|s| s.id == *id) {
            commands.entity(*entity).despawn();
        }
    }

    // Helper: determine whether a node is a junction (2+ connected segments).
    let is_junction = |node_id: SegmentNodeId| -> bool {
        store
            .get_node(node_id)
            .map(|n| n.connected_segments.len() >= 2)
            .unwrap_or(false)
    };

    // Shared unlit material for all lane markings.
    let marking_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.5,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Spawn new lane-marking meshes.
    for segment in &store.segments {
        // Skip if already spawned.
        if existing_ids.iter().any(|(_, id)| *id == segment.id) {
            continue;
        }

        // Only avenue, boulevard, and highway get dedicated lane markings.
        if matches!(
            segment.road_type,
            RoadType::Path | RoadType::Local | RoadType::OneWay
        ) {
            continue;
        }

        let road_half_w = road_half_width(segment.road_type);

        // Compute trim distances near junctions.
        let trim_start = if is_junction(segment.start_node) {
            road_half_w * 1.2
        } else {
            0.0
        };
        let trim_end = if is_junction(segment.end_node) {
            road_half_w * 1.2
        } else {
            0.0
        };

        let mesh = build_lane_marking_mesh(
            segment.p0,
            segment.p1,
            segment.p2,
            segment.p3,
            segment.road_type,
            road_half_w,
            segment.arc_length,
            trim_start,
            trim_end,
        );

        commands.spawn((
            LaneMarkingMesh {
                segment_id: segment.id,
            },
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(marking_material.clone()),
            Transform::IDENTITY,
        ));
    }
}

// ---------------------------------------------------------------------------
// Mesh builders
// ---------------------------------------------------------------------------

/// Build all lane-marking geometry for one road segment.
#[allow(clippy::too_many_arguments)]
fn build_lane_marking_mesh(
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
        // Path / Local / OneWay: no markings (filtered earlier).
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

// ---------------------------------------------------------------------------
// Line primitives along Bezier curves
// ---------------------------------------------------------------------------

/// Emit a **dashed** line strip along the Bezier at a perpendicular offset.
#[allow(clippy::too_many_arguments)]
fn emit_dashed_line(
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
fn emit_solid_line(
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

// ---------------------------------------------------------------------------
// Bezier helpers
// ---------------------------------------------------------------------------

fn eval_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
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

/// Get the road surface half-width for a given road type.
fn road_half_width(road_type: RoadType) -> f32 {
    match road_type {
        RoadType::Path => 1.5,
        RoadType::OneWay => 3.0,
        RoadType::Local => 4.0,
        RoadType::Avenue => 6.0,
        RoadType::Boulevard => 8.0,
        RoadType::Highway => 10.0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_road_half_width_values() {
        assert!((road_half_width(RoadType::Path) - 1.5).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::OneWay) - 3.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Local) - 4.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Avenue) - 6.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Boulevard) - 8.0).abs() < f32::EPSILON);
        assert!((road_half_width(RoadType::Highway) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_eval_bezier_endpoints() {
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(100.0, 0.0);
        let p1 = p0 + (p3 - p0) / 3.0;
        let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

        let start = eval_bezier(p0, p1, p2, p3, 0.0);
        let end = eval_bezier(p0, p1, p2, p3, 1.0);
        assert!((start - p0).length() < 0.01);
        assert!((end - p3).length() < 0.01);
    }

    #[test]
    fn test_bezier_tangent_straight_line() {
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(100.0, 0.0);
        let p1 = p0 + (p3 - p0) / 3.0;
        let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

        let tang = bezier_tangent(p0, p1, p2, p3, 0.5);
        // Tangent of a straight line should point along the x-axis.
        let normalised = tang.normalize();
        assert!((normalised.x - 1.0).abs() < 0.01);
        assert!(normalised.y.abs() < 0.01);
    }

    #[test]
    fn test_build_avenue_marking_mesh_not_empty() {
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(100.0, 0.0);
        let p1 = p0 + (p3 - p0) / 3.0;
        let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

        let mesh = build_lane_marking_mesh(
            p0,
            p1,
            p2,
            p3,
            RoadType::Avenue,
            road_half_width(RoadType::Avenue),
            100.0,
            0.0,
            0.0,
        );

        // Avenue should produce double yellow center lines.
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("mesh should have positions");
        assert!(
            positions.len() > 0,
            "avenue marking mesh should have vertices"
        );
    }

    #[test]
    fn test_build_boulevard_marking_mesh_not_empty() {
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(200.0, 0.0);
        let p1 = p0 + (p3 - p0) / 3.0;
        let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

        let mesh = build_lane_marking_mesh(
            p0,
            p1,
            p2,
            p3,
            RoadType::Boulevard,
            road_half_width(RoadType::Boulevard),
            200.0,
            0.0,
            0.0,
        );

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("mesh should have positions");
        assert!(
            positions.len() > 0,
            "boulevard marking mesh should have vertices"
        );
    }

    #[test]
    fn test_build_highway_marking_mesh_not_empty() {
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(300.0, 0.0);
        let p1 = p0 + (p3 - p0) / 3.0;
        let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

        let mesh = build_lane_marking_mesh(
            p0,
            p1,
            p2,
            p3,
            RoadType::Highway,
            road_half_width(RoadType::Highway),
            300.0,
            0.0,
            0.0,
        );

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("mesh should have positions");
        assert!(
            positions.len() > 0,
            "highway marking mesh should have vertices"
        );
    }

    #[test]
    fn test_local_road_produces_no_marking_mesh() {
        // Local roads should not produce any marking geometry.
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(100.0, 0.0);
        let p1 = p0 + (p3 - p0) / 3.0;
        let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

        let mesh = build_lane_marking_mesh(
            p0,
            p1,
            p2,
            p3,
            RoadType::Local,
            road_half_width(RoadType::Local),
            100.0,
            0.0,
            0.0,
        );

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("mesh should have positions");
        assert_eq!(
            positions.len(),
            0,
            "local road marking mesh should be empty"
        );
    }

    #[test]
    fn test_path_produces_no_marking_mesh() {
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(50.0, 0.0);
        let p1 = p0 + (p3 - p0) / 3.0;
        let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

        let mesh = build_lane_marking_mesh(
            p0,
            p1,
            p2,
            p3,
            RoadType::Path,
            road_half_width(RoadType::Path),
            50.0,
            0.0,
            0.0,
        );

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("mesh should have positions");
        assert_eq!(positions.len(), 0, "path marking mesh should be empty");
    }

    #[test]
    fn test_trim_reduces_vertex_count() {
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(200.0, 0.0);
        let p1 = p0 + (p3 - p0) / 3.0;
        let p2 = p0 + (p3 - p0) * 2.0 / 3.0;
        let hw = road_half_width(RoadType::Highway);

        let mesh_no_trim =
            build_lane_marking_mesh(p0, p1, p2, p3, RoadType::Highway, hw, 200.0, 0.0, 0.0);
        let mesh_trimmed = build_lane_marking_mesh(
            p0,
            p1,
            p2,
            p3,
            RoadType::Highway,
            hw,
            200.0,
            hw * 1.2,
            hw * 1.2,
        );

        let count_no_trim = mesh_no_trim
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .map(|a| a.len())
            .unwrap_or(0);
        let count_trimmed = mesh_trimmed
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .map(|a| a.len())
            .unwrap_or(0);

        assert!(
            count_trimmed <= count_no_trim,
            "trimmed mesh should have <= vertices ({count_trimmed} vs {count_no_trim})"
        );
    }
}

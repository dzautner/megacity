//! Crosswalk and stop-line rendering at road intersections.
//!
//! Spawns overlay geometry at junction nodes where two or more road segments
//! meet.  Each approach arm gets a white stop line and a crosswalk pattern
//! (alternating white bars perpendicular to the road direction).
//!
//! Road types that skip intersection markings: [`RoadType::Path`].

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;

use simulation::grid::RoadType;
use simulation::road_segments::{RoadSegmentStore, SegmentNodeId};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker for intersection marking overlay entities.
#[derive(Component)]
pub struct IntersectionMarkingMesh {
    pub node_id: SegmentNodeId,
}

/// Local state tracking which node generations we have already spawned.
#[derive(Default)]
pub struct IntersectionMarkingSyncState {
    generation: usize,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Y-height slightly above the intersection disc surface.
const Y_MARKING: f32 = 0.065;

/// White colour for crosswalk bars and stop lines.
const CROSSWALK_WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.6];

/// Stop line colour (slightly more opaque).
const STOP_LINE_WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.7];

/// Width of each crosswalk bar.
const BAR_WIDTH: f32 = 0.6;

/// Gap between crosswalk bars.
const BAR_GAP: f32 = 0.5;

/// Stop line thickness (along road direction).
const STOP_LINE_THICKNESS: f32 = 0.35;

/// Offset from the intersection centre where stop lines begin.
const STOP_LINE_INSET: f32 = 1.0;

// ---------------------------------------------------------------------------
// Road half-width helper (mirrors lane_markings::line_primitives)
// ---------------------------------------------------------------------------

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
// System
// ---------------------------------------------------------------------------

/// Synchronises crosswalk / stop-line overlay meshes at junctions.
#[allow(clippy::too_many_arguments)]
pub fn sync_intersection_markings(
    store: Res<RoadSegmentStore>,
    existing: Query<(Entity, &IntersectionMarkingMesh)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut local_state: Local<IntersectionMarkingSyncState>,
) {
    if !store.is_changed() {
        return;
    }

    let generation = store.nodes.len() + store.segments.len();
    if generation == local_state.generation && generation != 0 {
        return;
    }
    local_state.generation = generation;

    // Collect existing node IDs to avoid duplicates.
    let existing_ids: Vec<(Entity, SegmentNodeId)> =
        existing.iter().map(|(e, m)| (e, m.node_id)).collect();

    // Despawn meshes whose nodes are gone.
    for (entity, nid) in &existing_ids {
        if !store.nodes.iter().any(|n| n.id == *nid) {
            commands.entity(*entity).despawn();
        }
    }

    let marking_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.5,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    for node in &store.nodes {
        // Only render at junctions (2+ connected segments).
        if node.connected_segments.len() < 2 {
            continue;
        }

        // Skip if already spawned.
        if existing_ids.iter().any(|(_, nid)| *nid == node.id) {
            continue;
        }

        // Collect approach directions and half-widths for each arm.
        let arms: Vec<(Vec2, f32)> = node
            .connected_segments
            .iter()
            .filter_map(|&seg_id| {
                let seg = store.get_segment(seg_id)?;
                // Skip paths — they don't get intersection markings.
                if seg.road_type == RoadType::Path {
                    return None;
                }
                let hw = road_half_width(seg.road_type);
                // Direction pointing away from the node along the segment.
                let dir = approach_direction(seg, node.id);
                Some((dir, hw))
            })
            .collect();

        if arms.is_empty() {
            continue;
        }

        let mesh = build_intersection_marking_mesh(node.position, &arms);

        commands.spawn((
            IntersectionMarkingMesh { node_id: node.id },
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(marking_material.clone()),
            Transform::IDENTITY,
        ));
    }
}

/// Compute the direction from `node_id` outward along `segment`.
fn approach_direction(
    segment: &simulation::road_segments::RoadSegment,
    node_id: SegmentNodeId,
) -> Vec2 {
    if segment.start_node == node_id {
        // Node is the start — direction is start -> end (p0 -> p3).
        let tangent = segment.tangent(0.0);
        tangent.normalize_or_zero()
    } else {
        // Node is the end — direction is end -> start (reverse tangent at t=1).
        let tangent = segment.tangent(1.0);
        (-tangent).normalize_or_zero()
    }
}

// ---------------------------------------------------------------------------
// Mesh construction
// ---------------------------------------------------------------------------

/// Build crosswalk bars and stop lines for all arms of a junction.
fn build_intersection_marking_mesh(center: Vec2, arms: &[(Vec2, f32)]) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for &(dir, hw) in arms {
        let perp = Vec2::new(-dir.y, dir.x);

        // --- Stop line ---
        // Placed perpendicular to the approach direction, offset from centre.
        let stop_center = center + dir * (hw * 0.9 + STOP_LINE_INSET);
        emit_quad(
            &mut positions,
            &mut normals,
            &mut colors,
            &mut uvs,
            &mut indices,
            stop_center,
            perp,
            dir,
            hw * 0.85,
            STOP_LINE_THICKNESS,
            STOP_LINE_WHITE,
        );

        // --- Crosswalk bars ---
        // Crosswalk sits between the stop line and the intersection centre.
        let crosswalk_start = hw * 0.9 + STOP_LINE_INSET + STOP_LINE_THICKNESS + 0.3;
        let crosswalk_span = hw * 0.7;
        let bar_count = ((crosswalk_span / (BAR_WIDTH + BAR_GAP)).floor() as usize).max(2);

        for b in 0..bar_count {
            let offset_along = crosswalk_start + b as f32 * (BAR_WIDTH + BAR_GAP);
            let bar_center = center + dir * offset_along;
            emit_quad(
                &mut positions,
                &mut normals,
                &mut colors,
                &mut uvs,
                &mut indices,
                bar_center,
                perp,
                dir,
                hw * 0.75,
                BAR_WIDTH * 0.5,
                CROSSWALK_WHITE,
            );
        }
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

/// Emit a single axis-aligned quad (two triangles) at `center`.
///
/// `lateral` is the cross-road axis, `longitudinal` is along-road.
/// `half_lateral` is half the bar's lateral extent.
/// `half_longitudinal` is half the bar's extent along the road direction.
#[allow(clippy::too_many_arguments)]
fn emit_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec2,
    lateral: Vec2,
    longitudinal: Vec2,
    half_lateral: f32,
    half_longitudinal: f32,
    color: [f32; 4],
) {
    let vi = positions.len() as u32;
    let tl = center - lateral * half_lateral - longitudinal * half_longitudinal;
    let tr = center + lateral * half_lateral - longitudinal * half_longitudinal;
    let br = center + lateral * half_lateral + longitudinal * half_longitudinal;
    let bl = center - lateral * half_lateral + longitudinal * half_longitudinal;

    positions.push([tl.x, Y_MARKING, tl.y]);
    positions.push([tr.x, Y_MARKING, tr.y]);
    positions.push([br.x, Y_MARKING, br.y]);
    positions.push([bl.x, Y_MARKING, bl.y]);
    normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
    colors.extend_from_slice(&[color; 4]);
    uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
    indices.push(vi);
    indices.push(vi + 2);
    indices.push(vi + 1);
    indices.push(vi);
    indices.push(vi + 3);
    indices.push(vi + 2);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approach_direction_start_node() {
        use simulation::road_segments::{RoadSegment, SegmentId, SegmentNodeId};
        let seg = RoadSegment {
            id: SegmentId(0),
            start_node: SegmentNodeId(1),
            end_node: SegmentNodeId(2),
            p0: Vec2::new(0.0, 0.0),
            p1: Vec2::new(33.0, 0.0),
            p2: Vec2::new(66.0, 0.0),
            p3: Vec2::new(100.0, 0.0),
            road_type: RoadType::Local,
            arc_length: 100.0,
            rasterized_cells: vec![],
        };
        let dir = approach_direction(&seg, SegmentNodeId(1));
        assert!((dir.x - 1.0).abs() < 0.1, "should point along +x");
        assert!(dir.y.abs() < 0.1);
    }

    #[test]
    fn test_approach_direction_end_node() {
        use simulation::road_segments::{RoadSegment, SegmentId, SegmentNodeId};
        let seg = RoadSegment {
            id: SegmentId(0),
            start_node: SegmentNodeId(1),
            end_node: SegmentNodeId(2),
            p0: Vec2::new(0.0, 0.0),
            p1: Vec2::new(33.0, 0.0),
            p2: Vec2::new(66.0, 0.0),
            p3: Vec2::new(100.0, 0.0),
            road_type: RoadType::Avenue,
            arc_length: 100.0,
            rasterized_cells: vec![],
        };
        let dir = approach_direction(&seg, SegmentNodeId(2));
        // End node direction should point away from the segment (toward -x).
        assert!((dir.x - (-1.0)).abs() < 0.1, "should point along -x");
        assert!(dir.y.abs() < 0.1);
    }

    #[test]
    fn test_build_intersection_mesh_two_arms() {
        let center = Vec2::new(50.0, 50.0);
        let arms = vec![
            (Vec2::new(1.0, 0.0), 4.0),  // east arm, Local width
            (Vec2::new(0.0, 1.0), 4.0),  // north arm, Local width
        ];
        let mesh = build_intersection_marking_mesh(center, &arms);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("should have positions");
        // Two arms should produce stop lines + crosswalk bars.
        assert!(
            positions.len() >= 8,
            "two-arm intersection should have at least 8 vertices, got {}",
            positions.len()
        );
    }

    #[test]
    fn test_empty_arms_produce_empty_mesh() {
        let center = Vec2::new(0.0, 0.0);
        let arms: Vec<(Vec2, f32)> = vec![];
        let mesh = build_intersection_marking_mesh(center, &arms);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("should have positions");
        assert_eq!(positions.len(), 0, "no arms should produce empty mesh");
    }
}

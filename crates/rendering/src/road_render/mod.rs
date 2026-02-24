mod bezier;
mod intersection;
mod lane_markings;
mod tessellation;

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use simulation::grid::{RoadType, WorldGrid};
use simulation::road_segments::{RoadSegmentStore, SegmentId, SegmentNodeId};

use intersection::build_intersection_disc;
use tessellation::tessellate_road_segment;

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
pub struct RoadIntersectionMesh {
    pub node_id: SegmentNodeId,
}

/// Synchronize road segment meshes with the RoadSegmentStore.
/// Spawns meshes for new segments, despawns for removed ones.
/// Also generates intersection fill discs where segments meet.
#[allow(clippy::too_many_arguments)]
pub fn sync_road_segment_meshes(
    mut store: ResMut<RoadSegmentStore>,
    grid: Res<WorldGrid>,
    existing: Query<(Entity, &RoadSegmentMesh)>,
    existing_intersections: Query<(Entity, &RoadIntersectionMesh)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut local_count: Local<LastSyncedCount>,
) {
    if !store.is_changed() {
        return;
    }

    // Build HashMap of existing segment meshes for O(1) lookup
    let existing_map: HashMap<SegmentId, Entity> =
        existing.iter().map(|(e, m)| (m.segment_id, e)).collect();

    // Build HashSet of store segment IDs for O(1) membership checks
    let store_ids: HashSet<SegmentId> = store.segments.iter().map(|s| s.id).collect();

    // Track which node IDs need intersection mesh rebuilds
    let mut dirty_nodes: HashSet<SegmentNodeId> = HashSet::new();

    // Collect node IDs recorded during segment removal (fixes #1239).
    if !store.removed_segment_endpoints.is_empty() {
        for node_id in store.drain_removed_endpoints() {
            dirty_nodes.insert(node_id);
        }
    }

    // Despawn meshes for segments that no longer exist
    for (entity, seg_mesh) in &existing {
        if !store_ids.contains(&seg_mesh.segment_id) {
            commands.entity(entity).despawn();
        }
    }

    // Helper: check if a node is a junction (2+ connected segments)
    let is_junction = |node_id: SegmentNodeId| -> bool {
        store
            .get_node(node_id)
            .map(|n| n.connected_segments.len() >= 2)
            .unwrap_or(false)
    };

    // Spawn meshes for new segments
    for segment in &store.segments {
        if existing_map.contains_key(&segment.id) {
            continue;
        }

        dirty_nodes.insert(segment.start_node);
        dirty_nodes.insert(segment.end_node);

        let road_half_w: f32 = match segment.road_type {
            RoadType::Path => 1.5,
            RoadType::OneWay => 3.0,
            RoadType::Local => 4.0,
            RoadType::Avenue => 6.0,
            RoadType::Boulevard => 8.0,
            RoadType::Highway => 10.0,
        };
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

        let mesh = tessellate_road_segment(
            &segment.p0,
            &segment.p1,
            &segment.p2,
            &segment.p3,
            segment.road_type,
            segment.arc_length,
            trim_start,
            trim_end,
            &grid,
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

    // Only rebuild intersection meshes for dirty nodes
    if !dirty_nodes.is_empty() {
        for (entity, intersection_mesh) in &existing_intersections {
            if dirty_nodes.contains(&intersection_mesh.node_id) {
                commands.entity(entity).despawn();
            }
        }

        let intersection_material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.9,
            ..default()
        });

        for node in &store.nodes {
            if !dirty_nodes.contains(&node.id) {
                continue;
            }

            if node.connected_segments.len() < 2 {
                continue;
            }

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
                    for j in 0..4 {
                        avg_asphalt[j] += asph[j];
                    }
                    count += 1.0;
                }
            }

            if count < 1.0 {
                continue;
            }
            for val in &mut avg_asphalt {
                *val /= count;
            }

            let sidewalk_color: [f32; 4] = [0.58, 0.56, 0.53, 1.0];

            let mesh = build_intersection_disc(
                node.position,
                max_total_hw,
                max_road_hw,
                sidewalk_color,
                avg_asphalt,
                &grid,
            );

            commands.spawn((
                RoadIntersectionMesh { node_id: node.id },
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(intersection_material.clone()),
                Transform::IDENTITY,
            ));
        }
    }

    local_count.0 = store.segments.len();
}

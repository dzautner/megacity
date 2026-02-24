//! Road lane marking decals rendered as separate overlay meshes.
//!
//! Spawns thin geometry slightly above the road surface for center lines,
//! lane dividers, and edge markings.  Marking style varies per [`RoadType`]:
//!
//! | Road type  | Center         | Lane dividers | Edge lines |
//! |------------|----------------|---------------|------------|
//! | Path       | none           | none          | none       |
//! | Local      | dashed white   | none          | none       |
//! | OneWay     | dashed white   | none          | none       |
//! | Avenue     | double yellow  | none          | solid white|
//! | Boulevard  | yellow center  | dashed white  | solid white|
//! | Highway    | barrier        | dashed white  | solid white|

mod line_primitives;
mod mesh_builder;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

use simulation::grid::RoadType;
use simulation::road_segments::{RoadSegmentStore, SegmentId, SegmentNodeId};

use self::line_primitives::road_half_width;
use self::mesh_builder::build_lane_marking_mesh;

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
// Geometry constants (shared across sub-modules via `pub(super)`)
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

        // Paths have no lane markings.
        if segment.road_type == RoadType::Path {
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

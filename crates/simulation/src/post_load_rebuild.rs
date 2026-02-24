//! SAVE-026: Post-load derived state rebuild system.
//!
//! After loading a save file, several derived-state resources may be stale or
//! inconsistent with the freshly restored source-of-truth resources. This
//! module provides a one-shot system that explicitly rebuilds all derived state:
//!
//! - **CSR road graph**: rebuilt from `RoadNetwork` (+ one-way constraints)
//! - **Service coverage grid**: marked dirty so the next tick recalculates it
//! - **Traffic grid**: zeroed out (stale density data is meaningless after load)
//! - **Spatial grid**: cleared (rebuilt every frame by the LOD system)

use bevy::prelude::*;

use crate::happiness::ServiceCoverageGrid;
use crate::oneway::OneWayDirectionMap;
use crate::road_graph_csr::CsrGraph;
use crate::road_segments::RoadSegmentStore;
use crate::roads::{RoadNetwork, RoadNode};
use crate::spatial_grid::SpatialGrid;
use crate::traffic::TrafficGrid;

/// Marker resource inserted after a load completes, signalling the rebuild
/// system to reconstruct all derived state on the next `FixedUpdate` tick.
#[derive(Resource, Default)]
pub struct PostLoadRebuildPending;

pub struct PostLoadRebuildPlugin;

impl Plugin for PostLoadRebuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            post_load_rebuild_derived_state
                .run_if(resource_exists::<PostLoadRebuildPending>)
                .in_set(crate::SimulationSet::PreSim),
        );
    }
}

/// One-shot system that rebuilds all derived state after a save/load cycle.
///
/// This runs in `PreSim` (before gameplay systems) to ensure derived state
/// is consistent before any simulation logic reads it.
#[allow(clippy::too_many_arguments)]
fn post_load_rebuild_derived_state(
    mut commands: Commands,
    roads: Res<RoadNetwork>,
    segments: Res<RoadSegmentStore>,
    oneway_map: Res<OneWayDirectionMap>,
    mut csr: ResMut<CsrGraph>,
    mut coverage: ResMut<ServiceCoverageGrid>,
    mut traffic: ResMut<TrafficGrid>,
    mut spatial: ResMut<SpatialGrid>,
) {
    info!("Post-load rebuild: reconstructing derived state...");

    // 1. Rebuild CSR road graph from RoadNetwork, respecting one-way constraints.
    rebuild_csr_graph(&roads, &segments, &oneway_map, &mut csr);

    // 2. Mark service coverage grid as dirty so it recalculates on the next tick.
    coverage.dirty = true;

    // 3. Zero out traffic grid (stale density data from previous session).
    traffic.clear();

    // 4. Clear spatial grid (rebuilt every frame by the LOD update_spatial_grid system).
    spatial.clear();

    info!(
        "Post-load rebuild complete: CSR graph has {} nodes / {} edges, \
         traffic grid zeroed, service coverage marked dirty, spatial grid cleared",
        csr.node_count(),
        csr.edge_count(),
    );

    // Remove the flag so this system doesn't run again until the next load.
    commands.remove_resource::<PostLoadRebuildPending>();
}

/// Rebuild the CSR graph from the road network, incorporating one-way
/// direction constraints from the `OneWayDirectionMap`.
fn rebuild_csr_graph(
    roads: &RoadNetwork,
    segments: &RoadSegmentStore,
    oneway_map: &OneWayDirectionMap,
    csr: &mut CsrGraph,
) {
    if oneway_map.directions.is_empty() {
        // No one-way directions: use the standard builder.
        *csr = CsrGraph::from_road_network(roads);
    } else {
        // Build blocked edge set from one-way segment directions.
        let mut blocked_edges: std::collections::HashSet<(RoadNode, RoadNode)> =
            std::collections::HashSet::new();

        for segment in &segments.segments {
            let Some(direction) = oneway_map.get(segment.id) else {
                continue;
            };

            let cells = &segment.rasterized_cells;
            if cells.len() < 2 {
                continue;
            }

            for window in cells.windows(2) {
                let a = RoadNode(window[0].0, window[0].1);
                let b = RoadNode(window[1].0, window[1].1);

                match direction {
                    crate::oneway::OneWayDirection::Forward => {
                        blocked_edges.insert((b, a));
                    }
                    crate::oneway::OneWayDirection::Reverse => {
                        blocked_edges.insert((a, b));
                    }
                }
            }
        }

        *csr = CsrGraph::from_road_network_filtered(roads, &blocked_edges);
    }
}

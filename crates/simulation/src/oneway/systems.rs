use bevy::prelude::*;

use crate::road_graph_csr::CsrGraph;
use crate::road_segments::RoadSegmentStore;
use crate::roads::{RoadNetwork, RoadNode};

use super::types::{OneWayDirection, OneWayDirectionMap, ToggleOneWayEvent};

/// Handle toggle events by cycling through direction states.
pub fn handle_toggle_oneway(
    mut events: EventReader<ToggleOneWayEvent>,
    mut oneway_map: ResMut<OneWayDirectionMap>,
) {
    for event in events.read() {
        oneway_map.toggle(event.segment_id);
    }
}

/// Rebuild the CSR graph incorporating one-way direction constraints.
///
/// For bidirectional segments, edges go both ways (A->B and B->A).
/// For one-way Forward segments, only start_node->end_node edges exist.
/// For one-way Reverse segments, only end_node->start_node edges exist.
///
/// This replaces the default `rebuild_csr_on_road_change` when one-way
/// directions are active.
pub fn rebuild_csr_with_oneway(
    roads: Res<RoadNetwork>,
    segments: Res<RoadSegmentStore>,
    oneway_map: Res<OneWayDirectionMap>,
    mut csr: ResMut<CsrGraph>,
    mut last_gen: Local<u32>,
) {
    // Only rebuild if something changed
    if !roads.is_changed() && *last_gen == oneway_map.generation {
        return;
    }
    *last_gen = oneway_map.generation;

    // If no one-way directions exist, just use the standard builder
    if oneway_map.directions.is_empty() {
        *csr = CsrGraph::from_road_network(&roads);
        return;
    }

    // Build a set of directed edge restrictions from segments
    // For each one-way segment, identify which grid cells are rasterized and
    // restrict edges between consecutive rasterized cells to the allowed direction.
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

        // For each pair of consecutive rasterized cells, block the reverse direction
        for window in cells.windows(2) {
            let a = RoadNode(window[0].0, window[0].1);
            let b = RoadNode(window[1].0, window[1].1);

            match direction {
                OneWayDirection::Forward => {
                    // Allow A->B, block B->A
                    blocked_edges.insert((b, a));
                }
                OneWayDirection::Reverse => {
                    // Allow B->A, block A->B
                    blocked_edges.insert((a, b));
                }
            }
        }
    }

    // Build CSR graph from road network, filtering out blocked edges
    *csr = CsrGraph::from_road_network_filtered(&roads, &blocked_edges);
}

impl CsrGraph {
    /// Build CSR graph from road network, excluding blocked directed edges.
    ///
    /// **Determinism**: Edge indices are sorted per node, matching the
    /// deterministic behavior of `from_road_network`.
    pub fn from_road_network_filtered(
        network: &RoadNetwork,
        blocked: &std::collections::HashSet<(RoadNode, RoadNode)>,
    ) -> Self {
        let mut nodes: Vec<RoadNode> = network.edges.keys().copied().collect();
        nodes.sort_by(|a, b| (a.1, a.0).cmp(&(b.1, b.0)));

        let node_index: std::collections::HashMap<RoadNode, u32> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (*n, i as u32))
            .collect();

        let mut node_offsets = Vec::with_capacity(nodes.len() + 1);
        let mut edges = Vec::new();
        let mut weights = Vec::new();

        for node in &nodes {
            node_offsets.push(edges.len() as u32);
            if let Some(neighbors) = network.edges.get(node) {
                // Collect, filter blocked edges, and sort for deterministic order
                let mut neighbor_indices: Vec<u32> = neighbors
                    .iter()
                    .filter(|neighbor| !blocked.contains(&(*node, **neighbor)))
                    .filter_map(|neighbor| node_index.get(neighbor).copied())
                    .collect();
                neighbor_indices.sort_unstable();
                for idx in neighbor_indices {
                    edges.push(idx);
                    weights.push(1);
                }
            }
        }
        node_offsets.push(edges.len() as u32);

        Self {
            nodes,
            node_offsets,
            edges,
            weights,
        }
    }
}

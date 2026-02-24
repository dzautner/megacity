use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::roads::{RoadNetwork, RoadNode};

/// Compressed Sparse Row graph for cache-friendly traversal.
/// Stores the road network in a flat array format optimized for iteration.
///
/// **Determinism**: Node order is sorted by `(y, x)`. Edge lists per node are
/// sorted by neighbor index, ensuring identical CSR output for identical input
/// regardless of insertion order. The source `RoadNetwork` uses `BTreeMap`/`BTreeSet`
/// for deterministic iteration, and this builder additionally sorts edge indices
/// to guarantee the CSR layout is fully deterministic.
#[derive(Resource, Default, Serialize, Deserialize)]
pub struct CsrGraph {
    /// Sorted list of all nodes (by (y, x) for spatial locality)
    pub nodes: Vec<RoadNode>,
    /// node_offsets[i] is the start index in `edges` for node i.
    /// node_offsets[i+1] - node_offsets[i] = number of neighbors of node i.
    pub node_offsets: Vec<u32>,
    /// Flat array of neighbor indices (indices into `nodes`), sorted per node
    pub edges: Vec<u32>,
    /// Edge weights (same length as edges)
    pub weights: Vec<u32>,
}

impl CsrGraph {
    pub fn from_road_network(network: &RoadNetwork) -> Self {
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
                // Collect and sort neighbor indices for deterministic edge order
                // within each node's adjacency list.
                let mut neighbor_indices: Vec<u32> = neighbors
                    .iter()
                    .filter_map(|neighbor| node_index.get(neighbor).copied())
                    .collect();
                neighbor_indices.sort_unstable();
                for idx in neighbor_indices {
                    edges.push(idx);
                    weights.push(1); // default weight
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

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn neighbors(&self, node_idx: u32) -> &[u32] {
        let start = self.node_offsets[node_idx as usize] as usize;
        let end = self.node_offsets[node_idx as usize + 1] as usize;
        &self.edges[start..end]
    }

    pub fn neighbor_weights(&self, node_idx: u32) -> impl Iterator<Item = (u32, u32)> + '_ {
        let start = self.node_offsets[node_idx as usize] as usize;
        let end = self.node_offsets[node_idx as usize + 1] as usize;
        self.edges[start..end]
            .iter()
            .zip(self.weights[start..end].iter())
            .map(|(&e, &w)| (e, w))
    }

    pub fn find_node_index(&self, node: &RoadNode) -> Option<u32> {
        self.nodes
            .binary_search_by(|n| (n.1, n.0).cmp(&(node.1, node.0)))
            .ok()
            .map(|i| i as u32)
    }
}

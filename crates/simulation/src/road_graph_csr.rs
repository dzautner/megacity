use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::roads::{RoadNetwork, RoadNode};

/// Compressed Sparse Row graph for cache-friendly traversal.
/// Stores the road network in a flat array format optimized for iteration.
#[derive(Resource, Default, Serialize, Deserialize)]
pub struct CsrGraph {
    /// Sorted list of all nodes
    pub nodes: Vec<RoadNode>,
    /// node_offsets[i] is the start index in `edges` for node i.
    /// node_offsets[i+1] - node_offsets[i] = number of neighbors of node i.
    pub node_offsets: Vec<u32>,
    /// Flat array of neighbor indices (indices into `nodes`)
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
                for neighbor in neighbors {
                    if let Some(&idx) = node_index.get(neighbor) {
                        edges.push(idx);
                        weights.push(1); // default weight
                    }
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

/// Find path using A* on CSR graph
pub fn csr_find_path(csr: &CsrGraph, start: RoadNode, goal: RoadNode) -> Option<Vec<RoadNode>> {
    let start_idx = csr.find_node_index(&start)?;
    let goal_idx = csr.find_node_index(&goal)?;

    let goal_node = csr.nodes[goal_idx as usize];

    let result = pathfinding::prelude::astar(
        &start_idx,
        |&idx| csr.neighbor_weights(idx),
        |&idx| {
            let node = csr.nodes[idx as usize];
            let dx = (node.0 as i32 - goal_node.0 as i32).unsigned_abs();
            let dy = (node.1 as i32 - goal_node.1 as i32).unsigned_abs();
            dx + dy
        },
        |&idx| idx == goal_idx,
    );

    result.map(|(path_indices, _cost)| {
        path_indices
            .into_iter()
            .map(|idx| csr.nodes[idx as usize])
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::WorldGrid;

    #[test]
    fn test_csr_produces_same_paths() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();

        // Build an L-shaped road
        for x in 5..=15 {
            network.place_road(&mut grid, x, 10);
        }
        for y in 10..=20 {
            network.place_road(&mut grid, 15, y);
        }

        let csr = CsrGraph::from_road_network(&network);

        // Verify A* on original network matches CSR
        let path_original =
            crate::pathfinding_sys::find_path(&network, RoadNode(5, 10), RoadNode(15, 20));
        let path_csr = csr_find_path(&csr, RoadNode(5, 10), RoadNode(15, 20));

        assert!(path_original.is_some());
        assert!(path_csr.is_some());

        // Both should find paths of the same length
        let len_orig = path_original.unwrap().len();
        let len_csr = path_csr.unwrap().len();
        assert_eq!(len_orig, len_csr);
    }

    #[test]
    fn test_csr_node_count() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();

        for x in 0..10 {
            network.place_road(&mut grid, x, 0);
        }

        let csr = CsrGraph::from_road_network(&network);
        assert_eq!(csr.node_count(), 10);
    }
}

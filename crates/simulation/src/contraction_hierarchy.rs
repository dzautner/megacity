use std::collections::BinaryHeap;
use std::cmp::Reverse;

use serde::{Deserialize, Serialize};

use crate::road_graph_csr::CsrGraph;
use crate::roads::RoadNode;

/// Pre-computed contraction hierarchy for fast O(log n) path queries.
/// Shortcuts are added during a preprocessing step when nodes are "contracted"
/// (removed from the graph with shortcut edges added to preserve shortest paths).
#[derive(Default, Serialize, Deserialize)]
pub struct ContractionHierarchy {
    /// Node ordering: rank[node_idx] = contraction order (higher = more important)
    pub rank: Vec<u32>,
    /// Forward graph: edges to higher-ranked nodes
    pub forward_edges: Vec<Vec<(u32, u32)>>, // (target_idx, weight)
    /// Backward graph: edges to higher-ranked nodes (for bidirectional search)
    pub backward_edges: Vec<Vec<(u32, u32)>>,
    /// The original CSR graph (for node lookups)
    pub node_map: Vec<RoadNode>,
}

impl ContractionHierarchy {
    /// Build CH from a CSR graph. Uses a simple node ordering based on degree.
    pub fn build(csr: &CsrGraph) -> Self {
        let n = csr.node_count();
        if n == 0 {
            return Self::default();
        }

        // Simple ordering: contract low-degree nodes first
        let mut node_degree: Vec<(usize, usize)> = (0..n)
            .map(|i| (csr.neighbors(i as u32).len(), i))
            .collect();
        node_degree.sort();

        let mut rank = vec![0u32; n];
        for (order, &(_, node_idx)) in node_degree.iter().enumerate() {
            rank[node_idx] = order as u32;
        }

        // Build upward graphs based on rank.
        // In an undirected graph, both forward and backward CH searches
        // explore edges going to higher-ranked nodes.
        let mut upward_edges: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n];

        for u in 0..n {
            for (v, w) in csr.neighbor_weights(u as u32) {
                if rank[v as usize] > rank[u] {
                    upward_edges[u].push((v, w));
                }
                // Symmetric: since undirected, also add from v perspective
                // (already handled when u=v in the outer loop)
            }
        }

        // Add shortcuts: for each contracted node, add edges between its higher-ranked neighbors
        for &(_, node_idx) in &node_degree {
            let higher_neighbors: Vec<(u32, u32)> = csr
                .neighbor_weights(node_idx as u32)
                .filter(|&(v, _)| rank[v as usize] > rank[node_idx])
                .collect();

            for i in 0..higher_neighbors.len() {
                for j in (i + 1)..higher_neighbors.len() {
                    let (u, wu) = higher_neighbors[i];
                    let (v, wv) = higher_neighbors[j];
                    let shortcut_weight = wu + wv;

                    if rank[v as usize] > rank[u as usize] {
                        upward_edges[u as usize].push((v, shortcut_weight));
                    } else if rank[u as usize] > rank[v as usize] {
                        upward_edges[v as usize].push((u, shortcut_weight));
                    }
                    // If same rank, add to both
                    else {
                        upward_edges[u as usize].push((v, shortcut_weight));
                        upward_edges[v as usize].push((u, shortcut_weight));
                    }
                }
            }
        }

        // For undirected graphs, forward and backward upward edges are the same
        let forward_edges = upward_edges.clone();
        let backward_edges = upward_edges;

        Self {
            rank,
            forward_edges,
            backward_edges,
            node_map: csr.nodes.clone(),
        }
    }

    /// Bidirectional Dijkstra on the CH
    pub fn find_path(&self, start: RoadNode, goal: RoadNode) -> Option<u32> {
        let n = self.node_map.len();
        if n == 0 {
            return None;
        }

        let start_idx = self.find_index(&start)?;
        let goal_idx = self.find_index(&goal)?;

        if start_idx == goal_idx {
            return Some(0);
        }

        // Forward Dijkstra from start (only upward edges)
        let mut dist_forward = vec![u32::MAX; n];
        let mut dist_backward = vec![u32::MAX; n];

        dist_forward[start_idx as usize] = 0;
        dist_backward[goal_idx as usize] = 0;

        let mut heap_forward: BinaryHeap<Reverse<(u32, u32)>> = BinaryHeap::new();
        let mut heap_backward: BinaryHeap<Reverse<(u32, u32)>> = BinaryHeap::new();

        heap_forward.push(Reverse((0, start_idx)));
        heap_backward.push(Reverse((0, goal_idx)));

        // Run forward search
        while let Some(Reverse((d, u))) = heap_forward.pop() {
            if d > dist_forward[u as usize] {
                continue;
            }
            for &(v, w) in &self.forward_edges[u as usize] {
                let new_d = d + w;
                if new_d < dist_forward[v as usize] {
                    dist_forward[v as usize] = new_d;
                    heap_forward.push(Reverse((new_d, v)));
                }
            }
        }

        // Run backward search
        while let Some(Reverse((d, u))) = heap_backward.pop() {
            if d > dist_backward[u as usize] {
                continue;
            }
            for &(v, w) in &self.backward_edges[u as usize] {
                let new_d = d + w;
                if new_d < dist_backward[v as usize] {
                    dist_backward[v as usize] = new_d;
                    heap_backward.push(Reverse((new_d, v)));
                }
            }
        }

        // Find best meeting point
        let mut best = u32::MAX;
        for i in 0..n {
            if dist_forward[i] != u32::MAX && dist_backward[i] != u32::MAX {
                let total = dist_forward[i].saturating_add(dist_backward[i]);
                best = best.min(total);
            }
        }

        if best == u32::MAX {
            None
        } else {
            Some(best)
        }
    }

    fn find_index(&self, node: &RoadNode) -> Option<u32> {
        self.node_map
            .iter()
            .position(|n| n == node)
            .map(|i| i as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::WorldGrid;
    use crate::road_graph_csr::CsrGraph;
    use crate::roads::RoadNetwork;

    #[test]
    fn test_ch_matches_astar() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();

        // Build a grid of roads
        for x in 5..=15 {
            network.place_road(&mut grid, x, 10);
        }
        for y in 10..=20 {
            network.place_road(&mut grid, 15, y);
        }

        let csr = CsrGraph::from_road_network(&network);
        let ch = ContractionHierarchy::build(&csr);

        // A* path length
        let astar_path = crate::pathfinding_sys::find_path(
            &network,
            RoadNode(5, 10),
            RoadNode(15, 20),
        );
        let astar_cost = astar_path.map(|p| (p.len() - 1) as u32);

        // CH distance
        let ch_cost = ch.find_path(RoadNode(5, 10), RoadNode(15, 20));

        assert!(astar_cost.is_some());
        assert!(ch_cost.is_some());

        // CH should find a path with cost <= A* (shortcuts may make it same or less)
        // In an unweighted graph the costs should match
        assert_eq!(astar_cost.unwrap(), ch_cost.unwrap());
    }

    #[test]
    fn test_ch_no_path() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();

        network.place_road(&mut grid, 5, 10);
        network.place_road(&mut grid, 20, 10);

        let csr = CsrGraph::from_road_network(&network);
        let ch = ContractionHierarchy::build(&csr);

        let result = ch.find_path(RoadNode(5, 10), RoadNode(20, 10));
        assert!(result.is_none());
    }
}

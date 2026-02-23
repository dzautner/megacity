use crate::grid::RoadType;
use crate::roads::RoadNode;

use super::pathfinding::{bpr_travel_time, BPR_ALPHA, BPR_BETA};

// ---------------------------------------------------------------------------
// Async-friendly pathfinding data bundle
// ---------------------------------------------------------------------------

/// Self-contained pathfinding data for async tasks.
///
/// Bundles CSR graph topology with per-node road types and traffic density
/// so that A* can run without access to ECS resources. Designed to be shared
/// via `Arc` across multiple async tasks spawned in a single tick.
pub struct PathfindingData {
    /// Sorted list of all nodes (same as CsrGraph::nodes)
    pub nodes: Vec<RoadNode>,
    /// CSR row offsets
    pub node_offsets: Vec<u32>,
    /// CSR edge list (neighbor indices)
    pub edges: Vec<u32>,
    /// CSR edge weights
    pub weights: Vec<u32>,
    /// Road type for each CSR node (indexed by node position in `nodes`)
    pub node_road_types: Vec<RoadType>,
    /// Flat traffic density array (snapshot of TrafficGrid::density)
    pub traffic_density: Vec<u16>,
    /// Width of the traffic grid (for index calculation)
    pub traffic_width: usize,
}

impl Default for PathfindingData {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            node_offsets: vec![0],
            edges: Vec::new(),
            weights: Vec::new(),
            node_road_types: Vec::new(),
            traffic_density: Vec::new(),
            traffic_width: 256,
        }
    }
}

impl PathfindingData {
    /// Find node index by binary search (same algorithm as CsrGraph::find_node_index).
    fn find_node_index(&self, node: &RoadNode) -> Option<u32> {
        self.nodes
            .binary_search_by(|n| (n.1, n.0).cmp(&(node.1, node.0)))
            .ok()
            .map(|i| i as u32)
    }

    /// Get traffic density at grid position.
    #[inline]
    fn traffic_at(&self, x: usize, y: usize) -> u16 {
        self.traffic_density[y * self.traffic_width + x]
    }

    /// Run traffic-aware A* pathfinding on the bundled data.
    ///
    /// This is the async-safe equivalent of `csr_find_path_with_traffic`,
    /// using pre-extracted road types instead of WorldGrid lookups.
    pub fn find_path_with_traffic(&self, start: RoadNode, goal: RoadNode) -> Option<Vec<RoadNode>> {
        let start_idx = self.find_node_index(&start)?;
        let goal_idx = self.find_node_index(&goal)?;

        let goal_node = self.nodes[goal_idx as usize];

        let result = pathfinding::prelude::astar(
            &start_idx,
            |&idx| {
                let current_node = self.nodes[idx as usize];
                let start_offset = self.node_offsets[idx as usize] as usize;
                let end_offset = self.node_offsets[idx as usize + 1] as usize;

                (start_offset..end_offset).map(move |edge_pos| {
                    let neighbor_idx = self.edges[edge_pos];
                    let neighbor_node = self.nodes[neighbor_idx as usize];

                    // Get road type from pre-extracted per-node data
                    let road_type = self.node_road_types[neighbor_idx as usize];

                    // Free-flow time: distance / speed
                    let dx = (neighbor_node.0 as f64 - current_node.0 as f64).abs();
                    let dy = (neighbor_node.1 as f64 - current_node.1 as f64).abs();
                    let distance = (dx * dx + dy * dy).sqrt().max(1.0);
                    let speed = road_type.speed() as f64;
                    let free_flow_time = distance / speed * 100.0;

                    // Get traffic volume from snapshot
                    let volume = self.traffic_at(neighbor_node.0, neighbor_node.1) as f64;
                    let capacity = road_type.capacity() as f64;

                    // BPR travel time
                    let travel_time =
                        bpr_travel_time(free_flow_time, volume, capacity, BPR_ALPHA, BPR_BETA);

                    (neighbor_idx, travel_time as u32 + 1)
                })
            },
            |&idx| {
                let node = self.nodes[idx as usize];
                let dx = (node.0 as i32 - goal_node.0 as i32).unsigned_abs();
                let dy = (node.1 as i32 - goal_node.1 as i32).unsigned_abs();
                dx + dy
            },
            |&idx| idx == goal_idx,
        );

        result.map(|(path_indices, _cost)| {
            path_indices
                .into_iter()
                .map(|idx| self.nodes[idx as usize])
                .collect()
        })
    }
}

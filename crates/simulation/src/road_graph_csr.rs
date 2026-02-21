use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grid::{RoadType, WorldGrid};
use crate::roads::{RoadNetwork, RoadNode};
use crate::traffic::TrafficGrid;

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

/// BPR (Bureau of Public Roads) travel time function.
///
/// Models congestion nonlinearly:
///   travel_time = free_flow_time * (1 + alpha * (volume / capacity)^beta)
///
/// Standard parameters: alpha = 0.15, beta = 4.0
///
/// - `free_flow_time`: travel time with zero congestion (based on distance / speed)
/// - `volume`: current traffic volume on the edge
/// - `capacity`: maximum throughput of the edge (from RoadType::capacity())
/// - `alpha`: scaling factor for congestion delay (standard: 0.15)
/// - `beta`: exponent controlling nonlinearity (standard: 4.0)
pub fn bpr_travel_time(
    free_flow_time: f64,
    volume: f64,
    capacity: f64,
    alpha: f64,
    beta: f64,
) -> f64 {
    if capacity <= 0.0 {
        return free_flow_time;
    }
    let vc_ratio = volume / capacity;
    free_flow_time * (1.0 + alpha * vc_ratio.powf(beta))
}

/// Default BPR alpha parameter (standard value from Bureau of Public Roads).
pub const BPR_ALPHA: f64 = 0.15;

/// Default BPR beta parameter (standard value from Bureau of Public Roads).
pub const BPR_BETA: f64 = 4.0;

/// Find path using A* on CSR graph with BPR traffic-aware edge costs.
///
/// Uses the BPR function to compute edge travel times that account for
/// congestion. Higher traffic volumes cause nonlinearly increasing travel
/// times, encouraging route diversification.
#[allow(clippy::too_many_arguments)]
pub fn csr_find_path_with_traffic(
    csr: &CsrGraph,
    start: RoadNode,
    goal: RoadNode,
    grid: &WorldGrid,
    traffic: &TrafficGrid,
) -> Option<Vec<RoadNode>> {
    let start_idx = csr.find_node_index(&start)?;
    let goal_idx = csr.find_node_index(&goal)?;

    let goal_node = csr.nodes[goal_idx as usize];

    let result = pathfinding::prelude::astar(
        &start_idx,
        |&idx| {
            let current_node = csr.nodes[idx as usize];
            let start_offset = csr.node_offsets[idx as usize] as usize;
            let end_offset = csr.node_offsets[idx as usize + 1] as usize;

            (start_offset..end_offset).map(move |edge_pos| {
                let neighbor_idx = csr.edges[edge_pos];
                let neighbor_node = csr.nodes[neighbor_idx as usize];

                // Get road type at the neighbor cell for capacity info
                let road_type = grid.get(neighbor_node.0, neighbor_node.1).road_type;

                // Free-flow time: distance / speed (higher speed = lower time)
                let dx = (neighbor_node.0 as f64 - current_node.0 as f64).abs();
                let dy = (neighbor_node.1 as f64 - current_node.1 as f64).abs();
                let distance = (dx * dx + dy * dy).sqrt().max(1.0);
                let speed = road_type.speed() as f64;
                let free_flow_time = distance / speed * 100.0; // scale for integer costs

                // Get traffic volume at the neighbor cell
                let volume = traffic.get(neighbor_node.0, neighbor_node.1) as f64;
                let capacity = road_type.capacity() as f64;

                // BPR travel time
                let travel_time =
                    bpr_travel_time(free_flow_time, volume, capacity, BPR_ALPHA, BPR_BETA);

                (neighbor_idx, travel_time as u32 + 1) // +1 to ensure non-zero cost
            })
        },
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

    #[test]
    fn test_bpr_zero_volume() {
        // With zero traffic, travel time equals free-flow time
        let result = bpr_travel_time(10.0, 0.0, 100.0, BPR_ALPHA, BPR_BETA);
        assert!((result - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_bpr_at_capacity() {
        // At volume == capacity, v/c ratio = 1.0, so:
        // travel_time = free_flow * (1 + 0.15 * 1^4) = free_flow * 1.15
        let result = bpr_travel_time(10.0, 100.0, 100.0, BPR_ALPHA, BPR_BETA);
        let expected = 10.0 * 1.15;
        assert!((result - expected).abs() < 1e-9);
    }

    #[test]
    fn test_bpr_over_capacity() {
        // At 2x capacity, v/c ratio = 2.0:
        // travel_time = free_flow * (1 + 0.15 * 2^4) = free_flow * (1 + 0.15 * 16) = free_flow * 3.4
        let result = bpr_travel_time(10.0, 200.0, 100.0, BPR_ALPHA, BPR_BETA);
        let expected = 10.0 * 3.4;
        assert!((result - expected).abs() < 1e-9);
    }

    #[test]
    fn test_bpr_zero_capacity() {
        // Zero capacity should return free-flow time (avoid division by zero)
        let result = bpr_travel_time(10.0, 50.0, 0.0, BPR_ALPHA, BPR_BETA);
        assert!((result - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_bpr_increases_with_volume() {
        // Travel time should increase as volume increases
        let t1 = bpr_travel_time(10.0, 10.0, 100.0, BPR_ALPHA, BPR_BETA);
        let t2 = bpr_travel_time(10.0, 50.0, 100.0, BPR_ALPHA, BPR_BETA);
        let t3 = bpr_travel_time(10.0, 100.0, 100.0, BPR_ALPHA, BPR_BETA);
        assert!(t1 < t2);
        assert!(t2 < t3);
    }

    #[test]
    fn test_bpr_nonlinear_growth() {
        // BPR with beta=4 should grow much faster at high v/c ratios
        let t_half = bpr_travel_time(10.0, 50.0, 100.0, BPR_ALPHA, BPR_BETA);
        let t_full = bpr_travel_time(10.0, 100.0, 100.0, BPR_ALPHA, BPR_BETA);
        let t_double = bpr_travel_time(10.0, 200.0, 100.0, BPR_ALPHA, BPR_BETA);

        // Penalty at half capacity vs full should be much smaller than full vs double
        let penalty_half_to_full = t_full - t_half;
        let penalty_full_to_double = t_double - t_full;
        assert!(penalty_full_to_double > penalty_half_to_full * 4.0);
    }

    #[test]
    fn test_csr_find_path_with_traffic_no_congestion() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();
        let traffic = TrafficGrid::default();

        // Build a straight road
        for x in 5..=15 {
            network.place_road(&mut grid, x, 10);
        }

        let csr = CsrGraph::from_road_network(&network);

        // Should find a path with no traffic
        let path =
            csr_find_path_with_traffic(&csr, RoadNode(5, 10), RoadNode(15, 10), &grid, &traffic);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&RoadNode(5, 10)));
        assert_eq!(path.last(), Some(&RoadNode(15, 10)));
    }

    #[test]
    fn test_csr_find_path_with_traffic_avoids_congestion() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();
        let mut traffic = TrafficGrid::default();

        // Build two parallel routes: y=10 and y=12
        for x in 5..=15 {
            network.place_road(&mut grid, x, 10);
            network.place_road(&mut grid, x, 12);
        }
        // Connect them at both ends
        for y in 10..=12 {
            network.place_road(&mut grid, 5, y);
            network.place_road(&mut grid, 15, y);
        }

        // Add heavy congestion on the direct route (y=10)
        for x in 6..15 {
            traffic.set(x, 10, 100); // very congested
        }

        let csr = CsrGraph::from_road_network(&network);

        let path =
            csr_find_path_with_traffic(&csr, RoadNode(5, 10), RoadNode(15, 10), &grid, &traffic);
        assert!(path.is_some());
        let path = path.unwrap();

        // Path should route through y=12 to avoid congestion on y=10
        let uses_alternate = path.iter().any(|n| n.1 == 12);
        assert!(
            uses_alternate,
            "Path should avoid congested route and use y=12"
        );
    }

    #[test]
    fn test_road_type_capacity() {
        use crate::grid::RoadType;

        // Capacity should increase with road tier
        assert!(RoadType::Local.capacity() < RoadType::Avenue.capacity());
        assert!(RoadType::Avenue.capacity() < RoadType::Boulevard.capacity());
        assert!(RoadType::Boulevard.capacity() < RoadType::Highway.capacity());

        // Path should have lowest capacity
        assert!(RoadType::Path.capacity() < RoadType::Local.capacity());
    }

    #[test]
    fn test_pathfinding_data_matches_csr_with_traffic() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();
        let traffic = TrafficGrid::default();

        // Build a straight road
        for x in 5..=15 {
            network.place_road(&mut grid, x, 10);
        }

        let csr = CsrGraph::from_road_network(&network);

        // Build PathfindingData from the same sources
        let node_road_types: Vec<RoadType> = csr
            .nodes
            .iter()
            .map(|n| grid.get(n.0, n.1).road_type)
            .collect();

        let data = PathfindingData {
            nodes: csr.nodes.clone(),
            node_offsets: csr.node_offsets.clone(),
            edges: csr.edges.clone(),
            weights: csr.weights.clone(),
            node_road_types,
            traffic_density: traffic.density.clone(),
            traffic_width: traffic.width,
        };

        // Both should find the same path
        let path_csr =
            csr_find_path_with_traffic(&csr, RoadNode(5, 10), RoadNode(15, 10), &grid, &traffic);
        let path_data = data.find_path_with_traffic(RoadNode(5, 10), RoadNode(15, 10));

        assert!(path_csr.is_some());
        assert!(path_data.is_some());
        assert_eq!(path_csr.unwrap(), path_data.unwrap());
    }

    #[test]
    fn test_pathfinding_data_traffic_avoidance() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();
        let mut traffic = TrafficGrid::default();

        // Build two parallel routes
        for x in 5..=15 {
            network.place_road(&mut grid, x, 10);
            network.place_road(&mut grid, x, 12);
        }
        for y in 10..=12 {
            network.place_road(&mut grid, 5, y);
            network.place_road(&mut grid, 15, y);
        }

        // Congest the direct route
        for x in 6..15 {
            traffic.set(x, 10, 100);
        }

        let csr = CsrGraph::from_road_network(&network);
        let node_road_types: Vec<RoadType> = csr
            .nodes
            .iter()
            .map(|n| grid.get(n.0, n.1).road_type)
            .collect();

        let data = PathfindingData {
            nodes: csr.nodes.clone(),
            node_offsets: csr.node_offsets.clone(),
            edges: csr.edges.clone(),
            weights: csr.weights.clone(),
            node_road_types,
            traffic_density: traffic.density.clone(),
            traffic_width: traffic.width,
        };

        let path = data.find_path_with_traffic(RoadNode(5, 10), RoadNode(15, 10));
        assert!(path.is_some());
        let path = path.unwrap();

        // Should avoid congested y=10 route
        let uses_alternate = path.iter().any(|n| n.1 == 12);
        assert!(
            uses_alternate,
            "PathfindingData should route around congestion via y=12"
        );
    }
}

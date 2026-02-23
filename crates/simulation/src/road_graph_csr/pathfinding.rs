use crate::grid::WorldGrid;
use crate::roads::RoadNode;
use crate::traffic::TrafficGrid;

use super::graph::CsrGraph;

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

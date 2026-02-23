//! TEST-004: Unit tests for CSR A* pathfinding.
//!
//! Covers: straight-line paths, path around obstacles, no-path cases,
//! single-node graph, disconnected components, path cost monotonicity,
//! and traffic-aware routing.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::WorldGrid;
use crate::road_graph_csr::{
    bpr_travel_time, csr_find_path, csr_find_path_with_traffic, CsrGraph, BPR_ALPHA, BPR_BETA,
};
use crate::roads::{RoadNetwork, RoadNode};
use crate::traffic::TrafficGrid;

fn build_csr(setup: impl FnOnce(&mut WorldGrid, &mut RoadNetwork)) -> CsrGraph {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut net = RoadNetwork::default();
    setup(&mut grid, &mut net);
    CsrGraph::from_road_network(&net)
}

fn build_csr_with_grid(
    setup: impl FnOnce(&mut WorldGrid, &mut RoadNetwork),
) -> (CsrGraph, WorldGrid) {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut net = RoadNetwork::default();
    setup(&mut grid, &mut net);
    (CsrGraph::from_road_network(&net), grid)
}

// ── Straight-line path ──────────────────────────────────────────────

#[test]
fn test_straight_horizontal_path_returns_correct_waypoints() {
    let csr = build_csr(|g, n| {
        for x in 10..=20 { n.place_road(g, x, 15); }
    });
    let path = csr_find_path(&csr, RoadNode(10, 15), RoadNode(20, 15)).unwrap();
    assert_eq!(path.len(), 11);
    for (i, node) in path.iter().enumerate() {
        assert_eq!(*node, RoadNode(10 + i, 15));
    }
}

#[test]
fn test_straight_vertical_path_returns_correct_waypoints() {
    let csr = build_csr(|g, n| {
        for y in 30..=40 { n.place_road(g, 20, y); }
    });
    let path = csr_find_path(&csr, RoadNode(20, 30), RoadNode(20, 40)).unwrap();
    assert_eq!(path.len(), 11);
    assert_eq!(*path.first().unwrap(), RoadNode(20, 30));
    assert_eq!(*path.last().unwrap(), RoadNode(20, 40));
    for node in &path { assert_eq!(node.0, 20); }
}

#[test]
fn test_straight_path_works_in_reverse() {
    let csr = build_csr(|g, n| {
        for x in 10..=20 { n.place_road(g, x, 15); }
    });
    let path = csr_find_path(&csr, RoadNode(20, 15), RoadNode(10, 15)).unwrap();
    assert_eq!(path.len(), 11);
    assert_eq!(*path.first().unwrap(), RoadNode(20, 15));
    assert_eq!(*path.last().unwrap(), RoadNode(10, 15));
}

// ── Path around obstacle ────────────────────────────────────────────

#[test]
fn test_path_around_obstacle_l_shape() {
    let csr = build_csr(|g, n| {
        for x in 10..=20 { n.place_road(g, x, 10); }
        for y in 10..=20 { n.place_road(g, 20, y); }
    });
    let path = csr_find_path(&csr, RoadNode(10, 10), RoadNode(20, 20)).unwrap();
    assert!(path.contains(&RoadNode(20, 10)), "Must traverse corner");
    assert_eq!(path.len(), 21);
}

#[test]
fn test_path_around_obstacle_u_shape() {
    let csr = build_csr(|g, n| {
        for y in 10..=20 { n.place_road(g, 10, y); n.place_road(g, 20, y); }
        for x in 10..=20 { n.place_road(g, x, 20); }
    });
    let path = csr_find_path(&csr, RoadNode(10, 10), RoadNode(20, 10)).unwrap();
    assert!(path.iter().any(|n| n.1 == 20), "Must traverse bottom");
    assert_eq!(path.len(), 31);
}

#[test]
fn test_path_chooses_shorter_of_two_routes() {
    let csr = build_csr(|g, n| {
        for x in 10..=20 { n.place_road(g, x, 10); } // direct
        for y in 10..=15 { n.place_road(g, 10, y); n.place_road(g, 20, y); }
        for x in 10..=20 { n.place_road(g, x, 15); } // long detour
    });
    let path = csr_find_path(&csr, RoadNode(10, 10), RoadNode(20, 10)).unwrap();
    assert_eq!(path.len(), 11);
    for node in &path { assert_eq!(node.1, 10); }
}

// ── No-path-exists ──────────────────────────────────────────────────

#[test]
fn test_no_path_disconnected_segments() {
    let csr = build_csr(|g, n| {
        for x in 5..=10 { n.place_road(g, x, 5); }
        for x in 50..=55 { n.place_road(g, x, 50); }
    });
    assert!(csr_find_path(&csr, RoadNode(5, 5), RoadNode(50, 50)).is_none());
    assert!(csr_find_path(&csr, RoadNode(55, 50), RoadNode(10, 5)).is_none());
}

#[test]
fn test_no_path_nonexistent_nodes() {
    let csr = build_csr(|g, n| {
        for x in 5..=10 { n.place_road(g, x, 5); }
    });
    assert!(csr_find_path(&csr, RoadNode(100, 100), RoadNode(5, 5)).is_none());
    assert!(csr_find_path(&csr, RoadNode(5, 5), RoadNode(200, 200)).is_none());
}

#[test]
fn test_no_path_empty_graph() {
    let csr = CsrGraph::from_road_network(&RoadNetwork::default());
    assert!(csr_find_path(&csr, RoadNode(0, 0), RoadNode(1, 1)).is_none());
}

// ── Single-node graph ───────────────────────────────────────────────

#[test]
fn test_single_node_self_path() {
    let csr = build_csr(|g, n| { n.place_road(g, 50, 50); });
    assert_eq!(csr.node_count(), 1);
    assert_eq!(csr.edge_count(), 0);
    let path = csr_find_path(&csr, RoadNode(50, 50), RoadNode(50, 50)).unwrap();
    assert_eq!(path, vec![RoadNode(50, 50)]);
}

#[test]
fn test_single_node_no_path_to_other() {
    let csr = build_csr(|g, n| { n.place_road(g, 50, 50); });
    assert!(csr_find_path(&csr, RoadNode(50, 50), RoadNode(51, 50)).is_none());
}

// ── Disconnected components ─────────────────────────────────────────

#[test]
fn test_disconnected_components_intra_and_inter() {
    let csr = build_csr(|g, n| {
        for x in 10..=20 { n.place_road(g, x, 5); }   // A
        for x in 30..=40 { n.place_road(g, x, 50); }   // B
        n.place_road(g, 100, 100);                       // C (isolated)
    });
    // Intra-component
    assert!(csr_find_path(&csr, RoadNode(10, 5), RoadNode(20, 5)).is_some());
    assert!(csr_find_path(&csr, RoadNode(30, 50), RoadNode(40, 50)).is_some());
    assert!(csr_find_path(&csr, RoadNode(100, 100), RoadNode(100, 100)).is_some());
    // Inter-component
    assert!(csr_find_path(&csr, RoadNode(10, 5), RoadNode(30, 50)).is_none());
    assert!(csr_find_path(&csr, RoadNode(30, 50), RoadNode(100, 100)).is_none());
    assert!(csr_find_path(&csr, RoadNode(10, 5), RoadNode(100, 100)).is_none());
}

#[test]
fn test_connecting_components_enables_pathfinding() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut net = RoadNetwork::default();
    for x in 10..=15 { net.place_road(&mut grid, x, 10); }
    for x in 20..=25 { net.place_road(&mut grid, x, 10); }

    let csr = CsrGraph::from_road_network(&net);
    assert!(csr_find_path(&csr, RoadNode(10, 10), RoadNode(25, 10)).is_none());

    for x in 15..=20 { net.place_road(&mut grid, x, 10); }
    let csr = CsrGraph::from_road_network(&net);
    assert!(csr_find_path(&csr, RoadNode(10, 10), RoadNode(25, 10)).is_some());
}

// ── Path cost monotonicity ──────────────────────────────────────────

#[test]
fn test_cumulative_cost_nondecreasing() {
    let csr = build_csr(|g, n| {
        for x in 10..=30 { n.place_road(g, x, 10); }
        for y in 10..=30 { n.place_road(g, 30, y); }
    });
    let path = csr_find_path(&csr, RoadNode(10, 10), RoadNode(30, 30)).unwrap();

    let mut cumulative = 0u32;
    for i in 1..path.len() {
        let from = csr.find_node_index(&path[i - 1]).unwrap();
        let to = csr.find_node_index(&path[i]).unwrap();
        let w = csr.neighbor_weights(from)
            .find(|(n, _)| *n == to)
            .map(|(_, w)| w)
            .expect("Consecutive path nodes must share an edge");
        let prev = cumulative;
        cumulative += w;
        assert!(cumulative >= prev, "Cost must be non-decreasing at step {i}");
    }
    assert_eq!(cumulative, (path.len() - 1) as u32);
}

#[test]
fn test_cumulative_bpr_cost_nondecreasing() {
    let (csr, grid) = build_csr_with_grid(|g, n| {
        for x in 10..=30 { n.place_road(g, x, 15); }
    });
    let traffic = TrafficGrid::default();
    let path = csr_find_path_with_traffic(
        &csr, RoadNode(10, 15), RoadNode(30, 15), &grid, &traffic,
    ).unwrap();

    let mut cumulative = 0.0f64;
    for i in 1..path.len() {
        let (a, b) = (path[i - 1], path[i]);
        let dx = (b.0 as f64 - a.0 as f64).abs();
        let dy = (b.1 as f64 - a.1 as f64).abs();
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        let rt = grid.get(b.0, b.1).road_type;
        let ff = dist / rt.speed() as f64 * 100.0;
        let cost = bpr_travel_time(ff, 0.0, rt.capacity() as f64, BPR_ALPHA, BPR_BETA);
        let prev = cumulative;
        cumulative += cost;
        assert!(cumulative >= prev, "BPR cumulative cost non-decreasing");
    }
}

// ── Traffic-aware pathfinding ───────────────────────────────────────

fn build_parallel_routes() -> (CsrGraph, WorldGrid) {
    build_csr_with_grid(|g, n| {
        for x in 10..=30 { n.place_road(g, x, 10); n.place_road(g, x, 20); }
        for y in 10..=20 { n.place_road(g, 10, y); n.place_road(g, 30, y); }
    })
}

#[test]
fn test_traffic_aware_prefers_less_congested_route() {
    let (csr, grid) = build_parallel_routes();
    let mut traffic = TrafficGrid::default();
    for x in 11..30 { traffic.set(x, 10, 200); }

    let path = csr_find_path_with_traffic(
        &csr, RoadNode(10, 10), RoadNode(30, 10), &grid, &traffic,
    ).unwrap();
    assert!(
        path.iter().any(|n| n.1 == 20),
        "Should detour through y=20 to avoid congestion on y=10"
    );
}

#[test]
fn test_traffic_aware_takes_direct_route_when_clear() {
    let (csr, grid) = build_parallel_routes();
    let traffic = TrafficGrid::default();

    let path = csr_find_path_with_traffic(
        &csr, RoadNode(10, 10), RoadNode(30, 10), &grid, &traffic,
    ).unwrap();
    assert_eq!(path.len(), 21);
    for node in &path { assert_eq!(node.1, 10); }
}

#[test]
fn test_traffic_aware_no_path_returns_none() {
    let (csr, grid) = build_csr_with_grid(|g, n| {
        for x in 5..=10 { n.place_road(g, x, 5); }
        for x in 50..=55 { n.place_road(g, x, 50); }
    });
    let traffic = TrafficGrid::default();
    assert!(csr_find_path_with_traffic(
        &csr, RoadNode(5, 5), RoadNode(50, 50), &grid, &traffic,
    ).is_none());
}

#[test]
fn test_bpr_cost_strictly_increases_with_volume() {
    let ff = 100.0;
    let cap = 20.0;
    let c0 = bpr_travel_time(ff, 0.0, cap, BPR_ALPHA, BPR_BETA);
    let c1 = bpr_travel_time(ff, 10.0, cap, BPR_ALPHA, BPR_BETA);
    let c2 = bpr_travel_time(ff, 20.0, cap, BPR_ALPHA, BPR_BETA);
    let c3 = bpr_travel_time(ff, 40.0, cap, BPR_ALPHA, BPR_BETA);
    assert!(c0 < c1 && c1 < c2 && c2 < c3);
    assert!((c2 - ff * 1.15).abs() < 1e-9, "At capacity: 1.15x free-flow");
}

#[test]
fn test_path_nodes_are_all_valid_graph_members() {
    let csr = build_csr(|g, n| {
        for x in 5..=25 { n.place_road(g, x, 15); }
        for y in 5..=25 { n.place_road(g, 15, y); }
    });
    let path = csr_find_path(&csr, RoadNode(5, 15), RoadNode(15, 25)).unwrap();
    for node in &path {
        assert!(csr.find_node_index(node).is_some(), "{node:?} in graph");
    }
    for i in 1..path.len() {
        let from = csr.find_node_index(&path[i - 1]).unwrap();
        let to = csr.find_node_index(&path[i]).unwrap();
        assert!(csr.neighbors(from).contains(&to), "Edge must exist");
    }
}

//! TEST-021: Deterministic collections integration tests
//!
//! Verifies that road network iteration, CSR graph building, and pathfinding
//! produce identical results across multiple runs, ensuring simulation
//! determinism through ordered collection usage.

use crate::grid::{RoadType, ZoneType};
use crate::road_graph_csr::CsrGraph;
use crate::roads::{RoadNetwork, RoadNode};
use crate::test_harness::TestCity;

/// Build a road network and verify that neighbors are returned in a
/// deterministic (sorted) order, regardless of insertion order.
#[test]
fn test_road_network_neighbors_are_deterministic() {
    let mut city = TestCity::new();
    let world = city.world_mut();
    let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
    let mut roads = world.resource_mut::<RoadNetwork>();

    // Build a cross intersection at (50, 50)
    roads.place_road(&mut grid, 50, 50);
    roads.place_road(&mut grid, 49, 50);
    roads.place_road(&mut grid, 51, 50);
    roads.place_road(&mut grid, 50, 49);
    roads.place_road(&mut grid, 50, 51);

    let neighbors = roads.neighbors(&RoadNode(50, 50));
    assert_eq!(neighbors.len(), 4);

    // BTreeSet iteration is deterministic: sorted by (x, y) via derived Ord
    assert_eq!(neighbors[0], RoadNode(49, 50));
    assert_eq!(neighbors[1], RoadNode(50, 49));
    assert_eq!(neighbors[2], RoadNode(50, 51));
    assert_eq!(neighbors[3], RoadNode(51, 50));

    // Verify reproducibility: call again and get same result
    let neighbors2 = roads.neighbors(&RoadNode(50, 50));
    assert_eq!(neighbors, neighbors2, "neighbors must be identical across calls");
}

/// Build a CSR graph from the road network and verify that the edge list
/// for each node is sorted by neighbor index, ensuring deterministic
/// pathfinding traversal.
#[test]
fn test_csr_graph_edges_are_sorted() {
    let mut city = TestCity::new();
    let world = city.world_mut();
    let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
    let mut roads = world.resource_mut::<RoadNetwork>();

    // Create an L-shaped road with an intersection at (50, 50)
    for x in 48..=52 {
        roads.place_road(&mut grid, x, 50);
    }
    for y in 48..=52 {
        if y != 50 {
            roads.place_road(&mut grid, 50, y);
        }
    }

    let csr = CsrGraph::from_road_network(&roads);

    // For every node, verify its edge list is sorted
    for node_idx in 0..csr.node_count() as u32 {
        let neighbors = csr.neighbors(node_idx);
        for window in neighbors.windows(2) {
            assert!(
                window[0] <= window[1],
                "CSR edges for node {} are not sorted: {:?}",
                node_idx,
                neighbors
            );
        }
    }
}

/// Build two identical road networks with different insertion orders and
/// verify the resulting CSR graphs are byte-identical.
#[test]
fn test_csr_graph_deterministic_across_insertion_orders() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::WorldGrid;

    // Build network A: roads placed left-to-right, then top-to-bottom
    let mut grid_a = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads_a = RoadNetwork::default();
    for x in 10..=20 {
        roads_a.place_road(&mut grid_a, x, 15);
    }
    for y in 10..=20 {
        if y != 15 {
            roads_a.place_road(&mut grid_a, 15, y);
        }
    }
    let csr_a = CsrGraph::from_road_network(&roads_a);

    // Build network B: same roads but placed in reverse order
    let mut grid_b = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads_b = RoadNetwork::default();
    for y in (10..=20).rev() {
        if y != 15 {
            roads_b.place_road(&mut grid_b, 15, y);
        }
    }
    for x in (10..=20).rev() {
        roads_b.place_road(&mut grid_b, x, 15);
    }
    let csr_b = CsrGraph::from_road_network(&roads_b);

    // Both CSR graphs should have identical structure
    assert_eq!(csr_a.nodes.len(), csr_b.nodes.len(), "node count mismatch");
    assert_eq!(csr_a.edges.len(), csr_b.edges.len(), "edge count mismatch");
    assert_eq!(csr_a.nodes, csr_b.nodes, "nodes differ");
    assert_eq!(csr_a.node_offsets, csr_b.node_offsets, "offsets differ");
    assert_eq!(csr_a.edges, csr_b.edges, "edges differ");
    assert_eq!(csr_a.weights, csr_b.weights, "weights differ");
}

/// Verify that pathfinding produces identical results when the same
/// road network is queried multiple times.
#[test]
fn test_pathfinding_deterministic_on_same_network() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::WorldGrid;
    use crate::pathfinding_sys::find_path;

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Create a grid of roads with multiple possible paths
    for x in 10..=30 {
        roads.place_road(&mut grid, x, 15);
        roads.place_road(&mut grid, x, 20);
        roads.place_road(&mut grid, x, 25);
    }
    for y in 15..=25 {
        roads.place_road(&mut grid, 15, y);
        roads.place_road(&mut grid, 20, y);
        roads.place_road(&mut grid, 25, y);
    }

    // Find paths multiple times and verify they're identical
    let path1 = find_path(&roads, RoadNode(10, 15), RoadNode(30, 25));
    let path2 = find_path(&roads, RoadNode(10, 15), RoadNode(30, 25));
    assert!(path1.is_some(), "path should exist");
    assert_eq!(path1, path2, "paths must be identical across calls");
}

/// Verify that BTreeMap iteration order of RoadNetwork edges is
/// deterministic by checking keys come out sorted.
#[test]
fn test_road_network_edge_keys_are_sorted() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::WorldGrid;

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Add roads in a scattered pattern
    let positions = [
        (100, 50),
        (50, 100),
        (10, 10),
        (200, 200),
        (75, 25),
    ];
    for &(x, y) in &positions {
        roads.place_road(&mut grid, x, y);
    }

    // BTreeMap keys should be in sorted order
    let keys: Vec<RoadNode> = roads.edges.keys().copied().collect();
    for window in keys.windows(2) {
        assert!(
            window[0] <= window[1],
            "Edge keys are not sorted: {:?} > {:?}",
            window[0],
            window[1]
        );
    }
}

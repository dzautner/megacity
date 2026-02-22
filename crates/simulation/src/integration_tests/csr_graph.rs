use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{RoadType, WorldGrid};
use crate::roads::{RoadNetwork, RoadNode};
use crate::test_harness::TestCity;

// ====================================================================
// CSR Graph Construction Tests (TEST-052)
// ====================================================================

#[test]
fn test_csr_empty_network_produces_empty_graph() {
    let network = RoadNetwork::default();
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    assert_eq!(csr.node_count(), 0);
    assert_eq!(csr.edge_count(), 0);
    assert_eq!(csr.node_offsets.len(), 1);
    assert_eq!(csr.node_offsets[0], 0);
}

#[test]
fn test_csr_single_road_cell_no_edges() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    network.place_road(&mut grid, 10, 10);
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    assert_eq!(csr.node_count(), 1);
    assert_eq!(csr.edge_count(), 0);
    assert!(csr.find_node_index(&RoadNode(10, 10)).is_some());
}

#[test]
fn test_csr_node_count_matches_road_network() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 10..=14 {
        network.place_road(&mut grid, x, 10);
    }
    for y in 11..=13 {
        network.place_road(&mut grid, 12, y);
    }
    let unique_nodes = network.edges.len();
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    assert_eq!(csr.node_count(), unique_nodes);
    assert_eq!(csr.node_count(), 8);
}

#[test]
fn test_csr_edge_symmetry_undirected() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 5..=10 {
        network.place_road(&mut grid, x, 5);
    }
    for y in 5..=10 {
        network.place_road(&mut grid, 10, y);
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    for node_idx in 0..csr.node_count() as u32 {
        for &neighbor_idx in csr.neighbors(node_idx) {
            assert!(
                csr.neighbors(neighbor_idx).contains(&node_idx),
                "Symmetry: {} -> {} but not reverse",
                node_idx,
                neighbor_idx
            );
        }
    }
}

#[test]
fn test_csr_edge_count_matches_road_network() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 5..=15 {
        network.place_road(&mut grid, x, 10);
    }
    for y in 5..=15 {
        network.place_road(&mut grid, 10, y);
    }
    let network_edge_count: usize = network.edges.values().map(|n| n.len()).sum();
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    assert_eq!(csr.edge_count(), network_edge_count);
}

#[test]
fn test_csr_adjacency_matches_road_network() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 10..=13 {
        for y in 10..=13 {
            network.place_road(&mut grid, x, y);
        }
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    for (node, neighbors) in &network.edges {
        let node_idx = csr.find_node_index(node).unwrap();
        let csr_neighbors: std::collections::HashSet<u32> =
            csr.neighbors(node_idx).iter().copied().collect();
        for neighbor in neighbors {
            let neighbor_idx = csr.find_node_index(neighbor).unwrap();
            assert!(csr_neighbors.contains(&neighbor_idx));
        }
        assert_eq!(csr_neighbors.len(), neighbors.len());
    }
}

#[test]
fn test_csr_disconnected_components() {
    use crate::road_graph_csr::csr_find_path;
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 5..=10 {
        network.place_road(&mut grid, x, 5);
    }
    for x in 5..=10 {
        network.place_road(&mut grid, x, 20);
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    assert_eq!(csr.node_count(), 12);
    assert!(csr_find_path(&csr, RoadNode(5, 5), RoadNode(10, 5)).is_some());
    assert!(csr_find_path(&csr, RoadNode(5, 20), RoadNode(10, 20)).is_some());
    assert!(csr_find_path(&csr, RoadNode(5, 5), RoadNode(5, 20)).is_none());
}

#[test]
fn test_csr_pathfinding_start_equals_goal() {
    use crate::road_graph_csr::csr_find_path;
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 5..=10 {
        network.place_road(&mut grid, x, 5);
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    let path = csr_find_path(&csr, RoadNode(7, 5), RoadNode(7, 5)).unwrap();
    assert_eq!(path.len(), 1);
    assert_eq!(path[0], RoadNode(7, 5));
}

#[test]
fn test_csr_pathfinding_nonexistent_nodes() {
    use crate::road_graph_csr::csr_find_path;
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 5..=10 {
        network.place_road(&mut grid, x, 5);
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    assert!(csr_find_path(&csr, RoadNode(100, 100), RoadNode(5, 5)).is_none());
    assert!(csr_find_path(&csr, RoadNode(5, 5), RoadNode(100, 100)).is_none());
}

#[test]
fn test_csr_path_correctness_straight_line() {
    use crate::road_graph_csr::csr_find_path;
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 5..=15 {
        network.place_road(&mut grid, x, 10);
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    let path = csr_find_path(&csr, RoadNode(5, 10), RoadNode(15, 10)).unwrap();
    assert_eq!(path.len(), 11);
    assert_eq!(path[0], RoadNode(5, 10));
    assert_eq!(path[10], RoadNode(15, 10));
    for node in &path {
        assert_eq!(node.1, 10);
    }
}

#[test]
fn test_csr_node_offsets_structure() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 5..=10 {
        network.place_road(&mut grid, x, 5);
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    assert_eq!(csr.node_offsets.len(), csr.node_count() + 1);
    for i in 0..csr.node_offsets.len() - 1 {
        assert!(csr.node_offsets[i] <= csr.node_offsets[i + 1]);
    }
    assert_eq!(*csr.node_offsets.last().unwrap() as usize, csr.edge_count());
    assert_eq!(csr.weights.len(), csr.edges.len());
}

#[test]
fn test_csr_weights_are_all_one_by_default() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 5..=10 {
        network.place_road(&mut grid, x, 5);
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    for &w in &csr.weights {
        assert_eq!(w, 1);
    }
}

#[test]
fn test_csr_find_node_index_binary_search() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    network.place_road(&mut grid, 3, 7);
    network.place_road(&mut grid, 50, 100);
    network.place_road(&mut grid, 200, 200);
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    assert!(csr.find_node_index(&RoadNode(3, 7)).is_some());
    assert!(csr.find_node_index(&RoadNode(50, 100)).is_some());
    assert!(csr.find_node_index(&RoadNode(200, 200)).is_some());
    assert!(csr.find_node_index(&RoadNode(0, 0)).is_none());
    assert!(csr.find_node_index(&RoadNode(255, 255)).is_none());
}

#[test]
fn test_csr_large_grid_road_network() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 50..100 {
        for y in 50..100 {
            network.place_road(&mut grid, x, y);
        }
    }
    let unique_nodes = network.edges.len();
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    assert_eq!(csr.node_count(), unique_nodes);
    for node_idx in 0..csr.node_count() as u32 {
        for &neighbor_idx in csr.neighbors(node_idx) {
            assert!(csr.neighbors(neighbor_idx).contains(&node_idx));
        }
    }
}

#[test]
fn test_csr_path_through_intersection() {
    use crate::road_graph_csr::csr_find_path;
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 5..=15 {
        network.place_road(&mut grid, x, 10);
    }
    for y in 5..=15 {
        network.place_road(&mut grid, 10, y);
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    let path = csr_find_path(&csr, RoadNode(5, 10), RoadNode(10, 15)).unwrap();
    assert!(path.contains(&RoadNode(10, 10)));
}

#[test]
fn test_csr_integration_with_test_city() {
    use crate::road_graph_csr::CsrGraph;
    let city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_road(15, 10, 15, 20, RoadType::Local);
    let network = city.road_network();
    let csr = CsrGraph::from_road_network(network);
    assert!(csr.node_count() > 0);
    assert!(csr.edge_count() > 0);
    assert_eq!(csr.node_count(), network.edges.len());
}

#[test]
fn test_csr_neighbor_weights_iterator() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    for x in 5..=7 {
        network.place_road(&mut grid, x, 5);
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    let middle_idx = csr.find_node_index(&RoadNode(6, 5)).unwrap();
    let nw: Vec<(u32, u32)> = csr.neighbor_weights(middle_idx).collect();
    assert_eq!(nw.len(), 2);
    for &(_, w) in &nw {
        assert_eq!(w, 1);
    }
    assert_eq!(
        csr.neighbors(csr.find_node_index(&RoadNode(5, 5)).unwrap())
            .len(),
        1
    );
    assert_eq!(
        csr.neighbors(csr.find_node_index(&RoadNode(7, 5)).unwrap())
            .len(),
        1
    );
}

#[test]
fn test_csr_nodes_sorted_by_y_then_x() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    network.place_road(&mut grid, 20, 5);
    network.place_road(&mut grid, 5, 20);
    network.place_road(&mut grid, 10, 10);
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    for i in 0..csr.nodes.len() - 1 {
        let a = csr.nodes[i];
        let b = csr.nodes[i + 1];
        assert!((a.1, a.0) <= (b.1, b.0));
    }
}

#[test]
fn test_csr_edge_count_straight_road() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    let n = 10;
    for x in 0..n {
        network.place_road(&mut grid, x, 5);
    }
    let csr = crate::road_graph_csr::CsrGraph::from_road_network(&network);
    assert_eq!(csr.node_count(), n);
    assert_eq!(csr.edge_count(), 2 * (n - 1));
}

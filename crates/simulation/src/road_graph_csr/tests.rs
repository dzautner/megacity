#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{RoadType, WorldGrid};
    use crate::road_graph_csr::{
        bpr_travel_time, csr_find_path, csr_find_path_with_traffic, CsrGraph, PathfindingData,
        BPR_ALPHA, BPR_BETA,
    };
    use crate::roads::{RoadNetwork, RoadNode};
    use crate::traffic::TrafficGrid;

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

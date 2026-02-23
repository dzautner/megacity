#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, RoadType, WorldGrid};
    use crate::outside_connections::detection::*;
    use crate::outside_connections::*;

    // =========================================================================
    // Edge detection helpers
    // =========================================================================

    #[test]
    fn test_is_near_edge() {
        // Corners and edges (within EDGE_PROXIMITY=3)
        assert!(is_near_edge(0, 0));
        assert!(is_near_edge(1, 1));
        assert!(is_near_edge(2, 128));
        assert!(is_near_edge(128, 0));
        assert!(is_near_edge(GRID_WIDTH - 1, 128));
        assert!(is_near_edge(128, GRID_HEIGHT - 1));

        // Boundary: exactly at EDGE_PROXIMITY
        assert!(!is_near_edge(EDGE_PROXIMITY, EDGE_PROXIMITY));

        // Interior
        assert!(!is_near_edge(128, 128));
        assert!(!is_near_edge(50, 50));
        assert!(!is_near_edge(GRID_WIDTH / 2, GRID_HEIGHT / 2));
    }

    #[test]
    fn test_is_near_edge_boundary_values() {
        // x=2 is within EDGE_PROXIMITY=3 (range check: !(3..253).contains(&2) => true)
        assert!(is_near_edge(2, 128));
        // x=3 is NOT near edge (range check: !(3..253).contains(&3) => false)
        assert!(!is_near_edge(3, 128));
        // x=GRID_WIDTH-3 = 253 is near edge (range check: !(3..253).contains(&253) => true)
        assert!(is_near_edge(GRID_WIDTH - 3, 128));
        // x=GRID_WIDTH-4 = 252 is NOT near edge
        assert!(!is_near_edge(GRID_WIDTH - 4, 128));
    }

    #[test]
    fn test_is_near_water_edge_no_water() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Edge cell but no water nearby
        assert!(!is_near_water_edge(0, 0, &grid));
        assert!(!is_near_water_edge(128, 0, &grid));
    }

    #[test]
    fn test_is_near_water_edge_with_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place water near the edge
        grid.get_mut(2, 2).cell_type = CellType::Water;
        // Cell at (0,0) is near edge and water is within 5 cells
        assert!(is_near_water_edge(0, 0, &grid));
    }

    #[test]
    fn test_is_near_water_edge_interior_cell_returns_false() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place water at interior
        grid.get_mut(128, 128).cell_type = CellType::Water;
        // Interior cell is not near edge, so returns false even with water
        assert!(!is_near_water_edge(128, 128, &grid));
    }

    // =========================================================================
    // Highway detection
    // =========================================================================

    #[test]
    fn test_highway_detection_at_map_edges() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        // Place a highway road cell at the south edge (y=0)
        {
            let cell = grid.get_mut(185, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }

        // Place a highway road cell at the north edge (y=255)
        {
            let cell = grid.get_mut(185, GRID_HEIGHT - 1);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }

        // Place a highway road cell NOT at the edge (should NOT be detected)
        {
            let cell = grid.get_mut(100, 128);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }

        let connections = detect_highway_connections(&grid);
        assert_eq!(connections.len(), 2);
        assert!(connections
            .iter()
            .all(|c| c.connection_type == ConnectionType::Highway));

        let positions: Vec<(usize, usize)> =
            connections.iter().map(|c| (c.grid_x, c.grid_y)).collect();
        assert!(positions.contains(&(185, 0)));
        assert!(positions.contains(&(185, GRID_HEIGHT - 1)));
    }

    #[test]
    fn test_boulevard_detected_as_highway_connection() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Boulevard;
        }
        let connections = detect_highway_connections(&grid);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].connection_type, ConnectionType::Highway);
        assert_eq!(connections[0].grid_x, 100);
        assert_eq!(connections[0].grid_y, 0);
    }

    #[test]
    fn test_highway_detection_left_edge() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(0, 128);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].grid_x, 0);
        assert_eq!(connections[0].grid_y, 128);
    }

    #[test]
    fn test_highway_detection_right_edge() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(GRID_WIDTH - 1, 128);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].grid_x, GRID_WIDTH - 1);
        assert_eq!(connections[0].grid_y, 128);
    }

    #[test]
    fn test_highway_clustering_avoids_duplicates() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place two highway cells close together at south edge (within 10 Manhattan distance)
        for x in 50..55 {
            let cell = grid.get_mut(x, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        // Should cluster into 1 connection, not 5
        assert_eq!(connections.len(), 1);
    }

    #[test]
    fn test_highway_two_distant_clusters_detected_separately() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Two highway cells far apart on the same edge (>10 apart)
        {
            let cell = grid.get_mut(20, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        assert_eq!(connections.len(), 2);
    }

    #[test]
    fn test_highway_capacity_is_5000() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        assert_eq!(connections[0].capacity, 5000);
    }

    #[test]
    fn test_highway_initial_utilization_is_zero() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        assert!((connections[0].utilization - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_non_highway_road_at_edge_not_detected() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Local road at edge should NOT be detected
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Local;
        }
        let connections = detect_highway_connections(&grid);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_empty_grid_no_highway_connections() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let connections = detect_highway_connections(&grid);
        assert!(connections.is_empty());
    }
}

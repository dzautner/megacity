#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, RoadType, WorldGrid};
    use crate::outside_connections::detection::*;
    use crate::outside_connections::*;
    use crate::services::{ServiceBuilding, ServiceType};

    // =========================================================================
    // 3. Edge detection helpers
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
    // 4. Highway detection
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

    // =========================================================================
    // 5. Railway detection
    // =========================================================================

    #[test]
    fn test_railway_detection_from_train_station_near_edge() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::TrainStation,
            grid_x: 1,
            grid_y: 128,
            radius: 50.0,
        },)];
        let connections = detect_railway_connections(&services);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].connection_type, ConnectionType::Railway);
        assert_eq!(connections[0].capacity, 2000);
        assert_eq!(connections[0].grid_x, 1);
        assert_eq!(connections[0].grid_y, 128);
    }

    #[test]
    fn test_railway_not_detected_for_interior_train_station() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::TrainStation,
            grid_x: 128,
            grid_y: 128,
            radius: 50.0,
        },)];
        let connections = detect_railway_connections(&services);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_railway_not_detected_for_non_train_service_at_edge() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::FireStation,
            grid_x: 0,
            grid_y: 128,
            radius: 50.0,
        },)];
        let connections = detect_railway_connections(&services);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_multiple_railway_connections() {
        let services = vec![
            (&ServiceBuilding {
                service_type: ServiceType::TrainStation,
                grid_x: 0,
                grid_y: 50,
                radius: 50.0,
            },),
            (&ServiceBuilding {
                service_type: ServiceType::TrainStation,
                grid_x: GRID_WIDTH - 1,
                grid_y: 200,
                radius: 50.0,
            },),
        ];
        let connections = detect_railway_connections(&services);
        assert_eq!(connections.len(), 2);
    }

    // =========================================================================
    // 6. SeaPort detection
    // =========================================================================

    #[test]
    fn test_seaport_detection_from_ferry_pier_near_water_edge() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place water near the edge
        grid.get_mut(1, 1).cell_type = CellType::Water;

        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::FerryPier,
            grid_x: 0,
            grid_y: 0,
            radius: 30.0,
        },)];
        let connections = detect_seaport_connections(&services, &grid);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].connection_type, ConnectionType::SeaPort);
        assert_eq!(connections[0].capacity, 3000);
    }

    #[test]
    fn test_seaport_not_detected_without_water() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // FerryPier at edge but no water
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::FerryPier,
            grid_x: 0,
            grid_y: 128,
            radius: 30.0,
        },)];
        let connections = detect_seaport_connections(&services, &grid);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_seaport_not_detected_for_interior_ferry_pier() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Water in interior
        grid.get_mut(128, 128).cell_type = CellType::Water;
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::FerryPier,
            grid_x: 128,
            grid_y: 128,
            radius: 30.0,
        },)];
        let connections = detect_seaport_connections(&services, &grid);
        assert!(connections.is_empty());
    }

    // =========================================================================
    // 7. Airport detection
    // =========================================================================

    #[test]
    fn test_airport_detection_small_airstrip() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::SmallAirstrip,
            grid_x: 100,
            grid_y: 100,
            radius: 50.0,
        },)];
        let connections = detect_airport_connections(&services);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].connection_type, ConnectionType::Airport);
        assert_eq!(connections[0].capacity, 1000);
    }

    #[test]
    fn test_airport_detection_regional_airport() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::RegionalAirport,
            grid_x: 80,
            grid_y: 80,
            radius: 80.0,
        },)];
        let connections = detect_airport_connections(&services);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].capacity, 3000);
    }

    #[test]
    fn test_airport_detection_international_airport() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::InternationalAirport,
            grid_x: 60,
            grid_y: 60,
            radius: 120.0,
        },)];
        let connections = detect_airport_connections(&services);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].capacity, 5000);
    }

    #[test]
    fn test_airport_not_detected_for_non_airport_service() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: 100,
            grid_y: 100,
            radius: 50.0,
        },)];
        let connections = detect_airport_connections(&services);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_multiple_airport_types_detected() {
        let services = vec![
            (&ServiceBuilding {
                service_type: ServiceType::SmallAirstrip,
                grid_x: 30,
                grid_y: 30,
                radius: 50.0,
            },),
            (&ServiceBuilding {
                service_type: ServiceType::InternationalAirport,
                grid_x: 200,
                grid_y: 200,
                radius: 120.0,
            },),
        ];
        let connections = detect_airport_connections(&services);
        assert_eq!(connections.len(), 2);
        // Total capacity: 1000 + 5000 = 6000
        let total_capacity: u32 = connections.iter().map(|c| c.capacity).sum();
        assert_eq!(total_capacity, 6000);
    }

    #[test]
    fn test_airport_detection_does_not_require_edge() {
        // Unlike railway, airports don't need to be near the edge
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::InternationalAirport,
            grid_x: 128,
            grid_y: 128,
            radius: 120.0,
        },)];
        let connections = detect_airport_connections(&services);
        assert_eq!(connections.len(), 1);
    }

    // =========================================================================
    // 8. Connection capacity limits
    // =========================================================================

    #[test]
    fn test_connection_capacity_values_by_type() {
        // Verify each detection function assigns the correct capacity
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let highway_conns = detect_highway_connections(&grid);
        assert_eq!(
            highway_conns[0].capacity, 5000,
            "Highway capacity should be 5000"
        );

        let rail_services = vec![(&ServiceBuilding {
            service_type: ServiceType::TrainStation,
            grid_x: 0,
            grid_y: 128,
            radius: 50.0,
        },)];
        let rail_conns = detect_railway_connections(&rail_services);
        assert_eq!(
            rail_conns[0].capacity, 2000,
            "Railway capacity should be 2000"
        );

        let mut water_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        water_grid.get_mut(1, 1).cell_type = CellType::Water;
        let port_services = vec![(&ServiceBuilding {
            service_type: ServiceType::FerryPier,
            grid_x: 0,
            grid_y: 0,
            radius: 30.0,
        },)];
        let port_conns = detect_seaport_connections(&port_services, &water_grid);
        assert_eq!(
            port_conns[0].capacity, 3000,
            "SeaPort capacity should be 3000"
        );

        let air_services_small = vec![(&ServiceBuilding {
            service_type: ServiceType::SmallAirstrip,
            grid_x: 100,
            grid_y: 100,
            radius: 50.0,
        },)];
        let air_conns = detect_airport_connections(&air_services_small);
        assert_eq!(
            air_conns[0].capacity, 1000,
            "SmallAirstrip capacity should be 1000"
        );

        let air_services_regional = vec![(&ServiceBuilding {
            service_type: ServiceType::RegionalAirport,
            grid_x: 100,
            grid_y: 100,
            radius: 80.0,
        },)];
        let air_conns = detect_airport_connections(&air_services_regional);
        assert_eq!(
            air_conns[0].capacity, 3000,
            "RegionalAirport capacity should be 3000"
        );

        let air_services_intl = vec![(&ServiceBuilding {
            service_type: ServiceType::InternationalAirport,
            grid_x: 100,
            grid_y: 100,
            radius: 120.0,
        },)];
        let air_conns = detect_airport_connections(&air_services_intl);
        assert_eq!(
            air_conns[0].capacity, 5000,
            "InternationalAirport capacity should be 5000"
        );
    }
}

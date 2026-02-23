#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, RoadType, WorldGrid};
    use crate::outside_connections::detection::*;
    use crate::outside_connections::*;
    use crate::services::{ServiceBuilding, ServiceType};

    // =========================================================================
    // Railway detection
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
    // SeaPort detection
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
    // Airport detection
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
    // Connection capacity limits
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

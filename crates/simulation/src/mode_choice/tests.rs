//! Unit tests for the mode choice system.

#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, RoadType, WorldGrid};
    use crate::mode_choice::constants::*;
    use crate::mode_choice::evaluation::*;
    use crate::mode_choice::types::*;
    use crate::services::ServiceType;

    // -------------------------------------------------------------------------
    // TransportMode tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_transport_mode_speed_multipliers() {
        assert!(
            (TransportMode::Walk.speed_multiplier() - WALK_SPEED_MULTIPLIER).abs() < f32::EPSILON
        );
        assert!(
            (TransportMode::Bike.speed_multiplier() - BIKE_SPEED_MULTIPLIER).abs() < f32::EPSILON
        );
        assert!(
            (TransportMode::Drive.speed_multiplier() - DRIVE_SPEED_MULTIPLIER).abs() < f32::EPSILON
        );
        assert!(
            (TransportMode::Transit.speed_multiplier() - TRANSIT_SPEED_MULTIPLIER).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_transport_mode_comfort_factors() {
        assert!((TransportMode::Walk.comfort_factor() - WALK_COMFORT).abs() < f32::EPSILON);
        assert!((TransportMode::Bike.comfort_factor() - BIKE_COMFORT).abs() < f32::EPSILON);
        assert!((TransportMode::Drive.comfort_factor() - DRIVE_COMFORT).abs() < f32::EPSILON);
        assert!((TransportMode::Transit.comfort_factor() - TRANSIT_COMFORT).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transport_mode_labels() {
        assert_eq!(TransportMode::Walk.label(), "Walking");
        assert_eq!(TransportMode::Bike.label(), "Bicycle");
        assert_eq!(TransportMode::Drive.label(), "Car");
        assert_eq!(TransportMode::Transit.label(), "Transit");
    }

    #[test]
    fn test_default_mode_is_drive() {
        assert_eq!(TransportMode::default(), TransportMode::Drive);
    }

    // -------------------------------------------------------------------------
    // Distance helpers
    // -------------------------------------------------------------------------

    #[test]
    fn test_manhattan_distance() {
        assert!((manhattan_distance((0, 0), (10, 10)) - 20.0).abs() < f32::EPSILON);
        assert!((manhattan_distance((5, 5), (5, 5)) - 0.0).abs() < f32::EPSILON);
        assert!((manhattan_distance((0, 0), (3, 4)) - 7.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Mode evaluation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_walk_always_available() {
        let time = evaluate_walk(10.0);
        assert!(time > 0.0);
    }

    #[test]
    fn test_walk_perceived_time_calculation() {
        // walk time = distance / walk_speed / comfort
        let distance = 10.0;
        let expected = (distance / WALK_SPEED_MULTIPLIER) / WALK_COMFORT;
        let actual = evaluate_walk(distance);
        assert!((actual - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bike_unavailable_without_infrastructure() {
        let infra = ModeInfrastructureCache::default();
        assert!(evaluate_bike(10.0, (128, 128), &infra).is_none());
    }

    #[test]
    fn test_bike_available_with_nearby_path() {
        let infra = ModeInfrastructureCache {
            bike_paths: vec![(128, 128)],
            ..Default::default()
        };
        assert!(evaluate_bike(10.0, (128, 128), &infra).is_some());
    }

    #[test]
    fn test_bike_unavailable_for_long_distance() {
        let infra = ModeInfrastructureCache {
            bike_paths: vec![(128, 128)],
            ..Default::default()
        };
        assert!(evaluate_bike(MAX_PRACTICAL_BIKE_DISTANCE + 1.0, (128, 128), &infra).is_none());
    }

    #[test]
    fn test_drive_unavailable_without_roads() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert!(evaluate_drive(10.0, (128, 128), &grid).is_none());
    }

    #[test]
    fn test_drive_available_with_road() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(128, 128).cell_type = CellType::Road;
        grid.get_mut(128, 128).road_type = RoadType::Local;
        assert!(evaluate_drive(10.0, (128, 128), &grid).is_some());
    }

    #[test]
    fn test_transit_unavailable_without_stops() {
        let infra = ModeInfrastructureCache::default();
        assert!(evaluate_transit(10.0, (128, 128), (140, 140), &infra).is_none());
    }

    #[test]
    fn test_transit_available_with_stops_at_both_ends() {
        let infra = ModeInfrastructureCache {
            transit_stops: vec![(128, 128), (140, 140)],
            ..Default::default()
        };
        assert!(evaluate_transit(10.0, (128, 128), (140, 140), &infra).is_some());
    }

    #[test]
    fn test_transit_unavailable_with_stop_at_origin_only() {
        let infra = ModeInfrastructureCache {
            transit_stops: vec![(128, 128)],
            ..Default::default()
        };
        // Destination (200, 200) is far from any transit stop
        assert!(evaluate_transit(10.0, (128, 128), (200, 200), &infra).is_none());
    }

    // -------------------------------------------------------------------------
    // Mode choice preference tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_walking_preferred_for_short_distance() {
        // For a very short trip (2 cells), walking should be preferred over driving
        // because driving has a parking overhead of 5 cells.
        // Walk: 2 / 0.3 / 1.0 = 6.67
        // Drive: (2 + 5) / 1.0 / 0.9 = 7.78
        let distance = 2.0;
        let walk_time = evaluate_walk(distance);

        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(128, 128).cell_type = CellType::Road;
        grid.get_mut(128, 128).road_type = RoadType::Local;
        let drive_time = evaluate_drive(distance, (128, 128), &grid).unwrap();

        assert!(
            walk_time < drive_time,
            "For 2-cell trip, walking ({walk_time}) should be faster than driving ({drive_time})"
        );
    }

    #[test]
    fn test_driving_preferred_for_long_distance() {
        // For a long trip (100 cells), driving should be preferred over walking.
        let distance = 100.0;
        let walk_time = evaluate_walk(distance);

        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(50, 50).cell_type = CellType::Road;
        grid.get_mut(50, 50).road_type = RoadType::Local;
        let drive_time = evaluate_drive(distance, (50, 50), &grid).unwrap();

        assert!(
            drive_time < walk_time,
            "For 100-cell trip, driving ({drive_time}) should be faster than walking ({walk_time})"
        );
    }

    #[test]
    fn test_bike_faster_than_walk_for_medium_distance() {
        let distance = 30.0;
        let walk_time = evaluate_walk(distance);

        let infra = ModeInfrastructureCache {
            bike_paths: vec![(128, 128)],
            ..Default::default()
        };
        let bike_time = evaluate_bike(distance, (128, 128), &infra).unwrap();

        assert!(
            bike_time < walk_time,
            "For 30-cell trip, biking ({bike_time}) should be faster than walking ({walk_time})"
        );
    }

    // -------------------------------------------------------------------------
    // Mode share stats tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mode_share_default() {
        let stats = ModeShareStats::default();
        assert_eq!(stats.total(), 0);
        assert!((stats.drive_pct - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mode_share_total() {
        let stats = ModeShareStats {
            walk_count: 10,
            bike_count: 20,
            drive_count: 50,
            transit_count: 20,
            ..Default::default()
        };
        assert_eq!(stats.total(), 100);
    }

    // -------------------------------------------------------------------------
    // Transit stop classification
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_transit_stop() {
        assert!(is_transit_stop(ServiceType::BusDepot));
        assert!(is_transit_stop(ServiceType::TrainStation));
        assert!(is_transit_stop(ServiceType::SubwayStation));
        assert!(is_transit_stop(ServiceType::TramDepot));
        assert!(is_transit_stop(ServiceType::FerryPier));
        assert!(!is_transit_stop(ServiceType::FireStation));
        assert!(!is_transit_stop(ServiceType::Hospital));
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let stats = ModeShareStats::default();
        assert!(stats.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_non_zero() {
        use crate::Saveable;
        let stats = ModeShareStats {
            walk_count: 5,
            bike_count: 10,
            drive_count: 80,
            transit_count: 5,
            walk_pct: 5.0,
            bike_pct: 10.0,
            drive_pct: 80.0,
            transit_pct: 5.0,
        };
        assert!(stats.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let stats = ModeShareStats {
            walk_count: 10,
            bike_count: 20,
            drive_count: 50,
            transit_count: 20,
            walk_pct: 10.0,
            bike_pct: 20.0,
            drive_pct: 50.0,
            transit_pct: 20.0,
        };
        let bytes = stats.save_to_bytes().expect("should serialize");
        let restored = ModeShareStats::load_from_bytes(&bytes);
        assert_eq!(restored.walk_count, 10);
        assert_eq!(restored.bike_count, 20);
        assert_eq!(restored.drive_count, 50);
        assert_eq!(restored.transit_count, 20);
        assert!((restored.walk_pct - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(ModeShareStats::SAVE_KEY, "mode_share_stats");
    }

    // -------------------------------------------------------------------------
    // Nearby vehicle road check
    // -------------------------------------------------------------------------

    #[test]
    fn test_has_nearby_vehicle_road_none() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert!(!has_nearby_vehicle_road(&grid, 128, 128, 3));
    }

    #[test]
    fn test_has_nearby_vehicle_road_local() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(129, 128).cell_type = CellType::Road;
        grid.get_mut(129, 128).road_type = RoadType::Local;
        assert!(has_nearby_vehicle_road(&grid, 128, 128, 3));
    }

    #[test]
    fn test_has_nearby_vehicle_road_path_not_vehicle() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(129, 128).cell_type = CellType::Road;
        grid.get_mut(129, 128).road_type = RoadType::Path;
        // Path roads don't allow vehicles
        assert!(!has_nearby_vehicle_road(&grid, 128, 128, 3));
    }

    #[test]
    fn test_has_nearby_vehicle_road_out_of_range() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(135, 128).cell_type = CellType::Road;
        grid.get_mut(135, 128).road_type = RoadType::Local;
        // Road is 7 cells away, radius is 3
        assert!(!has_nearby_vehicle_road(&grid, 128, 128, 3));
    }
}

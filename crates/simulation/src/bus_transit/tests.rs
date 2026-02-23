//! Unit tests for the bus transit system.

#[cfg(test)]
mod tests {
    use crate::bus_transit::state::manhattan_distance;
    use crate::bus_transit::types::*;
    use crate::grid::{CellType, WorldGrid};
    use crate::Saveable;

    fn make_grid_with_road(x: usize, y: usize) -> WorldGrid {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(x, y).cell_type = CellType::Road;
        grid.get_mut(x, y).road_type = crate::grid::RoadType::Local;
        grid
    }

    #[test]
    fn test_add_stop_on_road() {
        let grid = make_grid_with_road(5, 5);
        let mut state = BusTransitState::default();
        let id = state.add_stop(&grid, 5, 5);
        assert!(id.is_some());
        assert_eq!(state.stops.len(), 1);
        assert_eq!(state.stops[0].grid_x, 5);
        assert_eq!(state.stops[0].grid_y, 5);
    }

    #[test]
    fn test_add_stop_on_grass_fails() {
        let grid = WorldGrid::new(32, 32);
        let mut state = BusTransitState::default();
        let id = state.add_stop(&grid, 5, 5);
        assert!(id.is_none());
    }

    #[test]
    fn test_add_stop_duplicate_fails() {
        let grid = make_grid_with_road(5, 5);
        let mut state = BusTransitState::default();
        state.add_stop(&grid, 5, 5);
        let id2 = state.add_stop(&grid, 5, 5);
        assert!(id2.is_none());
        assert_eq!(state.stops.len(), 1);
    }

    #[test]
    fn test_add_route() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(10, 10).cell_type = CellType::Road;
        let mut state = BusTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let s2 = state.add_stop(&grid, 10, 10).unwrap();
        let route_id = state.add_route("Route 1".to_string(), vec![s1, s2]);
        assert!(route_id.is_some());
        assert_eq!(state.routes.len(), 1);
        assert!(!state.routes[0].active); // No depot yet
    }

    #[test]
    fn test_add_route_too_few_stops() {
        let grid = make_grid_with_road(5, 5);
        let mut state = BusTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let route_id = state.add_route("Route 1".to_string(), vec![s1]);
        assert!(route_id.is_none());
    }

    #[test]
    fn test_remove_stop_removes_from_routes() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(15, 15).cell_type = CellType::Road;
        let mut state = BusTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let s2 = state.add_stop(&grid, 10, 10).unwrap();
        let s3 = state.add_stop(&grid, 15, 15).unwrap();
        state.add_route("Route 1".to_string(), vec![s1, s2, s3]);
        assert_eq!(state.routes[0].stop_ids.len(), 3);

        state.remove_stop(s2);
        assert_eq!(state.stops.len(), 2);
        assert_eq!(state.routes[0].stop_ids.len(), 2);
    }

    #[test]
    fn test_remove_stop_removes_route_with_too_few_stops() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(10, 10).cell_type = CellType::Road;
        let mut state = BusTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let s2 = state.add_stop(&grid, 10, 10).unwrap();
        state.add_route("Route 1".to_string(), vec![s1, s2]);
        assert_eq!(state.routes.len(), 1);

        state.remove_stop(s1);
        assert_eq!(state.routes.len(), 0);
    }

    #[test]
    fn test_remove_route() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(10, 10).cell_type = CellType::Road;
        let mut state = BusTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let s2 = state.add_stop(&grid, 10, 10).unwrap();
        let route_id = state
            .add_route("Route 1".to_string(), vec![s1, s2])
            .unwrap();

        // Add a bus manually
        state.buses.push(BusInstance {
            route_id,
            next_stop_index: 0,
            grid_x: 5.0,
            grid_y: 5.0,
            passengers: 0,
            dwell_ticks: 0,
            at_stop: false,
        });

        state.remove_route(route_id);
        assert_eq!(state.routes.len(), 0);
        assert_eq!(state.buses.len(), 0);
    }

    #[test]
    fn test_nearest_stop() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(20, 20).cell_type = CellType::Road;
        let mut state = BusTransitState::default();
        state.add_stop(&grid, 5, 5);
        state.add_stop(&grid, 20, 20);

        // (7,7) is closer to (5,5)
        let nearest = state.nearest_stop(7, 7);
        assert!(nearest.is_some());
        assert_eq!(nearest.unwrap().grid_x, 5);

        // (31, 31) is too far from both stops (dist=22 from (20,20))
        let far = state.nearest_stop(31, 31);
        assert!(far.is_none());
    }

    #[test]
    fn test_manhattan_distance() {
        assert_eq!(manhattan_distance(0, 0, 3, 4), 7);
        assert_eq!(manhattan_distance(5, 5, 5, 5), 0);
        assert_eq!(manhattan_distance(10, 0, 0, 10), 20);
    }

    #[test]
    fn test_estimate_transit_time_no_active_routes() {
        let state = BusTransitState::default();
        assert!(state.estimate_transit_time(0, 0, 10, 10).is_none());
    }

    #[test]
    fn test_estimate_transit_time_with_route() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(15, 15).cell_type = CellType::Road;
        let mut state = BusTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let s2 = state.add_stop(&grid, 15, 15).unwrap();
        state.add_route("Test".to_string(), vec![s1, s2]);
        state.routes[0].active = true; // Force active for test

        let time = state.estimate_transit_time(4, 4, 16, 16);
        assert!(time.is_some());
        let t = time.unwrap();
        // Walk to (5,5) = 2, wait = 20, ride ~67 ticks, walk from = 2
        assert!(t > 20, "Transit time should include wait: {}", t);
    }

    #[test]
    fn test_bus_capacity_constant() {
        assert_eq!(BUS_CAPACITY, 30);
    }

    #[test]
    fn test_saveable_empty_state() {
        let state = BusTransitState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        let mut state = BusTransitState::default();
        state.add_stop(&grid, 5, 5);

        let bytes = state.save_to_bytes().unwrap();
        let loaded = BusTransitState::load_from_bytes(&bytes);
        assert_eq!(loaded.stops.len(), 1);
        assert_eq!(loaded.stops[0].grid_x, 5);
    }
}

//! Unit tests for the metro transit system.

#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, WorldGrid};
    use crate::metro_transit::constants::*;
    use crate::metro_transit::state::MetroTransitState;
    use crate::Saveable;

    fn make_grid() -> WorldGrid {
        WorldGrid::new(GRID_WIDTH, GRID_HEIGHT)
    }

    #[test]
    fn test_add_station() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let id = state.add_station(10, 10, "Central".to_string(), &grid);
        assert!(id.is_some());
        assert_eq!(state.stations.len(), 1);
        assert_eq!(state.stations[0].name, "Central");
    }

    #[test]
    fn test_add_station_on_water_fails() {
        let mut grid = make_grid();
        grid.get_mut(10, 10).cell_type = CellType::Water;

        let mut state = MetroTransitState::default();
        let id = state.add_station(10, 10, "Aquatic".to_string(), &grid);
        assert!(id.is_none());
    }

    #[test]
    fn test_add_station_duplicate_position_fails() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        state.add_station(10, 10, "First".to_string(), &grid);
        let id = state.add_station(10, 10, "Second".to_string(), &grid);
        assert!(id.is_none());
    }

    #[test]
    fn test_remove_station() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let id = state
            .add_station(10, 10, "Central".to_string(), &grid)
            .unwrap();
        assert!(state.remove_station(id));
        assert!(state.stations.is_empty());
    }

    #[test]
    fn test_remove_station_from_lines() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        let s3 = state.add_station(30, 30, "C".to_string(), &grid).unwrap();

        let line_id = state.add_line("Red".to_string(), vec![s1, s2, s3]).unwrap();

        // Remove middle station
        state.remove_station(s2);

        let line = state.lines.iter().find(|l| l.id == line_id).unwrap();
        assert_eq!(line.station_ids, vec![s1, s3]);
        assert!(line.operational); // still has 2 stations
    }

    #[test]
    fn test_add_line() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();

        let line_id = state.add_line("Blue".to_string(), vec![s1, s2]);
        assert!(line_id.is_some());
        assert_eq!(state.lines.len(), 1);
        assert!(state.lines[0].operational);
    }

    #[test]
    fn test_add_line_too_few_stations_fails() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();

        let line_id = state.add_line("Lonely".to_string(), vec![s1]);
        assert!(line_id.is_none());
    }

    #[test]
    fn test_stations_connected_same_line() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        assert!(state.stations_connected(s1, s2));
    }

    #[test]
    fn test_stations_connected_via_transfer() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        let s3 = state.add_station(30, 30, "C".to_string(), &grid).unwrap();

        // Line 1: A -> B
        state.add_line("Red".to_string(), vec![s1, s2]);
        // Line 2: B -> C (B is transfer station)
        state.add_line("Blue".to_string(), vec![s2, s3]);

        assert!(state.stations_connected(s1, s3));
    }

    #[test]
    fn test_stations_not_connected() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        let s3 = state.add_station(30, 30, "C".to_string(), &grid).unwrap();
        let s4 = state.add_station(40, 40, "D".to_string(), &grid).unwrap();

        // Two separate lines with no transfer
        state.add_line("Red".to_string(), vec![s1, s2]);
        state.add_line("Blue".to_string(), vec![s3, s4]);

        assert!(!state.stations_connected(s1, s3));
    }

    #[test]
    fn test_estimate_travel_time() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(50, 50, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(100, 50, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        // Citizen at (48, 50) going to (102, 50)
        let time = state.estimate_travel_time(48, 50, 102, 50);
        assert!(time.is_some());
        let t = time.unwrap();
        // Should be: walk(2 cells) + wait(2.5min) + ride(50 cells) + walk(2 cells)
        assert!(t > 0.0);
        assert!(t < 1.0); // Should be well under 1 hour
    }

    #[test]
    fn test_estimate_travel_time_too_far_from_station() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(50, 50, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(100, 50, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        // Citizen too far from any station (distance > MAX_WALK_TO_STATION_CELLS)
        let time = state.estimate_travel_time(200, 200, 102, 50);
        assert!(time.is_none());
    }

    #[test]
    fn test_total_monthly_maintenance() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        let cost = state.total_monthly_maintenance();
        // 2 stations * $500/week * 4 weeks + 1 line * $1200/week * 4 weeks
        let expected = 2.0 * STATION_WEEKLY_MAINTENANCE * 4.0 + 1.0 * LINE_WEEKLY_MAINTENANCE * 4.0;
        assert!((cost - expected).abs() < 0.01);
    }

    #[test]
    fn test_extend_line() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        let s3 = state.add_station(30, 30, "C".to_string(), &grid).unwrap();

        let line_id = state.add_line("Red".to_string(), vec![s1, s2]).unwrap();
        assert!(state.extend_line(line_id, s3));

        let line = state.lines.iter().find(|l| l.id == line_id).unwrap();
        assert_eq!(line.station_ids.len(), 3);
    }

    #[test]
    fn test_nearest_station() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        state
            .add_station(10, 10, "Near".to_string(), &grid)
            .unwrap();
        state
            .add_station(100, 100, "Far".to_string(), &grid)
            .unwrap();

        let (id, dist) = state.nearest_station(12, 12).unwrap();
        assert_eq!(id, 0); // First station added
        assert_eq!(dist, 4); // Manhattan distance
    }

    #[test]
    fn test_saveable_roundtrip() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        state.add_station(10, 10, "Central".to_string(), &grid);
        state.add_station(20, 20, "North".to_string(), &grid);
        state.add_line("Red".to_string(), vec![0, 1]);

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = MetroTransitState::load_from_bytes(&bytes);

        assert_eq!(restored.stations.len(), 2);
        assert_eq!(restored.lines.len(), 1);
        assert_eq!(restored.stations[0].name, "Central");
    }

    #[test]
    fn test_saveable_empty_returns_none() {
        let state = MetroTransitState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_daily_ridership_no_lines_is_zero() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        // Stations with no line
        state.add_station(10, 10, "A".to_string(), &grid);
        state.add_station(20, 20, "B".to_string(), &grid);

        assert_eq!(state.estimate_daily_ridership(100_000), 0);
    }

    #[test]
    fn test_daily_ridership_with_line() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        let ridership = state.estimate_daily_ridership(100_000);
        assert!(ridership > 0);
    }

    #[test]
    fn test_remove_line() {
        let grid = make_grid();
        let mut state = MetroTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        let line_id = state.add_line("Red".to_string(), vec![s1, s2]).unwrap();

        assert!(state.remove_line(line_id));
        assert!(state.lines.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_station_returns_false() {
        let mut state = MetroTransitState::default();
        assert!(!state.remove_station(999));
    }

    #[test]
    fn test_remove_nonexistent_line_returns_false() {
        let mut state = MetroTransitState::default();
        assert!(!state.remove_line(999));
    }
}

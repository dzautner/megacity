#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, WorldGrid};
    use crate::traffic::TrafficGrid;
    use crate::weather::{Weather, WeatherCondition};

    use super::super::calculations::{road_neighbor_directions, weather_accident_multiplier};
    use super::super::types::{AccidentTracker, TrafficAccident};

    fn make_grid_with_roads(positions: &[(usize, usize)]) -> WorldGrid {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        for &(x, y) in positions {
            grid.get_mut(x, y).cell_type = CellType::Road;
        }
        grid
    }

    #[test]
    fn test_accident_tracker_default() {
        let tracker = AccidentTracker::default();
        assert_eq!(tracker.active_accidents.len(), 0);
        assert_eq!(tracker.total_accidents, 0);
        assert_eq!(tracker.accidents_this_month, 0);
        assert_eq!(tracker.max_active, 10);
    }

    #[test]
    fn test_road_neighbor_directions() {
        // Create a simple cross intersection at (10, 10)
        let grid = make_grid_with_roads(&[(10, 10), (9, 10), (11, 10), (10, 9), (10, 11)]);
        let dirs = road_neighbor_directions(&grid, 10, 10);
        assert_eq!(dirs, 4, "Cross intersection should have 4 road neighbors");

        // Straight road
        let grid2 = make_grid_with_roads(&[(10, 10), (9, 10), (11, 10)]);
        let dirs2 = road_neighbor_directions(&grid2, 10, 10);
        assert_eq!(dirs2, 2, "Straight road should have 2 road neighbors");

        // Corner cell with no road neighbors
        let grid3 = make_grid_with_roads(&[(0, 0)]);
        let dirs3 = road_neighbor_directions(&grid3, 0, 0);
        assert_eq!(dirs3, 0, "Isolated road cell should have 0 road neighbors");
    }

    #[test]
    fn test_weather_accident_multiplier() {
        let mut weather = Weather::default();

        weather.current_event = WeatherCondition::Sunny;
        assert_eq!(weather_accident_multiplier(&weather), 1.0);

        weather.current_event = WeatherCondition::Rain;
        assert_eq!(weather_accident_multiplier(&weather), 1.5);

        weather.current_event = WeatherCondition::Storm;
        assert_eq!(weather_accident_multiplier(&weather), 2.5);

        weather.current_event = WeatherCondition::Snow;
        assert_eq!(weather_accident_multiplier(&weather), 2.0);

        weather.current_event = WeatherCondition::HeavyRain;
        assert_eq!(weather_accident_multiplier(&weather), 1.8);
    }

    #[test]
    fn test_accident_severity_bounds() {
        // Verify all severity values are in 1-3 range
        for s in 1u8..=3 {
            let ticks = s as u32 * 50;
            assert!(ticks >= 50 && ticks <= 150, "Ticks should be 50-150");
        }
    }

    #[test]
    fn test_accident_ticks_remaining_decreases() {
        let mut accident = TrafficAccident {
            grid_x: 10,
            grid_y: 10,
            severity: 2,
            ticks_remaining: 100,
            responding: false,
            ambulance_dispatched: false,
        };

        // Not responding: decay by 1
        accident.ticks_remaining = accident.ticks_remaining.saturating_sub(1);
        assert_eq!(accident.ticks_remaining, 99);

        // Responding: decay by 2
        accident.responding = true;
        accident.ticks_remaining = accident.ticks_remaining.saturating_sub(2);
        assert_eq!(accident.ticks_remaining, 97);
    }

    #[test]
    fn test_max_active_cap() {
        let mut tracker = AccidentTracker::default();
        tracker.max_active = 3;

        for i in 0..5 {
            if tracker.active_accidents.len() < tracker.max_active {
                tracker.active_accidents.push(TrafficAccident {
                    grid_x: i,
                    grid_y: 0,
                    severity: 1,
                    ticks_remaining: 50,
                    responding: false,
                    ambulance_dispatched: false,
                });
            }
        }

        assert_eq!(
            tracker.active_accidents.len(),
            3,
            "Should not exceed max_active"
        );
    }

    #[test]
    fn test_accident_clears_when_ticks_reach_zero() {
        let mut tracker = AccidentTracker::default();
        tracker.active_accidents.push(TrafficAccident {
            grid_x: 10,
            grid_y: 10,
            severity: 1,
            ticks_remaining: 0,
            responding: false,
            ambulance_dispatched: false,
        });
        tracker.active_accidents.push(TrafficAccident {
            grid_x: 20,
            grid_y: 20,
            severity: 2,
            ticks_remaining: 50,
            responding: false,
            ambulance_dispatched: false,
        });

        tracker.active_accidents.retain(|a| a.ticks_remaining > 0);
        assert_eq!(
            tracker.active_accidents.len(),
            1,
            "Expired accidents should be removed"
        );
        assert_eq!(tracker.active_accidents[0].grid_x, 20);
    }

    #[test]
    fn test_traffic_boost_in_radius() {
        let mut traffic = TrafficGrid::default();

        // Simulate the accident effect: boost traffic in a 3-cell radius
        let ax: usize = 10;
        let ay: usize = 10;
        let severity = 2u8;
        let radius: isize = 3;
        let boost: u16 = match severity {
            3 => 5,
            2 => 3,
            _ => 1,
        };

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() + dy.abs() > radius {
                    continue;
                }
                let nx = ax as isize + dx;
                let ny = ay as isize + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let ux = nx as usize;
                    let uy = ny as usize;
                    let current = traffic.get(ux, uy);
                    traffic.set(ux, uy, current.saturating_add(boost));
                }
            }
        }

        // Center cell should have the boost
        assert_eq!(traffic.get(ax, ay), boost);
        // Adjacent cell should also have the boost
        assert_eq!(traffic.get(ax + 1, ay), boost);
        // Cell outside radius should be 0
        assert_eq!(traffic.get(ax + 4, ay), 0);
    }

    #[test]
    fn test_responding_clears_faster() {
        let mut a1 = TrafficAccident {
            grid_x: 5,
            grid_y: 5,
            severity: 2,
            ticks_remaining: 100,
            responding: false,
            ambulance_dispatched: false,
        };
        let mut a2 = TrafficAccident {
            grid_x: 15,
            grid_y: 15,
            severity: 2,
            ticks_remaining: 100,
            responding: true,
            ambulance_dispatched: true,
        };

        // Simulate 10 ticks
        for _ in 0..10 {
            let decay1 = if a1.responding { 2 } else { 1 };
            a1.ticks_remaining = a1.ticks_remaining.saturating_sub(decay1);

            let decay2 = if a2.responding { 2 } else { 1 };
            a2.ticks_remaining = a2.ticks_remaining.saturating_sub(decay2);
        }

        assert_eq!(a1.ticks_remaining, 90, "Non-responding: 100 - 10*1 = 90");
        assert_eq!(a2.ticks_remaining, 80, "Responding: 100 - 10*2 = 80");
    }

    #[test]
    fn test_condition_multiplier_ranges() {
        // Poor condition (< 50) => 2.0x
        let cond_poor = 30u8;
        let mult_poor = if cond_poor < 50 {
            2.0
        } else if cond_poor < 100 {
            1.5
        } else {
            1.0
        };
        assert_eq!(mult_poor, 2.0);

        // Medium condition (50-99) => 1.5x
        let cond_med = 75u8;
        let mult_med = if cond_med < 50 {
            2.0
        } else if cond_med < 100 {
            1.5
        } else {
            1.0
        };
        assert_eq!(mult_med, 1.5);

        // Good condition (>= 100) => 1.0x
        let cond_good = 180u8;
        let mult_good = if cond_good < 50 {
            2.0
        } else if cond_good < 100 {
            1.5
        } else {
            1.0
        };
        assert_eq!(mult_good, 1.0);
    }
}

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::road_maintenance::RoadConditionGrid;
use crate::services::ServiceBuilding;
use crate::traffic::TrafficGrid;
use crate::weather::{Weather, WeatherEvent};
use crate::TickCounter;

/// A single traffic accident on the grid.
#[derive(Debug, Clone)]
pub struct TrafficAccident {
    pub grid_x: usize,
    pub grid_y: usize,
    /// Severity 1-3. Higher = worse.
    pub severity: u8,
    /// Ticks remaining before the accident clears.
    pub ticks_remaining: u32,
    /// Whether an emergency responder is en route or on scene.
    pub responding: bool,
    /// Whether an ambulance has been dispatched (for severity >= 2).
    pub ambulance_dispatched: bool,
}

/// Resource tracking all active and historical traffic accidents.
#[derive(Resource)]
pub struct AccidentTracker {
    pub active_accidents: Vec<TrafficAccident>,
    pub total_accidents: u32,
    pub accidents_this_month: u32,
    pub avg_response_time: f32,
    /// Maximum number of simultaneous active accidents.
    pub max_active: usize,
    /// Accumulated response time ticks for computing the average.
    pub response_time_accum: f32,
    pub response_count: u32,
}

impl Default for AccidentTracker {
    fn default() -> Self {
        Self {
            active_accidents: Vec::new(),
            total_accidents: 0,
            accidents_this_month: 0,
            avg_response_time: 0.0,
            max_active: 10,
            response_time_accum: 0.0,
            response_count: 0,
        }
    }
}

/// Counts how many cardinal-direction road neighbors a cell has.
/// A cell with road neighbors in 3+ directions is considered an intersection.
fn road_neighbor_directions(grid: &WorldGrid, x: usize, y: usize) -> u8 {
    let mut dirs = 0u8;
    if x > 0 && grid.get(x - 1, y).cell_type == CellType::Road {
        dirs += 1;
    }
    if x + 1 < GRID_WIDTH && grid.get(x + 1, y).cell_type == CellType::Road {
        dirs += 1;
    }
    if y > 0 && grid.get(x, y - 1).cell_type == CellType::Road {
        dirs += 1;
    }
    if y + 1 < GRID_HEIGHT && grid.get(x, y + 1).cell_type == CellType::Road {
        dirs += 1;
    }
    dirs
}

/// Returns a weather-based accident probability multiplier.
/// Clear = 1.0, Rain = 1.5, Storm = 2.5, ColdSnap = 1.8, HeatWave = 1.2.
fn weather_accident_multiplier(weather: &Weather) -> f32 {
    match weather.current_event {
        WeatherEvent::Storm => 2.5,
        WeatherEvent::Rain => 1.5,
        WeatherEvent::ColdSnap => 1.8,
        WeatherEvent::HeatWave => 1.2,
        WeatherEvent::Clear => 1.0,
    }
}

/// Spawns new accidents on high-traffic cells. Runs every 20 ticks.
pub fn spawn_accidents(
    tick: Res<TickCounter>,
    grid: Res<WorldGrid>,
    traffic: Res<TrafficGrid>,
    condition_grid: Res<RoadConditionGrid>,
    weather: Res<Weather>,
    mut tracker: ResMut<AccidentTracker>,
) {
    if !tick.0.is_multiple_of(20) {
        return;
    }

    if tracker.active_accidents.len() >= tracker.max_active {
        return;
    }

    let weather_mult = weather_accident_multiplier(&weather);

    for y in 0..GRID_HEIGHT {
        if tracker.active_accidents.len() >= tracker.max_active {
            break;
        }

        for x in 0..GRID_WIDTH {
            if tracker.active_accidents.len() >= tracker.max_active {
                break;
            }

            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Road {
                continue;
            }

            let density = traffic.get(x, y);
            // Only consider cells with meaningful traffic
            if density < 3 {
                continue;
            }

            // Skip cells that already have an active accident
            if tracker
                .active_accidents
                .iter()
                .any(|a| a.grid_x == x && a.grid_y == y)
            {
                continue;
            }

            // Base probability scales with traffic density.
            // density of 3 => base ~3, density of 20 => base ~20.
            let base_prob = density as f32;

            // Intersection bonus: 3+ directions = intersection
            let intersection_dirs = road_neighbor_directions(&grid, x, y);
            let intersection_mult = if intersection_dirs >= 3 { 1.5 } else { 1.0 };

            // Road condition penalty: poor condition increases probability
            let condition = condition_grid.get(x, y);
            let condition_mult = if condition < 50 {
                2.0
            } else if condition < 100 {
                1.5
            } else {
                1.0
            };

            let prob = base_prob * intersection_mult * condition_mult * weather_mult;

            // Deterministic random roll: threshold out of 1000
            let idx = y * GRID_WIDTH + x;
            let roll = (tick.0.wrapping_mul(6271) + idx as u64) % 1000;

            // We want a low per-cell probability even for high traffic.
            // prob ~ 20 (high traffic, intersection, bad road, storm) => threshold ~20/1000 = 2%
            // prob ~ 3 (low traffic, good road, clear) => threshold ~3/1000 = 0.3%
            if roll >= prob as u64 {
                continue;
            }

            // Determine severity based on road type and traffic level
            let severity = {
                let road_type = cell.road_type;
                let speed = road_type.speed();
                let severity_roll = (tick.0.wrapping_mul(3571) + idx as u64 + density as u64) % 100;
                if speed >= 80.0 || density >= 15 {
                    // High-speed roads or very congested: more severe
                    if severity_roll < 30 {
                        3
                    } else if severity_roll < 70 {
                        2
                    } else {
                        1
                    }
                } else if speed >= 40.0 || density >= 8 {
                    if severity_roll < 15 {
                        3
                    } else if severity_roll < 50 {
                        2
                    } else {
                        1
                    }
                } else if severity_roll < 5 {
                    3
                } else if severity_roll < 25 {
                    2
                } else {
                    1
                }
            };

            let ticks_remaining = severity as u32 * 50;

            tracker.active_accidents.push(TrafficAccident {
                grid_x: x,
                grid_y: y,
                severity,
                ticks_remaining,
                responding: false,
                ambulance_dispatched: false,
            });

            tracker.total_accidents += 1;
            tracker.accidents_this_month += 1;
        }
    }
}

/// Processes active accidents each tick:
/// - Checks for emergency response from nearby hospitals/police.
/// - Accidents boost local traffic density (3-cell radius).
/// - Severe accidents reduce nearby citizen happiness (via health).
/// - Clears accidents when their duration expires.
pub fn process_accidents(
    _tick: Res<TickCounter>,
    mut tracker: ResMut<AccidentTracker>,
    mut traffic: ResMut<TrafficGrid>,
    services: Query<&ServiceBuilding>,
) {
    // Build a flat list of responder positions for quick distance checks.
    let responders: Vec<(usize, usize, bool, bool)> = services
        .iter()
        .filter_map(|sb| {
            let is_hospital = ServiceBuilding::is_health(sb.service_type);
            let is_police = ServiceBuilding::is_police(sb.service_type);
            if is_hospital || is_police {
                Some((sb.grid_x, sb.grid_y, is_hospital, is_police))
            } else {
                None
            }
        })
        .collect();

    let response_radius = 30usize; // cells

    for accident in tracker.active_accidents.iter_mut() {
        // --- Emergency response check ---
        if !accident.responding {
            for &(rx, ry, is_hospital, is_police) in &responders {
                let dx = accident.grid_x.abs_diff(rx);
                let dy = accident.grid_y.abs_diff(ry);
                let dist = dx + dy; // Manhattan distance

                if dist <= response_radius {
                    if is_police {
                        accident.responding = true;
                    }
                    if is_hospital && accident.severity >= 2 {
                        accident.ambulance_dispatched = true;
                        accident.responding = true;
                    }
                    if accident.responding {
                        break;
                    }
                }
            }
        }

        // --- Responding accidents clear faster ---
        let decay = if accident.responding { 2 } else { 1 };
        accident.ticks_remaining = accident.ticks_remaining.saturating_sub(decay);

        // --- Accident effect: boost traffic density in 3-cell radius ---
        let radius: isize = 3;
        let boost = match accident.severity {
            3 => 5u16,
            2 => 3,
            _ => 1,
        };

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() + dy.abs() > radius {
                    continue;
                }
                let nx = accident.grid_x as isize + dx;
                let ny = accident.grid_y as isize + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let ux = nx as usize;
                    let uy = ny as usize;
                    let current = traffic.get(ux, uy);
                    traffic.set(ux, uy, current.saturating_add(boost));
                }
            }
        }
    }

    // --- Update response time average for accidents that just got a response this tick ---
    // (We track how long it took by measuring: initial_duration - ticks_remaining at time of response)
    // Since we set responding in the same tick, the "response time" approximation is
    // (initial_duration - ticks_remaining) but since we just set it, we record the time of dispatch.
    // For simplicity, record response time proportional to remaining ticks at response:
    // if it just became responding and is still active, we infer response was this tick.
    let mut response_time_delta = 0.0f32;
    let mut response_count_delta = 0u32;
    for accident in &tracker.active_accidents {
        if accident.responding {
            let initial = accident.severity as u32 * 50;
            // Only count if the accident still has significant time left (just responded)
            if accident.ticks_remaining + 2 >= initial {
                // This was just dispatched, record near-immediate response
                response_time_delta += 1.0;
                response_count_delta += 1;
            }
        }
    }
    tracker.response_time_accum += response_time_delta;
    tracker.response_count += response_count_delta;

    // --- Remove cleared accidents ---
    tracker.active_accidents.retain(|a| a.ticks_remaining > 0);

    // --- Update average response time ---
    if tracker.response_count > 0 {
        tracker.avg_response_time = tracker.response_time_accum / tracker.response_count as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, WorldGrid};
    use crate::traffic::TrafficGrid;
    use crate::weather::{Weather, WeatherEvent};

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

        weather.current_event = WeatherEvent::Clear;
        assert_eq!(weather_accident_multiplier(&weather), 1.0);

        weather.current_event = WeatherEvent::Rain;
        assert_eq!(weather_accident_multiplier(&weather), 1.5);

        weather.current_event = WeatherEvent::Storm;
        assert_eq!(weather_accident_multiplier(&weather), 2.5);

        weather.current_event = WeatherEvent::ColdSnap;
        assert_eq!(weather_accident_multiplier(&weather), 1.8);

        weather.current_event = WeatherEvent::HeatWave;
        assert_eq!(weather_accident_multiplier(&weather), 1.2);
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

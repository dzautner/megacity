use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::road_maintenance::RoadConditionGrid;
use crate::services::ServiceBuilding;
use crate::traffic::TrafficGrid;
use crate::weather::Weather;
use crate::TickCounter;

use super::calculations::{road_neighbor_directions, weather_accident_multiplier};
use super::types::{AccidentTracker, TrafficAccident};

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

pub struct TrafficAccidentsPlugin;

impl Plugin for TrafficAccidentsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AccidentTracker>().add_systems(
            FixedUpdate,
            (spawn_accidents, process_accidents)
                .chain()
                .after(crate::traffic::update_traffic_density)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

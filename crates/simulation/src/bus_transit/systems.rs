//! ECS systems for the bus transit simulation.

use bevy::prelude::*;

use crate::economy::CityBudget;
use crate::grid::WorldGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::time_of_day::GameClock;
use crate::SlowTickTimer;

use super::state::manhattan_distance;
use super::types::*;

// =============================================================================
// Systems
// =============================================================================

/// System: check depot coverage and activate/deactivate routes.
///
/// A route is active if at least one BusDepot service building exists within
/// coverage radius of any stop on the route.
pub fn update_route_activation(
    mut transit: ResMut<BusTransitState>,
    services: Query<&ServiceBuilding>,
) {
    let depots: Vec<(usize, usize, f32)> = services
        .iter()
        .filter(|s| s.service_type == ServiceType::BusDepot)
        .map(|s| (s.grid_x, s.grid_y, s.radius))
        .collect();

    // Pre-collect stop coordinates into a map to avoid borrow conflict
    let stop_coords: Vec<(BusStopId, usize, usize)> = transit
        .stops
        .iter()
        .map(|s| (s.id, s.grid_x, s.grid_y))
        .collect();

    // Collect deactivated route IDs to remove their buses afterward
    let mut deactivated_routes: Vec<BusRouteId> = Vec::new();

    for route in &mut transit.routes {
        let was_active = route.active;
        route.active = false;

        // A route is active if any of its stops is within depot coverage
        'stop_loop: for stop_id in &route.stop_ids {
            if let Some(&(_, sx, sy)) = stop_coords.iter().find(|(id, _, _)| id == stop_id) {
                for &(dx, dy, radius) in &depots {
                    let dist = manhattan_distance(sx, sy, dx, dy) as f32;
                    if dist * crate::config::CELL_SIZE <= radius {
                        route.active = true;
                        break 'stop_loop;
                    }
                }
            }
        }

        // If route just became inactive, mark for bus removal
        if was_active && !route.active {
            deactivated_routes.push(route.id);
        }
    }

    // Remove buses for deactivated routes
    for route_id in deactivated_routes {
        transit.buses.retain(|b| b.route_id != route_id);
    }
}

/// System: spawn buses on active routes that don't have enough buses.
pub fn spawn_buses(mut transit: ResMut<BusTransitState>) {
    // Pre-collect stop coordinates
    let stop_positions: Vec<(BusStopId, f32, f32)> = transit
        .stops
        .iter()
        .map(|s| (s.id, s.grid_x as f32, s.grid_y as f32))
        .collect();

    // Collect spawn info first to avoid borrow conflicts
    struct SpawnInfo {
        route_id: BusRouteId,
        current_count: usize,
        target_count: usize,
        positions: Vec<(f32, f32)>,
    }

    let spawn_infos: Vec<SpawnInfo> = transit
        .routes
        .iter()
        .filter(|r| r.active && !r.stop_ids.is_empty())
        .map(|r| {
            let current_count = transit.buses.iter().filter(|b| b.route_id == r.id).count();
            let positions: Vec<(f32, f32)> = r
                .stop_ids
                .iter()
                .filter_map(|sid| {
                    stop_positions
                        .iter()
                        .find(|(id, _, _)| id == sid)
                        .map(|(_, x, y)| (*x, *y))
                })
                .collect();
            SpawnInfo {
                route_id: r.id,
                current_count,
                target_count: BUSES_PER_ROUTE as usize,
                positions,
            }
        })
        .filter(|info| info.current_count < info.target_count)
        .collect();

    // Now spawn buses
    for info in spawn_infos {
        if info.positions.is_empty() {
            continue;
        }
        let num_stops = info.positions.len();
        for i in info.current_count..info.target_count {
            // Offset buses along the route so they don't bunch
            let stop_idx = i % num_stops;
            let (sx, sy) = info.positions[stop_idx];
            let next_idx = (stop_idx + 1) % num_stops;

            transit.buses.push(BusInstance {
                route_id: info.route_id,
                next_stop_index: next_idx,
                grid_x: sx,
                grid_y: sy,
                passengers: 0,
                dwell_ticks: 0,
                at_stop: false,
            });
        }
    }
}

/// System: move buses along their routes and handle passenger pickup/dropoff.
pub fn update_buses(mut transit: ResMut<BusTransitState>) {
    // Pre-collect route stop data to avoid borrow conflicts
    struct RouteStopData {
        route_id: BusRouteId,
        stops: Vec<(BusStopId, f32, f32)>,
    }

    let route_data: Vec<RouteStopData> = transit
        .routes
        .iter()
        .filter(|r| r.active)
        .map(|r| {
            let stops: Vec<(BusStopId, f32, f32)> = r
                .stop_ids
                .iter()
                .filter_map(|sid| {
                    transit
                        .stops
                        .iter()
                        .find(|s| s.id == *sid)
                        .map(|s| (s.id, s.grid_x as f32, s.grid_y as f32))
                })
                .collect();
            RouteStopData {
                route_id: r.id,
                stops,
            }
        })
        .collect();

    // Pre-collect waiting counts to avoid borrow conflict with transit.buses
    let waiting_counts: Vec<(BusStopId, u32)> =
        transit.stops.iter().map(|s| (s.id, s.waiting)).collect();

    // Track changes to apply after iteration
    let mut ridership_increments: Vec<(BusRouteId, u32)> = Vec::new();
    let mut fare_revenue = 0.0_f64;
    let mut stop_waiting_decrements: Vec<(BusStopId, u32)> = Vec::new();

    for bus in &mut transit.buses {
        let Some(rd) = route_data.iter().find(|d| d.route_id == bus.route_id) else {
            continue;
        };
        if rd.stops.is_empty() {
            continue;
        }

        // Handle dwelling at stop
        if bus.at_stop {
            if bus.dwell_ticks > 0 {
                bus.dwell_ticks -= 1;
                continue;
            }
            bus.at_stop = false;
            // Move to next stop
            bus.next_stop_index = (bus.next_stop_index + 1) % rd.stops.len();
        }

        // Move toward next stop
        let next = &rd.stops[bus.next_stop_index % rd.stops.len()];
        let dx = next.1 - bus.grid_x;
        let dy = next.2 - bus.grid_y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < BUS_SPEED_CELLS_PER_TICK {
            // Arrived at stop
            bus.grid_x = next.1;
            bus.grid_y = next.2;
            bus.at_stop = true;
            bus.dwell_ticks = 3; // Short dwell time

            // Drop off passengers (some fraction disembark at each stop)
            let dropoff = (bus.passengers / 3).max(1).min(bus.passengers);
            bus.passengers = bus.passengers.saturating_sub(dropoff);

            // Pick up waiting passengers — use pre-collected waiting counts
            let stop_id = next.0;
            let waiting = waiting_counts
                .iter()
                .find(|(id, _)| *id == stop_id)
                .map(|(_, w)| *w)
                .unwrap_or(0);
            let space = BUS_CAPACITY.saturating_sub(bus.passengers);
            let pickup = waiting.min(space);
            if pickup > 0 {
                bus.passengers += pickup;
                fare_revenue += pickup as f64 * FARE_PER_RIDE;
                stop_waiting_decrements.push((stop_id, pickup));
                ridership_increments.push((bus.route_id, pickup));
            }
        } else {
            // Move toward stop
            let norm = 1.0 / dist;
            bus.grid_x += dx * norm * BUS_SPEED_CELLS_PER_TICK;
            bus.grid_y += dy * norm * BUS_SPEED_CELLS_PER_TICK;
        }
    }

    // Apply waiting decrements
    for (stop_id, decrement) in stop_waiting_decrements {
        if let Some(stop) = transit.stops.iter_mut().find(|s| s.id == stop_id) {
            stop.waiting = stop.waiting.saturating_sub(decrement);
        }
    }

    // Apply ridership increments
    for (route_id, count) in ridership_increments {
        if let Some(route) = transit.routes.iter_mut().find(|r| r.id == route_id) {
            route.total_ridership += count as u64;
            route.monthly_ridership += count;
        }
    }

    transit.monthly_fare_revenue += fare_revenue;
}

/// System: apply bus transit costs and revenue to the city budget.
/// Runs on slow tick (every 100 ticks).
pub fn apply_transit_costs(
    timer: Res<SlowTickTimer>,
    clock: Res<GameClock>,
    mut transit: ResMut<BusTransitState>,
    mut budget: ResMut<CityBudget>,
) {
    if !timer.should_run() {
        return;
    }

    // Apply costs monthly (every 30 days)
    if clock.day <= transit.last_cost_day + 30 {
        return;
    }
    transit.last_cost_day = clock.day;

    // Route maintenance costs
    let route_cost = transit.routes.iter().filter(|r| r.active).count() as f64 * ROUTE_MONTHLY_COST;

    // Bus operating costs
    let bus_cost = transit.buses.len() as f64 * BUS_MONTHLY_COST;

    let total_cost = route_cost + bus_cost;
    transit.monthly_operating_cost = total_cost;

    // Apply to budget: deduct costs, add fare revenue
    budget.treasury -= total_cost;
    budget.treasury += transit.monthly_fare_revenue;

    // Reset monthly counters
    transit.monthly_fare_revenue = 0.0;
    for route in &mut transit.routes {
        route.monthly_ridership = 0;
    }
}

/// System: simulate citizens arriving at bus stops (simplified model).
///
/// Each slow tick, a fraction of citizens near active bus stops are added
/// as waiting passengers. This is a simplified model — full citizen mode
/// choice integration would modify the movement system's pathfinding.
pub fn simulate_waiting_citizens(
    timer: Res<SlowTickTimer>,
    mut transit: ResMut<BusTransitState>,
    grid: Res<WorldGrid>,
) {
    if !timer.should_run() {
        return;
    }

    // Pre-collect active route stop IDs to avoid borrow conflicts
    let active_stop_ids: Vec<BusStopId> = transit
        .routes
        .iter()
        .filter(|r| r.active)
        .flat_map(|r| r.stop_ids.iter().copied())
        .collect();

    for stop in &mut transit.stops {
        let on_active_route = active_stop_ids.contains(&stop.id);

        if !on_active_route {
            stop.waiting = 0;
            continue;
        }

        // Count nearby zoned cells as demand proxy
        let mut demand = 0u32;
        let range = 5i32;
        for dy in -range..=range {
            for dx in -range..=range {
                let nx = stop.grid_x as i32 + dx;
                let ny = stop.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && grid.in_bounds(nx as usize, ny as usize) {
                    let cell = grid.get(nx as usize, ny as usize);
                    if cell.zone != crate::grid::ZoneType::None {
                        demand += 1;
                    }
                }
            }
        }

        // Add waiting passengers proportional to demand (capped)
        let new_waiting = (demand / 10).min(5);
        stop.waiting = (stop.waiting + new_waiting).min(BUS_CAPACITY * 2);
    }
}

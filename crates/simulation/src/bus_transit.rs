//! TRAF-005: Bus Public Transit System
//!
//! Implements bus lines as the first public transit type. Buses follow
//! player-drawn routes with stops, pick up citizens, and reduce car traffic.
//!
//! ## Data model
//! - `BusStop`: a stop placed on a road cell (grid coords)
//! - `BusRoute`: an ordered sequence of bus stop IDs
//! - `Bus`: an entity that travels along a route, picking up/dropping off passengers
//! - `BusTransitState`: top-level resource storing all stops, routes, and stats
//!
//! ## Costs
//! - $400/month per route + $100/month per active bus
//! - Fare revenue: $2 per ride
//!
//! ## Citizen mode choice
//! Citizens choose bus when: walk_to_stop + wait + ride + walk_from_stop < drive_time * 1.3

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::economy::CityBudget;
use crate::grid::{CellType, WorldGrid};
use crate::services::{ServiceBuilding, ServiceType};
use crate::time_of_day::GameClock;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Maximum passengers per bus.
pub const BUS_CAPACITY: u32 = 30;

/// Monthly cost per bus route.
pub const ROUTE_MONTHLY_COST: f64 = 400.0;

/// Monthly cost per active bus.
pub const BUS_MONTHLY_COST: f64 = 100.0;

/// Fare per ride (revenue).
pub const FARE_PER_RIDE: f64 = 2.0;

/// Number of buses spawned per route.
pub const BUSES_PER_ROUTE: u32 = 2;

/// Maximum number of stops per route.
pub const MAX_STOPS_PER_ROUTE: usize = 20;

/// Maximum walking distance (in grid cells) to consider a bus stop reachable.
pub const MAX_WALK_DISTANCE: u32 = 10;

/// Average wait time at a stop in ticks (simulated).
pub const AVERAGE_WAIT_TICKS: u32 = 20;

/// Bus speed in grid cells per tick (slower than cars, affected by traffic).
pub const BUS_SPEED_CELLS_PER_TICK: f32 = 0.3;

// =============================================================================
// Data structures
// =============================================================================

/// Unique identifier for a bus stop.
pub type BusStopId = u32;

/// Unique identifier for a bus route.
pub type BusRouteId = u32;

/// A bus stop placed on a road cell.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BusStop {
    pub id: BusStopId,
    pub grid_x: usize,
    pub grid_y: usize,
    /// Number of citizens currently waiting at this stop.
    pub waiting: u32,
}

/// A bus route connecting an ordered sequence of bus stops.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BusRoute {
    pub id: BusRouteId,
    /// Human-readable name for the route.
    pub name: String,
    /// Ordered list of bus stop IDs along this route.
    pub stop_ids: Vec<BusStopId>,
    /// Whether this route is currently active (has a depot nearby).
    pub active: bool,
    /// Total lifetime ridership on this route.
    pub total_ridership: u64,
    /// Ridership in the current month.
    pub monthly_ridership: u32,
}

/// A bus entity traveling along a route.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BusInstance {
    pub route_id: BusRouteId,
    /// Index into the route's stop_ids for the next stop.
    pub next_stop_index: usize,
    /// Current grid position (fractional for smooth movement).
    pub grid_x: f32,
    pub grid_y: f32,
    /// Number of passengers currently on board.
    pub passengers: u32,
    /// Ticks remaining at current stop (for dwell time).
    pub dwell_ticks: u32,
    /// Whether the bus is currently dwelling at a stop.
    pub at_stop: bool,
}

// =============================================================================
// Resource: BusTransitState
// =============================================================================

/// Top-level resource for the bus transit system.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct BusTransitState {
    /// All bus stops, keyed by ID.
    pub stops: Vec<BusStop>,
    /// All bus routes.
    pub routes: Vec<BusRoute>,
    /// All active bus instances.
    pub buses: Vec<BusInstance>,
    /// Next ID for bus stops.
    pub next_stop_id: BusStopId,
    /// Next ID for bus routes.
    pub next_route_id: BusRouteId,
    /// Total fare revenue this month.
    pub monthly_fare_revenue: f64,
    /// Total operating cost this month.
    pub monthly_operating_cost: f64,
    /// Last day costs were applied.
    pub last_cost_day: u32,
}

// =============================================================================
// Public API for placing stops and creating routes
// =============================================================================

impl BusTransitState {
    /// Place a bus stop on a road cell. Returns the new stop ID, or None if
    /// the cell is not a road or a stop already exists there.
    pub fn add_stop(&mut self, grid: &WorldGrid, x: usize, y: usize) -> Option<BusStopId> {
        if !grid.in_bounds(x, y) || grid.get(x, y).cell_type != CellType::Road {
            return None;
        }
        // Check for duplicate stop at same location
        if self.stops.iter().any(|s| s.grid_x == x && s.grid_y == y) {
            return None;
        }
        let id = self.next_stop_id;
        self.next_stop_id += 1;
        self.stops.push(BusStop {
            id,
            grid_x: x,
            grid_y: y,
            waiting: 0,
        });
        Some(id)
    }

    /// Remove a bus stop by ID. Also removes the stop from any routes.
    pub fn remove_stop(&mut self, stop_id: BusStopId) {
        self.stops.retain(|s| s.id != stop_id);
        for route in &mut self.routes {
            route.stop_ids.retain(|&id| id != stop_id);
        }
        // Remove routes that now have fewer than 2 stops
        let removed_route_ids: Vec<BusRouteId> = self
            .routes
            .iter()
            .filter(|r| r.stop_ids.len() < 2)
            .map(|r| r.id)
            .collect();
        for route_id in &removed_route_ids {
            self.remove_route(*route_id);
        }
    }

    /// Create a new bus route from an ordered list of stop IDs.
    /// Returns the route ID, or None if fewer than 2 valid stops.
    pub fn add_route(&mut self, name: String, stop_ids: Vec<BusStopId>) -> Option<BusRouteId> {
        // Validate all stop IDs exist
        let valid_stops: Vec<BusStopId> = stop_ids
            .into_iter()
            .filter(|id| self.stops.iter().any(|s| s.id == *id))
            .collect();

        if valid_stops.len() < 2 || valid_stops.len() > MAX_STOPS_PER_ROUTE {
            return None;
        }

        let id = self.next_route_id;
        self.next_route_id += 1;
        self.routes.push(BusRoute {
            id,
            name,
            stop_ids: valid_stops,
            active: false, // activated by depot check
            total_ridership: 0,
            monthly_ridership: 0,
        });
        Some(id)
    }

    /// Remove a bus route and its buses.
    pub fn remove_route(&mut self, route_id: BusRouteId) {
        self.routes.retain(|r| r.id != route_id);
        self.buses.retain(|b| b.route_id != route_id);
    }

    /// Find the bus stop nearest to the given grid position within MAX_WALK_DISTANCE.
    pub fn nearest_stop(&self, gx: usize, gy: usize) -> Option<&BusStop> {
        self.stops
            .iter()
            .filter(|s| {
                let dx = (s.grid_x as i32 - gx as i32).unsigned_abs();
                let dy = (s.grid_y as i32 - gy as i32).unsigned_abs();
                dx + dy <= MAX_WALK_DISTANCE
            })
            .min_by_key(|s| {
                let dx = (s.grid_x as i32 - gx as i32).unsigned_abs();
                let dy = (s.grid_y as i32 - gy as i32).unsigned_abs();
                dx + dy
            })
    }

    /// Find the bus stop nearest to a destination, considering only stops
    /// on active routes.
    pub fn nearest_active_stop(&self, gx: usize, gy: usize) -> Option<&BusStop> {
        let active_route_stop_ids: Vec<BusStopId> = self
            .routes
            .iter()
            .filter(|r| r.active)
            .flat_map(|r| r.stop_ids.iter().copied())
            .collect();

        self.stops
            .iter()
            .filter(|s| active_route_stop_ids.contains(&s.id))
            .filter(|s| {
                let dx = (s.grid_x as i32 - gx as i32).unsigned_abs();
                let dy = (s.grid_y as i32 - gy as i32).unsigned_abs();
                dx + dy <= MAX_WALK_DISTANCE
            })
            .min_by_key(|s| {
                let dx = (s.grid_x as i32 - gx as i32).unsigned_abs();
                let dy = (s.grid_y as i32 - gy as i32).unsigned_abs();
                dx + dy
            })
    }

    /// Estimate transit time in ticks for traveling from (sx,sy) to (dx,dy)
    /// using the bus system. Returns None if no viable route exists.
    pub fn estimate_transit_time(
        &self,
        src_x: usize,
        src_y: usize,
        dst_x: usize,
        dst_y: usize,
    ) -> Option<u32> {
        let origin_stop = self.nearest_active_stop(src_x, src_y)?;
        let dest_stop = self.nearest_active_stop(dst_x, dst_y)?;

        // Walk to origin stop
        let walk_to = manhattan_distance(src_x, src_y, origin_stop.grid_x, origin_stop.grid_y);

        // Check if both stops are on the same route
        let _shared_route = self.routes.iter().find(|r| {
            r.active && r.stop_ids.contains(&origin_stop.id) && r.stop_ids.contains(&dest_stop.id)
        })?;

        // Ride distance (Manhattan between stops as approximation)
        let ride_dist = manhattan_distance(
            origin_stop.grid_x,
            origin_stop.grid_y,
            dest_stop.grid_x,
            dest_stop.grid_y,
        );
        let ride_ticks = (ride_dist as f32 / BUS_SPEED_CELLS_PER_TICK) as u32;

        // Walk from destination stop
        let walk_from = manhattan_distance(dest_stop.grid_x, dest_stop.grid_y, dst_x, dst_y);

        // Total: walk + wait + ride + walk
        Some(walk_to + AVERAGE_WAIT_TICKS + ride_ticks + walk_from)
    }

    /// Get total number of active routes.
    pub fn active_route_count(&self) -> usize {
        self.routes.iter().filter(|r| r.active).count()
    }

    /// Get total ridership across all routes.
    pub fn total_ridership(&self) -> u64 {
        self.routes.iter().map(|r| r.total_ridership).sum()
    }

    /// Get the stop by ID.
    pub fn stop_by_id(&self, id: BusStopId) -> Option<&BusStop> {
        self.stops.iter().find(|s| s.id == id)
    }
}

/// Manhattan distance between two grid cells.
fn manhattan_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> u32 {
    let dx = (x1 as i32 - x2 as i32).unsigned_abs();
    let dy = (y1 as i32 - y2 as i32).unsigned_abs();
    dx + dy
}

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

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for BusTransitState {
    const SAVE_KEY: &'static str = "bus_transit";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.stops.is_empty() && self.routes.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct BusTransitPlugin;

impl Plugin for BusTransitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BusTransitState>().add_systems(
            FixedUpdate,
            (
                update_route_activation,
                spawn_buses,
                update_buses,
                apply_transit_costs,
                simulate_waiting_citizens,
            )
                .chain()
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<BusTransitState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::WorldGrid;
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

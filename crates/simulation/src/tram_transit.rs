//! TRAF-014: Tram/Light Rail Transit System
//!
//! Implements a tram/light rail system that shares road space with other traffic.
//! Trams have higher capacity than buses (90 passengers) and run on fixed lines
//! between stops placed on road cells.
//!
//! ## Data model
//! - `TramStop`: a stop placed on a road cell (grid coords, passenger queue)
//! - `TramLine`: a named sequence of tram stops with active flag
//! - `TramVehicle`: a tram entity traveling along a line, picking up/dropping off
//! - `TramTransitState`: top-level resource storing all stops, lines, vehicles, and stats
//!
//! ## Costs
//! - $600/week per active line
//! - $200/week per tram depot
//! - Fare revenue: $2.50 per ride
//!
//! ## Depot requirement
//! A tram line is only active if at least one `TramDepot` service building
//! exists within coverage radius of any stop on that line.

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

/// Maximum passengers per tram vehicle.
pub const TRAM_CAPACITY: u32 = 90;

/// Weekly cost per active tram line.
pub const LINE_WEEKLY_COST: f64 = 600.0;

/// Weekly cost per tram depot.
pub const DEPOT_WEEKLY_COST: f64 = 200.0;

/// Fare per ride (revenue).
pub const FARE_PER_RIDE: f64 = 2.50;

/// Number of tram vehicles spawned per active line.
pub const TRAMS_PER_LINE: u32 = 2;

/// Maximum number of stops per line.
pub const MAX_STOPS_PER_LINE: usize = 25;

/// Maximum walking distance (in grid cells) to consider a tram stop reachable.
pub const MAX_WALK_DISTANCE: u32 = 12;

/// Average wait time at a stop in ticks (simulated).
pub const AVERAGE_WAIT_TICKS: u32 = 15;

/// Tram speed in grid cells per tick (slightly faster than buses, affected by traffic).
pub const TRAM_SPEED_CELLS_PER_TICK: f32 = 0.35;

/// Dwell time at each stop in ticks.
pub const DWELL_TICKS: u32 = 4;

// =============================================================================
// Data structures
// =============================================================================

/// Unique identifier for a tram stop.
pub type TramStopId = u32;

/// Unique identifier for a tram line.
pub type TramLineId = u32;

/// A tram stop placed on a road cell.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TramStop {
    pub id: TramStopId,
    pub grid_x: usize,
    pub grid_y: usize,
    /// Number of citizens currently waiting at this stop.
    pub waiting: u32,
}

/// A tram line connecting an ordered sequence of stops.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TramLine {
    pub id: TramLineId,
    /// Human-readable name for the line.
    pub name: String,
    /// Ordered list of tram stop IDs along this line.
    pub stop_ids: Vec<TramStopId>,
    /// Whether this line is currently active (has a depot nearby).
    pub active: bool,
    /// Total lifetime ridership on this line.
    pub total_ridership: u64,
    /// Ridership in the current period.
    pub period_ridership: u32,
}

/// A tram vehicle traveling along a line.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TramVehicle {
    pub line_id: TramLineId,
    /// Index into the line's stop_ids for the next stop.
    pub next_stop_index: usize,
    /// Current grid position (fractional for smooth movement).
    pub grid_x: f32,
    pub grid_y: f32,
    /// Number of passengers currently on board.
    pub passengers: u32,
    /// Ticks remaining at current stop (for dwell time).
    pub dwell_ticks: u32,
    /// Whether the tram is currently dwelling at a stop.
    pub at_stop: bool,
}

// =============================================================================
// Resource: TramTransitState
// =============================================================================

/// Top-level resource for the tram transit system.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct TramTransitState {
    /// All tram stops.
    pub stops: Vec<TramStop>,
    /// All tram lines.
    pub lines: Vec<TramLine>,
    /// All active tram vehicles.
    pub trams: Vec<TramVehicle>,
    /// Next ID for tram stops.
    pub next_stop_id: TramStopId,
    /// Next ID for tram lines.
    pub next_line_id: TramLineId,
    /// Total fare revenue this period.
    pub period_fare_revenue: f64,
    /// Total operating cost this period.
    pub period_operating_cost: f64,
    /// Last day costs were applied.
    pub last_cost_day: u32,
    /// Cumulative ridership across all lines.
    pub cumulative_ridership: u64,
}

// =============================================================================
// TramTransitStats (separate stats resource)
// =============================================================================

/// Aggregate statistics for the tram network.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct TramTransitStats {
    /// Number of active tram lines.
    pub active_lines: u32,
    /// Total stops in the network.
    pub total_stops: u32,
    /// Estimated daily ridership.
    pub daily_ridership: u32,
    /// Monthly operating cost.
    pub monthly_operating_cost: f64,
    /// Monthly fare revenue.
    pub monthly_fare_revenue: f64,
    /// Total cumulative ridership.
    pub cumulative_ridership: u64,
}

// =============================================================================
// Public API
// =============================================================================

impl TramTransitState {
    /// Place a tram stop on a road cell. Returns the new stop ID, or None if
    /// the cell is not a road or a stop already exists there.
    pub fn add_stop(&mut self, grid: &WorldGrid, x: usize, y: usize) -> Option<TramStopId> {
        if !grid.in_bounds(x, y) || grid.get(x, y).cell_type != CellType::Road {
            return None;
        }
        // Check for duplicate stop at same location
        if self.stops.iter().any(|s| s.grid_x == x && s.grid_y == y) {
            return None;
        }
        let id = self.next_stop_id;
        self.next_stop_id += 1;
        self.stops.push(TramStop {
            id,
            grid_x: x,
            grid_y: y,
            waiting: 0,
        });
        Some(id)
    }

    /// Remove a tram stop by ID. Also removes the stop from any lines.
    pub fn remove_stop(&mut self, stop_id: TramStopId) {
        self.stops.retain(|s| s.id != stop_id);
        for line in &mut self.lines {
            line.stop_ids.retain(|&id| id != stop_id);
        }
        // Remove lines that now have fewer than 2 stops
        let removed_line_ids: Vec<TramLineId> = self
            .lines
            .iter()
            .filter(|l| l.stop_ids.len() < 2)
            .map(|l| l.id)
            .collect();
        for line_id in &removed_line_ids {
            self.remove_line(*line_id);
        }
    }

    /// Create a new tram line from an ordered list of stop IDs.
    /// Returns the line ID, or None if fewer than 2 valid stops.
    pub fn add_line(&mut self, name: String, stop_ids: Vec<TramStopId>) -> Option<TramLineId> {
        // Validate all stop IDs exist
        let valid_stops: Vec<TramStopId> = stop_ids
            .into_iter()
            .filter(|id| self.stops.iter().any(|s| s.id == *id))
            .collect();

        if valid_stops.len() < 2 || valid_stops.len() > MAX_STOPS_PER_LINE {
            return None;
        }

        let id = self.next_line_id;
        self.next_line_id += 1;
        self.lines.push(TramLine {
            id,
            name,
            stop_ids: valid_stops,
            active: false, // activated by depot check
            total_ridership: 0,
            period_ridership: 0,
        });
        Some(id)
    }

    /// Remove a tram line and its vehicles.
    pub fn remove_line(&mut self, line_id: TramLineId) {
        self.lines.retain(|l| l.id != line_id);
        self.trams.retain(|t| t.line_id != line_id);
    }

    /// Find the tram stop nearest to the given grid position within MAX_WALK_DISTANCE.
    pub fn nearest_stop(&self, gx: usize, gy: usize) -> Option<&TramStop> {
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

    /// Find the tram stop nearest to a destination, considering only stops on active lines.
    pub fn nearest_active_stop(&self, gx: usize, gy: usize) -> Option<&TramStop> {
        let active_line_stop_ids: Vec<TramStopId> = self
            .lines
            .iter()
            .filter(|l| l.active)
            .flat_map(|l| l.stop_ids.iter().copied())
            .collect();

        self.stops
            .iter()
            .filter(|s| active_line_stop_ids.contains(&s.id))
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
    /// using the tram system. Returns None if no viable route exists.
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

        // Check if both stops are on the same line
        let _shared_line = self.lines.iter().find(|l| {
            l.active && l.stop_ids.contains(&origin_stop.id) && l.stop_ids.contains(&dest_stop.id)
        })?;

        // Ride distance (Manhattan between stops as approximation)
        let ride_dist = manhattan_distance(
            origin_stop.grid_x,
            origin_stop.grid_y,
            dest_stop.grid_x,
            dest_stop.grid_y,
        );
        let ride_ticks = (ride_dist as f32 / TRAM_SPEED_CELLS_PER_TICK) as u32;

        // Walk from destination stop
        let walk_from = manhattan_distance(dest_stop.grid_x, dest_stop.grid_y, dst_x, dst_y);

        // Total: walk + wait + ride + walk
        Some(walk_to + AVERAGE_WAIT_TICKS + ride_ticks + walk_from)
    }

    /// Get total number of active lines.
    pub fn active_line_count(&self) -> usize {
        self.lines.iter().filter(|l| l.active).count()
    }

    /// Get total ridership across all lines.
    pub fn total_ridership(&self) -> u64 {
        self.lines.iter().map(|l| l.total_ridership).sum()
    }

    /// Get the stop by ID.
    pub fn stop_by_id(&self, id: TramStopId) -> Option<&TramStop> {
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

/// System: check depot coverage and activate/deactivate tram lines.
///
/// A tram line is active if at least one TramDepot service building exists within
/// coverage radius of any stop on the line.
pub fn update_tram_lines(mut transit: ResMut<TramTransitState>, services: Query<&ServiceBuilding>) {
    let depots: Vec<(usize, usize, f32)> = services
        .iter()
        .filter(|s| s.service_type == ServiceType::TramDepot)
        .map(|s| (s.grid_x, s.grid_y, s.radius))
        .collect();

    // Pre-collect stop coordinates to avoid borrow conflict
    let stop_coords: Vec<(TramStopId, usize, usize)> = transit
        .stops
        .iter()
        .map(|s| (s.id, s.grid_x, s.grid_y))
        .collect();

    let mut deactivated_lines: Vec<TramLineId> = Vec::new();

    for line in &mut transit.lines {
        let was_active = line.active;
        line.active = false;

        // A line is active if any of its stops is within depot coverage
        'stop_loop: for stop_id in &line.stop_ids {
            if let Some(&(_, sx, sy)) = stop_coords.iter().find(|(id, _, _)| id == stop_id) {
                for &(dx, dy, radius) in &depots {
                    let dist = manhattan_distance(sx, sy, dx, dy) as f32;
                    if dist * crate::config::CELL_SIZE <= radius {
                        line.active = true;
                        break 'stop_loop;
                    }
                }
            }
        }

        if was_active && !line.active {
            deactivated_lines.push(line.id);
        }
    }

    // Remove tram vehicles for deactivated lines
    for line_id in deactivated_lines {
        transit.trams.retain(|t| t.line_id != line_id);
    }

    // Spawn tram vehicles on active lines that don't have enough
    let stop_positions: Vec<(TramStopId, f32, f32)> = transit
        .stops
        .iter()
        .map(|s| (s.id, s.grid_x as f32, s.grid_y as f32))
        .collect();

    struct SpawnInfo {
        line_id: TramLineId,
        current_count: usize,
        target_count: usize,
        positions: Vec<(f32, f32)>,
    }

    let spawn_infos: Vec<SpawnInfo> = transit
        .lines
        .iter()
        .filter(|l| l.active && !l.stop_ids.is_empty())
        .map(|l| {
            let current_count = transit.trams.iter().filter(|t| t.line_id == l.id).count();
            let positions: Vec<(f32, f32)> = l
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
                line_id: l.id,
                current_count,
                target_count: TRAMS_PER_LINE as usize,
                positions,
            }
        })
        .filter(|info| info.current_count < info.target_count)
        .collect();

    for info in spawn_infos {
        if info.positions.is_empty() {
            continue;
        }
        let num_stops = info.positions.len();
        for i in info.current_count..info.target_count {
            let stop_idx = i % num_stops;
            let (sx, sy) = info.positions[stop_idx];
            let next_idx = (stop_idx + 1) % num_stops;

            transit.trams.push(TramVehicle {
                line_id: info.line_id,
                next_stop_index: next_idx,
                grid_x: sx,
                grid_y: sy,
                passengers: 0,
                dwell_ticks: 0,
                at_stop: false,
            });
        }
    }

    // Move trams along their lines and handle passenger pickup/dropoff
    struct LineStopData {
        line_id: TramLineId,
        stops: Vec<(TramStopId, f32, f32)>,
    }

    let line_data: Vec<LineStopData> = transit
        .lines
        .iter()
        .filter(|l| l.active)
        .map(|l| {
            let stops: Vec<(TramStopId, f32, f32)> = l
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
            LineStopData {
                line_id: l.id,
                stops,
            }
        })
        .collect();

    let waiting_counts: Vec<(TramStopId, u32)> =
        transit.stops.iter().map(|s| (s.id, s.waiting)).collect();

    let mut ridership_increments: Vec<(TramLineId, u32)> = Vec::new();
    let mut fare_revenue = 0.0_f64;
    let mut stop_waiting_decrements: Vec<(TramStopId, u32)> = Vec::new();

    for tram in &mut transit.trams {
        let Some(ld) = line_data.iter().find(|d| d.line_id == tram.line_id) else {
            continue;
        };
        if ld.stops.is_empty() {
            continue;
        }

        // Handle dwelling at stop
        if tram.at_stop {
            if tram.dwell_ticks > 0 {
                tram.dwell_ticks -= 1;
                continue;
            }
            tram.at_stop = false;
            tram.next_stop_index = (tram.next_stop_index + 1) % ld.stops.len();
        }

        // Move toward next stop
        let next = &ld.stops[tram.next_stop_index % ld.stops.len()];
        let dx = next.1 - tram.grid_x;
        let dy = next.2 - tram.grid_y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < TRAM_SPEED_CELLS_PER_TICK {
            // Arrived at stop
            tram.grid_x = next.1;
            tram.grid_y = next.2;
            tram.at_stop = true;
            tram.dwell_ticks = DWELL_TICKS;

            // Drop off passengers (some fraction disembark at each stop)
            let dropoff = (tram.passengers / 3).max(1).min(tram.passengers);
            tram.passengers = tram.passengers.saturating_sub(dropoff);

            // Pick up waiting passengers
            let stop_id = next.0;
            let waiting = waiting_counts
                .iter()
                .find(|(id, _)| *id == stop_id)
                .map(|(_, w)| *w)
                .unwrap_or(0);
            let space = TRAM_CAPACITY.saturating_sub(tram.passengers);
            let pickup = waiting.min(space);
            if pickup > 0 {
                tram.passengers += pickup;
                fare_revenue += pickup as f64 * FARE_PER_RIDE;
                stop_waiting_decrements.push((stop_id, pickup));
                ridership_increments.push((tram.line_id, pickup));
            }
        } else {
            // Move toward stop
            let norm = 1.0 / dist;
            tram.grid_x += dx * norm * TRAM_SPEED_CELLS_PER_TICK;
            tram.grid_y += dy * norm * TRAM_SPEED_CELLS_PER_TICK;
        }
    }

    // Apply waiting decrements
    for (stop_id, decrement) in stop_waiting_decrements {
        if let Some(stop) = transit.stops.iter_mut().find(|s| s.id == stop_id) {
            stop.waiting = stop.waiting.saturating_sub(decrement);
        }
    }

    // Apply ridership increments
    for (line_id, count) in ridership_increments {
        if let Some(line) = transit.lines.iter_mut().find(|l| l.id == line_id) {
            line.total_ridership += count as u64;
            line.period_ridership += count;
        }
    }

    transit.period_fare_revenue += fare_revenue;
    transit.cumulative_ridership = transit.total_ridership();
}

/// System: apply tram transit costs and revenue to the city budget.
/// Runs on slow tick (every 100 ticks).
pub fn update_tram_costs(
    timer: Res<SlowTickTimer>,
    clock: Res<GameClock>,
    mut transit: ResMut<TramTransitState>,
    mut budget: ResMut<CityBudget>,
    services: Query<&ServiceBuilding>,
) {
    if !timer.should_run() {
        return;
    }

    // Apply costs weekly (every 7 days)
    if clock.day <= transit.last_cost_day + 7 {
        return;
    }
    transit.last_cost_day = clock.day;

    // Line operating costs
    let line_cost = transit.lines.iter().filter(|l| l.active).count() as f64 * LINE_WEEKLY_COST;

    // Depot maintenance costs
    let depot_count = services
        .iter()
        .filter(|s| s.service_type == ServiceType::TramDepot)
        .count();
    let depot_cost = depot_count as f64 * DEPOT_WEEKLY_COST;

    let total_cost = line_cost + depot_cost;
    transit.period_operating_cost = total_cost;

    // Apply to budget: deduct costs, add fare revenue
    budget.treasury -= total_cost;
    budget.treasury += transit.period_fare_revenue;

    // Reset period counters
    transit.period_fare_revenue = 0.0;
    for line in &mut transit.lines {
        line.period_ridership = 0;
    }
}

/// System: simulate citizens arriving at tram stops (simplified model).
///
/// Each slow tick, a fraction of citizens near active tram stops are added
/// as waiting passengers. Trams have higher demand than buses due to
/// higher capacity and reliability.
pub fn tram_depot_coverage(
    timer: Res<SlowTickTimer>,
    mut transit: ResMut<TramTransitState>,
    grid: Res<WorldGrid>,
    mut stats: ResMut<TramTransitStats>,
) {
    if !timer.should_run() {
        return;
    }

    // Pre-collect active line stop IDs
    let active_stop_ids: Vec<TramStopId> = transit
        .lines
        .iter()
        .filter(|l| l.active)
        .flat_map(|l| l.stop_ids.iter().copied())
        .collect();

    for stop in &mut transit.stops {
        let on_active_line = active_stop_ids.contains(&stop.id);

        if !on_active_line {
            stop.waiting = 0;
            continue;
        }

        // Count nearby zoned cells as demand proxy
        let mut demand = 0u32;
        let range = 6i32; // Slightly wider catchment than bus stops
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

        // Higher demand multiplier than buses (trams attract more riders)
        let new_waiting = (demand / 8).min(8);
        stop.waiting = (stop.waiting + new_waiting).min(TRAM_CAPACITY * 2);
    }

    // Update aggregate stats
    stats.active_lines = transit.active_line_count() as u32;
    stats.total_stops = transit.stops.len() as u32;
    stats.daily_ridership = transit
        .lines
        .iter()
        .map(|l| l.period_ridership)
        .sum::<u32>();
    stats.monthly_operating_cost = transit.period_operating_cost * 4.0; // ~4 weeks
    stats.monthly_fare_revenue = transit.period_fare_revenue * 4.0;
    stats.cumulative_ridership = transit.cumulative_ridership;
}

// =============================================================================
// Saveable implementations
// =============================================================================

impl crate::Saveable for TramTransitState {
    const SAVE_KEY: &'static str = "tram_transit";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.stops.is_empty() && self.lines.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

impl crate::Saveable for TramTransitStats {
    const SAVE_KEY: &'static str = "tram_transit_stats";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.active_lines == 0 && self.total_stops == 0 && self.cumulative_ridership == 0 {
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

pub struct TramTransitPlugin;

impl Plugin for TramTransitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TramTransitState>()
            .init_resource::<TramTransitStats>()
            .add_systems(
                FixedUpdate,
                (update_tram_lines, update_tram_costs, tram_depot_coverage)
                    .chain()
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<TramTransitState>();
        registry.register::<TramTransitStats>();
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
        let mut state = TramTransitState::default();
        let id = state.add_stop(&grid, 5, 5);
        assert!(id.is_some());
        assert_eq!(state.stops.len(), 1);
        assert_eq!(state.stops[0].grid_x, 5);
        assert_eq!(state.stops[0].grid_y, 5);
    }

    #[test]
    fn test_add_stop_on_grass_fails() {
        let grid = WorldGrid::new(32, 32);
        let mut state = TramTransitState::default();
        let id = state.add_stop(&grid, 5, 5);
        assert!(id.is_none());
    }

    #[test]
    fn test_add_stop_duplicate_fails() {
        let grid = make_grid_with_road(5, 5);
        let mut state = TramTransitState::default();
        state.add_stop(&grid, 5, 5);
        let id2 = state.add_stop(&grid, 5, 5);
        assert!(id2.is_none());
        assert_eq!(state.stops.len(), 1);
    }

    #[test]
    fn test_add_line() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(10, 10).cell_type = CellType::Road;
        let mut state = TramTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let s2 = state.add_stop(&grid, 10, 10).unwrap();
        let line_id = state.add_line("Tram Line 1".to_string(), vec![s1, s2]);
        assert!(line_id.is_some());
        assert_eq!(state.lines.len(), 1);
        assert!(!state.lines[0].active); // No depot yet
    }

    #[test]
    fn test_add_line_too_few_stops() {
        let grid = make_grid_with_road(5, 5);
        let mut state = TramTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let line_id = state.add_line("Tram Line 1".to_string(), vec![s1]);
        assert!(line_id.is_none());
    }

    #[test]
    fn test_remove_stop_removes_from_lines() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(15, 15).cell_type = CellType::Road;
        let mut state = TramTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let s2 = state.add_stop(&grid, 10, 10).unwrap();
        let s3 = state.add_stop(&grid, 15, 15).unwrap();
        state.add_line("Tram Line 1".to_string(), vec![s1, s2, s3]);
        assert_eq!(state.lines[0].stop_ids.len(), 3);

        state.remove_stop(s2);
        assert_eq!(state.stops.len(), 2);
        assert_eq!(state.lines[0].stop_ids.len(), 2);
    }

    #[test]
    fn test_remove_stop_removes_line_with_too_few_stops() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(10, 10).cell_type = CellType::Road;
        let mut state = TramTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let s2 = state.add_stop(&grid, 10, 10).unwrap();
        state.add_line("Tram Line 1".to_string(), vec![s1, s2]);
        assert_eq!(state.lines.len(), 1);

        state.remove_stop(s1);
        assert_eq!(state.lines.len(), 0);
    }

    #[test]
    fn test_remove_line() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(10, 10).cell_type = CellType::Road;
        let mut state = TramTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let s2 = state.add_stop(&grid, 10, 10).unwrap();
        let line_id = state
            .add_line("Tram Line 1".to_string(), vec![s1, s2])
            .unwrap();

        // Add a tram manually
        state.trams.push(TramVehicle {
            line_id,
            next_stop_index: 0,
            grid_x: 5.0,
            grid_y: 5.0,
            passengers: 0,
            dwell_ticks: 0,
            at_stop: false,
        });

        state.remove_line(line_id);
        assert_eq!(state.lines.len(), 0);
        assert_eq!(state.trams.len(), 0);
    }

    #[test]
    fn test_nearest_stop() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(20, 20).cell_type = CellType::Road;
        let mut state = TramTransitState::default();
        state.add_stop(&grid, 5, 5);
        state.add_stop(&grid, 20, 20);

        // (7,7) is closer to (5,5)
        let nearest = state.nearest_stop(7, 7);
        assert!(nearest.is_some());
        assert_eq!(nearest.unwrap().grid_x, 5);

        // (31, 31) is too far from both stops
        let far = state.nearest_stop(31, 31);
        assert!(far.is_none());
    }

    #[test]
    fn test_tram_capacity_constant() {
        assert_eq!(TRAM_CAPACITY, 90);
    }

    #[test]
    fn test_manhattan_distance() {
        assert_eq!(manhattan_distance(0, 0, 3, 4), 7);
        assert_eq!(manhattan_distance(5, 5, 5, 5), 0);
        assert_eq!(manhattan_distance(10, 0, 0, 10), 20);
    }

    #[test]
    fn test_estimate_transit_time_no_active_lines() {
        let state = TramTransitState::default();
        assert!(state.estimate_transit_time(0, 0, 10, 10).is_none());
    }

    #[test]
    fn test_estimate_transit_time_with_line() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        grid.get_mut(15, 15).cell_type = CellType::Road;
        let mut state = TramTransitState::default();
        let s1 = state.add_stop(&grid, 5, 5).unwrap();
        let s2 = state.add_stop(&grid, 15, 15).unwrap();
        state.add_line("Test".to_string(), vec![s1, s2]);
        state.lines[0].active = true; // Force active for test

        let time = state.estimate_transit_time(4, 4, 16, 16);
        assert!(time.is_some());
        let t = time.unwrap();
        assert!(t > 15, "Transit time should include wait: {}", t);
    }

    #[test]
    fn test_saveable_empty_state() {
        let state = TramTransitState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut grid = WorldGrid::new(32, 32);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        let mut state = TramTransitState::default();
        state.add_stop(&grid, 5, 5);

        let bytes = state.save_to_bytes().unwrap();
        let loaded = TramTransitState::load_from_bytes(&bytes);
        assert_eq!(loaded.stops.len(), 1);
        assert_eq!(loaded.stops[0].grid_x, 5);
    }

    #[test]
    fn test_stats_saveable_empty() {
        let stats = TramTransitStats::default();
        assert!(stats.save_to_bytes().is_none());
    }

    #[test]
    fn test_stats_saveable_roundtrip() {
        let stats = TramTransitStats {
            active_lines: 2,
            total_stops: 10,
            daily_ridership: 500,
            monthly_operating_cost: 4800.0,
            monthly_fare_revenue: 2500.0,
            cumulative_ridership: 50_000,
        };
        let bytes = stats.save_to_bytes().unwrap();
        let loaded = TramTransitStats::load_from_bytes(&bytes);
        assert_eq!(loaded.active_lines, 2);
        assert_eq!(loaded.total_stops, 10);
        assert_eq!(loaded.cumulative_ridership, 50_000);
    }
}

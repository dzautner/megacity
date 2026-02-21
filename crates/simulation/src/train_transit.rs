//! TRAF-012: Train/Rail Transit System
//!
//! Implements a surface-level train/rail transit system for high-capacity
//! intercity and commuter rail service. Trains run on dedicated rail lines
//! separate from roads, connecting train stations placed on the grid.
//!
//! ## Data model
//! - `TrainStation`: a station placed on a grid cell (capacity, passenger queue)
//! - `TrainLine`: a named route connecting stations in sequence
//! - `TrainInstance`: a train entity traveling along a line
//! - `TrainTransitState`: top-level resource storing all stations, lines, trains, and stats
//!
//! ## Costs
//! - $2000/week per active line + $800/week per station
//! - Fare revenue: $3 per ride
//!
//! ## Key differences from metro
//! - Surface-level rail (marks grid cells as rail track)
//! - Higher capacity (200 passengers) but lower frequency
//! - Different cost structure (weekly operating costs)
//! - Larger land value boost radius (commuter rail effect)
//!
//! The `TrainTransitState` resource is the source of truth and is persisted
//! via the `Saveable` extension map.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::{CellType, WorldGrid};
use crate::land_value::LandValueGrid;
use crate::stats::CityStats;
use crate::Saveable;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Maximum passengers per train.
pub const TRAIN_CAPACITY: u32 = 200;

/// Train speed in km/h.
pub const TRAIN_SPEED_KMH: f32 = 120.0;

/// Train speed in grid-cells per game-hour.
/// 120 km/h = 120_000 m/h / 16 m/cell = 7500 cells/h.
pub const TRAIN_SPEED_CELLS_PER_HOUR: f32 = 7500.0;

/// Maximum walking distance to a train station (in grid cells).
/// ~1200m / 16m = 75 cells.
pub const MAX_WALK_TO_STATION_CELLS: u32 = 75;

/// Walking speed for station access in grid-cells per game-hour.
pub const WALK_SPEED_CELLS_PER_HOUR: f32 = 250.0;

/// Headway between trains in minutes (game time).
pub const TRAIN_HEADWAY_MINUTES: f32 = 10.0;

/// Average wait time is half the headway.
pub const AVERAGE_WAIT_MINUTES: f32 = TRAIN_HEADWAY_MINUTES / 2.0;

/// Weekly cost per active train line.
pub const LINE_WEEKLY_COST: f64 = 2000.0;

/// Weekly cost per station.
pub const STATION_WEEKLY_COST: f64 = 800.0;

/// Fare per ride (revenue).
pub const FARE_PER_RIDE: f64 = 3.0;

/// Land value boost at station center.
pub const STATION_LAND_VALUE_BOOST_CENTER: i32 = 20;

/// Land value boost radius (in grid cells).
pub const STATION_LAND_VALUE_BOOST_RADIUS: i32 = 15;

/// Minimum land value boost at edge of radius.
pub const STATION_LAND_VALUE_BOOST_MIN: i32 = 10;

// =============================================================================
// Data Structures
// =============================================================================

/// Unique identifier for a train station.
pub type StationId = u32;

/// Unique identifier for a train line.
pub type LineId = u32;

/// A single train station placed on a grid cell.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TrainStation {
    /// Unique station identifier.
    pub id: StationId,
    /// Grid X coordinate.
    pub grid_x: usize,
    /// Grid Y coordinate.
    pub grid_y: usize,
    /// Human-readable station name.
    pub name: String,
    /// Maximum passenger capacity of the station platform.
    pub capacity: u32,
    /// Number of passengers currently waiting at this station.
    pub passenger_queue: u32,
    /// Cumulative ridership (total boardings).
    pub total_ridership: u64,
    /// Ridership in the current period (reset each slow tick).
    pub period_ridership: u32,
}

/// A train line connecting stations in sequence.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TrainLine {
    /// Unique line identifier.
    pub id: LineId,
    /// Human-readable line name (e.g., "Northern Line").
    pub name: String,
    /// Ordered list of station IDs on this line.
    pub station_ids: Vec<StationId>,
    /// Whether this line is currently active.
    pub active: bool,
}

/// A train entity traveling along a line.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TrainInstance {
    /// ID of the line this train runs on.
    pub line_id: LineId,
    /// Index of the next station in the line's station list.
    pub next_station_index: usize,
    /// Current grid position (fractional for smooth movement).
    pub grid_x: f32,
    /// Current grid position (fractional for smooth movement).
    pub grid_y: f32,
    /// Number of passengers on board.
    pub passengers: u32,
    /// Ticks remaining at current station (dwell time).
    pub dwell_ticks: u32,
    /// Whether the train is currently at a station.
    pub at_station: bool,
}

/// Aggregate statistics for the train transit system.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct TrainTransitStats {
    /// Total stations in the network.
    pub total_stations: u32,
    /// Total active lines.
    pub total_active_lines: u32,
    /// Estimated daily ridership.
    pub daily_ridership: u32,
    /// Cargo units moved (simplified).
    pub cargo_moved: u64,
    /// Monthly maintenance cost.
    pub monthly_maintenance_cost: f64,
    /// Monthly fare revenue.
    pub monthly_fare_revenue: f64,
    /// Cumulative ridership since city founding.
    pub cumulative_ridership: u64,
}

// =============================================================================
// Resource (Saveable)
// =============================================================================

/// The source of truth for the train transit system.
///
/// Contains all stations, lines, trains, and aggregate statistics. Persisted via
/// the `Saveable` extension map so no save-system files need modification.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Encode, Decode)]
pub struct TrainTransitState {
    /// All train stations.
    pub stations: Vec<TrainStation>,
    /// All train lines.
    pub lines: Vec<TrainLine>,
    /// All active train instances.
    pub trains: Vec<TrainInstance>,
    /// Aggregate statistics.
    pub stats: TrainTransitStats,
    /// Auto-incrementing station ID counter.
    next_station_id: u32,
    /// Auto-incrementing line ID counter.
    next_line_id: u32,
    /// Last day costs were deducted.
    last_cost_day: u32,
}

impl TrainTransitState {
    // -------------------------------------------------------------------------
    // Station management
    // -------------------------------------------------------------------------

    /// Add a new train station at the given grid position.
    /// Returns the new station ID, or `None` if the position is invalid.
    pub fn add_station(
        &mut self,
        grid_x: usize,
        grid_y: usize,
        name: String,
        grid: &WorldGrid,
    ) -> Option<StationId> {
        // Validate bounds
        if !grid.in_bounds(grid_x, grid_y) {
            return None;
        }
        // Cannot place on water
        if grid.get(grid_x, grid_y).cell_type == CellType::Water {
            return None;
        }
        // Cannot place on a cell that already has a station
        if self.station_at(grid_x, grid_y).is_some() {
            return None;
        }

        let id = self.next_station_id;
        self.next_station_id += 1;

        self.stations.push(TrainStation {
            id,
            grid_x,
            grid_y,
            name,
            capacity: TRAIN_CAPACITY,
            passenger_queue: 0,
            total_ridership: 0,
            period_ridership: 0,
        });

        Some(id)
    }

    /// Remove a station by ID. Also removes it from any lines.
    pub fn remove_station(&mut self, station_id: StationId) -> bool {
        let before = self.stations.len();
        self.stations.retain(|s| s.id != station_id);
        if self.stations.len() == before {
            return false;
        }
        // Remove from all lines and deactivate lines with < 2 stations
        for line in &mut self.lines {
            line.station_ids.retain(|&id| id != station_id);
            if line.station_ids.len() < 2 {
                line.active = false;
            }
        }
        // Remove trains on deactivated lines
        let inactive_lines: Vec<LineId> = self
            .lines
            .iter()
            .filter(|l| !l.active)
            .map(|l| l.id)
            .collect();
        self.trains.retain(|t| !inactive_lines.contains(&t.line_id));
        true
    }

    /// Find a station at the given grid position.
    pub fn station_at(&self, grid_x: usize, grid_y: usize) -> Option<&TrainStation> {
        self.stations
            .iter()
            .find(|s| s.grid_x == grid_x && s.grid_y == grid_y)
    }

    /// Find a station by ID.
    pub fn station_by_id(&self, id: StationId) -> Option<&TrainStation> {
        self.stations.iter().find(|s| s.id == id)
    }

    // -------------------------------------------------------------------------
    // Line management
    // -------------------------------------------------------------------------

    /// Create a new train line with the given name and station sequence.
    /// Returns the line ID, or `None` if fewer than 2 valid stations.
    pub fn add_line(&mut self, name: String, station_ids: Vec<StationId>) -> Option<LineId> {
        // Filter to only existing station IDs
        let valid: Vec<StationId> = station_ids
            .into_iter()
            .filter(|&id| self.station_by_id(id).is_some())
            .collect();

        if valid.len() < 2 {
            return None;
        }

        let id = self.next_line_id;
        self.next_line_id += 1;

        self.lines.push(TrainLine {
            id,
            name,
            station_ids: valid,
            active: true,
        });

        // Spawn initial trains for the new line
        self.spawn_trains_for_line(id);

        Some(id)
    }

    /// Remove a train line by ID.
    pub fn remove_line(&mut self, line_id: LineId) -> bool {
        let before = self.lines.len();
        self.lines.retain(|l| l.id != line_id);
        self.trains.retain(|t| t.line_id != line_id);
        self.lines.len() < before
    }

    /// Spawn trains for a specific line.
    fn spawn_trains_for_line(&mut self, line_id: LineId) {
        let Some(line) = self.lines.iter().find(|l| l.id == line_id) else {
            return;
        };
        if !line.active || line.station_ids.is_empty() {
            return;
        }

        // Spawn one train at the first station
        let first_station_id = line.station_ids[0];
        if let Some(station) = self.station_by_id(first_station_id) {
            let next_idx = if line.station_ids.len() > 1 { 1 } else { 0 };
            self.trains.push(TrainInstance {
                line_id,
                next_station_index: next_idx,
                grid_x: station.grid_x as f32,
                grid_y: station.grid_y as f32,
                passengers: 0,
                dwell_ticks: 0,
                at_station: false,
            });
        }
    }

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    /// Find the nearest station to a grid position.
    /// Returns (station_id, manhattan_distance) or None if no stations.
    pub fn nearest_station(&self, grid_x: usize, grid_y: usize) -> Option<(StationId, u32)> {
        self.stations
            .iter()
            .map(|s| {
                let dx = (s.grid_x as i32 - grid_x as i32).unsigned_abs();
                let dy = (s.grid_y as i32 - grid_y as i32).unsigned_abs();
                (s.id, dx + dy)
            })
            .min_by_key(|&(_, dist)| dist)
    }

    /// Check if two stations are connected (on the same active line or via transfer).
    pub fn stations_connected(&self, a: StationId, b: StationId) -> bool {
        if a == b {
            return true;
        }
        // Direct: both on same active line
        for line in &self.lines {
            if !line.active {
                continue;
            }
            if line.station_ids.contains(&a) && line.station_ids.contains(&b) {
                return true;
            }
        }
        // Transfer: lines share a common station
        let lines_with_a: Vec<&TrainLine> = self
            .lines
            .iter()
            .filter(|l| l.active && l.station_ids.contains(&a))
            .collect();
        let lines_with_b: Vec<&TrainLine> = self
            .lines
            .iter()
            .filter(|l| l.active && l.station_ids.contains(&b))
            .collect();

        for la in &lines_with_a {
            for lb in &lines_with_b {
                if la.station_ids.iter().any(|s| lb.station_ids.contains(s)) {
                    return true;
                }
            }
        }

        false
    }

    /// Estimate the travel time (in game-hours) for a citizen to use the train
    /// between two grid positions. Returns `None` if no viable route.
    pub fn estimate_travel_time(
        &self,
        from_x: usize,
        from_y: usize,
        to_x: usize,
        to_y: usize,
    ) -> Option<f32> {
        let (origin_station, origin_dist) = self.nearest_station(from_x, from_y)?;
        if origin_dist > MAX_WALK_TO_STATION_CELLS {
            return None;
        }

        let (dest_station, dest_dist) = self.nearest_station(to_x, to_y)?;
        if dest_dist > MAX_WALK_TO_STATION_CELLS {
            return None;
        }

        if !self.stations_connected(origin_station, dest_station) {
            return None;
        }

        let walk_to = origin_dist as f32 / WALK_SPEED_CELLS_PER_HOUR;
        let wait = AVERAGE_WAIT_MINUTES / 60.0;
        let ride_dist = self.route_distance(origin_station, dest_station);
        let ride_time = ride_dist as f32 / TRAIN_SPEED_CELLS_PER_HOUR;
        let walk_from = dest_dist as f32 / WALK_SPEED_CELLS_PER_HOUR;

        Some(walk_to + wait + ride_time + walk_from)
    }

    /// Calculate the route distance between two stations (in grid cells).
    fn route_distance(&self, a: StationId, b: StationId) -> u32 {
        let sa = self.station_by_id(a);
        let sb = self.station_by_id(b);
        match (sa, sb) {
            (Some(sa), Some(sb)) => {
                let dx = (sa.grid_x as i32 - sb.grid_x as i32).unsigned_abs();
                let dy = (sa.grid_y as i32 - sb.grid_y as i32).unsigned_abs();
                dx + dy
            }
            _ => 0,
        }
    }

    /// Calculate total weekly operating cost for all stations and lines.
    pub fn total_weekly_cost(&self) -> f64 {
        let station_cost = self.stations.len() as f64 * STATION_WEEKLY_COST;
        let line_cost = self.lines.iter().filter(|l| l.active).count() as f64 * LINE_WEEKLY_COST;
        station_cost + line_cost
    }

    /// Calculate total monthly operating cost (4 weeks).
    pub fn total_monthly_cost(&self) -> f64 {
        self.total_weekly_cost() * 4.0
    }

    /// Get total number of active lines.
    pub fn active_line_count(&self) -> usize {
        self.lines.iter().filter(|l| l.active).count()
    }

    /// Estimate daily ridership based on nearby population and network connectivity.
    fn estimate_daily_ridership(&self, population: u32) -> u32 {
        if self.stations.is_empty() || self.lines.iter().all(|l| !l.active) {
            return 0;
        }

        let operational_stations = self.stations.len() as f32;
        let operational_lines = self.lines.iter().filter(|l| l.active).count() as f32;

        // Base ridership: fraction of population that uses trains
        // Trains have wider coverage but lower frequency than metro
        let network_factor = (operational_stations * 0.008 + operational_lines * 0.015).min(0.25);
        let base_ridership = population as f32 * network_factor;

        // Capacity constraint
        let trips_per_day = (24.0 * 60.0 / TRAIN_HEADWAY_MINUTES) as u32;
        let total_capacity = operational_lines as u32 * TRAIN_CAPACITY * trips_per_day;

        (base_ridership as u32).min(total_capacity)
    }
}

// =============================================================================
// Saveable implementations
// =============================================================================

impl Saveable for TrainTransitState {
    const SAVE_KEY: &'static str = "train_transit";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.stations.is_empty() && self.lines.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Update train transit statistics and ridership every slow tick.
pub fn update_train_lines(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<TrainTransitState>,
    city_stats: Res<CityStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let total_stations = state.stations.len() as u32;
    let total_active_lines = state.lines.iter().filter(|l| l.active).count() as u32;
    let daily_ridership = state.estimate_daily_ridership(city_stats.population);
    let monthly_cost = state.total_monthly_cost();

    // Distribute ridership across stations proportionally
    if total_stations > 0 && daily_ridership > 0 {
        let per_station = daily_ridership / total_stations;
        let remainder = daily_ridership % total_stations;
        for (i, station) in state.stations.iter_mut().enumerate() {
            let riders = per_station + if (i as u32) < remainder { 1 } else { 0 };
            station.period_ridership = riders;
            station.total_ridership += riders as u64;
            // Add some passengers to the queue (capped by capacity)
            station.passenger_queue = (station.passenger_queue + riders / 4).min(station.capacity);
        }
    } else {
        for station in &mut state.stations {
            station.period_ridership = 0;
        }
    }

    // Simulate trains moving between stations
    simulate_train_movement(&mut state);

    // Estimate cargo moved (simplified: proportional to ridership)
    let cargo_increment = (daily_ridership as u64) / 10;

    // Calculate fare revenue from ridership
    let fare_revenue = daily_ridership as f64 * FARE_PER_RIDE;

    state.stats = TrainTransitStats {
        total_stations,
        total_active_lines,
        daily_ridership,
        cargo_moved: state.stats.cargo_moved + cargo_increment,
        monthly_maintenance_cost: monthly_cost,
        monthly_fare_revenue: state.stats.monthly_fare_revenue + fare_revenue,
        cumulative_ridership: state.stations.iter().map(|s| s.total_ridership).sum(),
    };
}

/// Simulate train movement along lines, picking up and dropping off passengers.
fn simulate_train_movement(state: &mut TrainTransitState) {
    // Pre-collect station data to avoid borrow conflicts
    struct StationData {
        id: StationId,
        grid_x: f32,
        grid_y: f32,
        passenger_queue: u32,
    }

    let station_data: Vec<StationData> = state
        .stations
        .iter()
        .map(|s| StationData {
            id: s.id,
            grid_x: s.grid_x as f32,
            grid_y: s.grid_y as f32,
            passenger_queue: s.passenger_queue,
        })
        .collect();

    // Pre-collect line data
    struct LineData {
        id: LineId,
        station_ids: Vec<StationId>,
        active: bool,
    }

    let line_data: Vec<LineData> = state
        .lines
        .iter()
        .map(|l| LineData {
            id: l.id,
            station_ids: l.station_ids.clone(),
            active: l.active,
        })
        .collect();

    let mut queue_decrements: Vec<(StationId, u32)> = Vec::new();
    let mut ridership_increments: Vec<(StationId, u32)> = Vec::new();

    for train in &mut state.trains {
        let Some(ld) = line_data.iter().find(|l| l.id == train.line_id) else {
            continue;
        };
        if !ld.active || ld.station_ids.is_empty() {
            continue;
        }

        // Handle dwelling at station
        if train.at_station {
            if train.dwell_ticks > 0 {
                train.dwell_ticks -= 1;
                continue;
            }
            train.at_station = false;
            train.next_station_index = (train.next_station_index + 1) % ld.station_ids.len();
        }

        // Move toward next station
        let next_station_id = ld.station_ids[train.next_station_index % ld.station_ids.len()];
        let Some(next_sd) = station_data.iter().find(|s| s.id == next_station_id) else {
            continue;
        };

        let dx = next_sd.grid_x - train.grid_x;
        let dy = next_sd.grid_y - train.grid_y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Train moves ~1 cell per tick at this simplified rate
        let speed = 1.0_f32;

        if dist < speed {
            // Arrived at station
            train.grid_x = next_sd.grid_x;
            train.grid_y = next_sd.grid_y;
            train.at_station = true;
            train.dwell_ticks = 3;

            // Drop off passengers
            let dropoff = (train.passengers / 3).max(1).min(train.passengers);
            train.passengers = train.passengers.saturating_sub(dropoff);

            // Pick up waiting passengers
            let waiting = next_sd.passenger_queue;
            let space = TRAIN_CAPACITY.saturating_sub(train.passengers);
            let pickup = waiting.min(space);
            if pickup > 0 {
                train.passengers += pickup;
                queue_decrements.push((next_station_id, pickup));
                ridership_increments.push((next_station_id, pickup));
            }
        } else {
            let norm = 1.0 / dist;
            train.grid_x += dx * norm * speed;
            train.grid_y += dy * norm * speed;
        }
    }

    // Apply queue decrements
    for (station_id, decrement) in queue_decrements {
        if let Some(station) = state.stations.iter_mut().find(|s| s.id == station_id) {
            station.passenger_queue = station.passenger_queue.saturating_sub(decrement);
        }
    }

    // Apply ridership increments from train pickups
    for (station_id, count) in ridership_increments {
        if let Some(station) = state.stations.iter_mut().find(|s| s.id == station_id) {
            station.total_ridership += count as u64;
        }
    }
}

/// Boost land value around train stations.
///
/// Each station provides a +10–20 land value bonus in a radius around it,
/// with the boost diminishing linearly with distance.
pub fn train_station_land_value(
    slow_timer: Res<SlowTickTimer>,
    state: Res<TrainTransitState>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for station in &state.stations {
        let cx = station.grid_x as i32;
        let cy = station.grid_y as i32;
        let radius = STATION_LAND_VALUE_BOOST_RADIUS;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }

                let dist = dx.abs() + dy.abs();
                if dist > radius {
                    continue;
                }

                // Linear interpolation: center gets BOOST_CENTER, edge gets BOOST_MIN
                let t = dist as f32 / radius as f32;
                let boost = STATION_LAND_VALUE_BOOST_CENTER as f32 * (1.0 - t)
                    + STATION_LAND_VALUE_BOOST_MIN as f32 * t;
                let boost = boost as i32;
                if boost <= 0 {
                    continue;
                }

                let ux = nx as usize;
                let uy = ny as usize;
                let cur = land_value.get(ux, uy) as i32;
                land_value.set(ux, uy, (cur + boost).min(255) as u8);
            }
        }
    }
}

/// Deduct train operating costs from the city budget.
///
/// Runs on slow tick, deducting costs every 7 days (weekly).
pub fn update_train_costs(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<TrainTransitState>,
    mut budget: ResMut<CityBudget>,
    clock: Res<crate::time_of_day::GameClock>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Only deduct every ~7 days (weekly costs)
    if clock.day <= state.last_cost_day + 7 {
        return;
    }
    state.last_cost_day = clock.day;

    let weekly_cost = state.total_weekly_cost();
    if weekly_cost > 0.0 {
        budget.treasury -= weekly_cost;
    }

    // Add fare revenue to budget
    let fare_revenue = state.stats.monthly_fare_revenue;
    if fare_revenue > 0.0 {
        budget.treasury += fare_revenue;
        state.stats.monthly_fare_revenue = 0.0;
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct TrainTransitPlugin;

impl Plugin for TrainTransitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrainTransitState>().add_systems(
            FixedUpdate,
            (
                update_train_lines,
                train_station_land_value.after(crate::land_value::update_land_value),
                update_train_costs,
            ),
        );

        // Register for save/load via the extension map
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<TrainTransitState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> WorldGrid {
        WorldGrid::new(GRID_WIDTH, GRID_HEIGHT)
    }

    #[test]
    fn test_add_station() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let id = state.add_station(10, 10, "Central".to_string(), &grid);
        assert!(id.is_some());
        assert_eq!(state.stations.len(), 1);
        assert_eq!(state.stations[0].name, "Central");
    }

    #[test]
    fn test_add_station_on_water_fails() {
        let mut grid = make_grid();
        grid.get_mut(10, 10).cell_type = CellType::Water;

        let mut state = TrainTransitState::default();
        let id = state.add_station(10, 10, "Aquatic".to_string(), &grid);
        assert!(id.is_none());
    }

    #[test]
    fn test_add_station_duplicate_position_fails() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        state.add_station(10, 10, "First".to_string(), &grid);
        let id = state.add_station(10, 10, "Second".to_string(), &grid);
        assert!(id.is_none());
    }

    #[test]
    fn test_remove_station() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let id = state
            .add_station(10, 10, "Central".to_string(), &grid)
            .unwrap();
        assert!(state.remove_station(id));
        assert!(state.stations.is_empty());
    }

    #[test]
    fn test_remove_station_from_lines() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        let s3 = state.add_station(30, 30, "C".to_string(), &grid).unwrap();

        let line_id = state
            .add_line("Northern".to_string(), vec![s1, s2, s3])
            .unwrap();

        // Remove middle station
        state.remove_station(s2);

        let line = state.lines.iter().find(|l| l.id == line_id).unwrap();
        assert_eq!(line.station_ids, vec![s1, s3]);
        assert!(line.active); // still has 2 stations
    }

    #[test]
    fn test_add_line() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();

        let line_id = state.add_line("Blue".to_string(), vec![s1, s2]);
        assert!(line_id.is_some());
        assert_eq!(state.lines.len(), 1);
        assert!(state.lines[0].active);
        // Should also have spawned a train
        assert_eq!(state.trains.len(), 1);
    }

    #[test]
    fn test_add_line_too_few_stations_fails() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();

        let line_id = state.add_line("Lonely".to_string(), vec![s1]);
        assert!(line_id.is_none());
    }

    #[test]
    fn test_stations_connected_same_line() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        assert!(state.stations_connected(s1, s2));
    }

    #[test]
    fn test_stations_connected_via_transfer() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        let s3 = state.add_station(30, 30, "C".to_string(), &grid).unwrap();

        state.add_line("Red".to_string(), vec![s1, s2]);
        state.add_line("Blue".to_string(), vec![s2, s3]);

        assert!(state.stations_connected(s1, s3));
    }

    #[test]
    fn test_stations_not_connected() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        let s3 = state.add_station(30, 30, "C".to_string(), &grid).unwrap();
        let s4 = state.add_station(40, 40, "D".to_string(), &grid).unwrap();

        state.add_line("Red".to_string(), vec![s1, s2]);
        state.add_line("Blue".to_string(), vec![s3, s4]);

        assert!(!state.stations_connected(s1, s3));
    }

    #[test]
    fn test_total_weekly_cost() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        let cost = state.total_weekly_cost();
        // 2 stations * $800/week + 1 line * $2000/week = $3600
        let expected = 2.0 * STATION_WEEKLY_COST + 1.0 * LINE_WEEKLY_COST;
        assert!((cost - expected).abs() < 0.01);
    }

    #[test]
    fn test_total_monthly_cost() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        let monthly = state.total_monthly_cost();
        let weekly = state.total_weekly_cost();
        assert!((monthly - weekly * 4.0).abs() < 0.01);
    }

    #[test]
    fn test_estimate_travel_time() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(50, 50, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(100, 50, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        let time = state.estimate_travel_time(48, 50, 102, 50);
        assert!(time.is_some());
        let t = time.unwrap();
        assert!(t > 0.0);
        assert!(t < 1.0); // Should be well under 1 hour
    }

    #[test]
    fn test_estimate_travel_time_too_far() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(50, 50, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(100, 50, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        let time = state.estimate_travel_time(200, 200, 102, 50);
        assert!(time.is_none());
    }

    #[test]
    fn test_remove_line() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        let line_id = state.add_line("Red".to_string(), vec![s1, s2]).unwrap();

        assert!(state.remove_line(line_id));
        assert!(state.lines.is_empty());
        assert!(state.trains.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_station_returns_false() {
        let mut state = TrainTransitState::default();
        assert!(!state.remove_station(999));
    }

    #[test]
    fn test_remove_nonexistent_line_returns_false() {
        let mut state = TrainTransitState::default();
        assert!(!state.remove_line(999));
    }

    #[test]
    fn test_saveable_roundtrip() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        state.add_station(10, 10, "Central".to_string(), &grid);
        state.add_station(20, 20, "North".to_string(), &grid);
        state.add_line("Red".to_string(), vec![0, 1]);

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = TrainTransitState::load_from_bytes(&bytes);

        assert_eq!(restored.stations.len(), 2);
        assert_eq!(restored.lines.len(), 1);
        assert_eq!(restored.stations[0].name, "Central");
    }

    #[test]
    fn test_saveable_empty_returns_none() {
        let state = TrainTransitState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_daily_ridership_no_lines_is_zero() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        state.add_station(10, 10, "A".to_string(), &grid);
        state.add_station(20, 20, "B".to_string(), &grid);

        // No lines created, so ridership should be 0
        assert_eq!(state.estimate_daily_ridership(100_000), 0);
    }

    #[test]
    fn test_daily_ridership_with_line() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);

        let ridership = state.estimate_daily_ridership(100_000);
        assert!(ridership > 0);
    }

    #[test]
    fn test_nearest_station() {
        let grid = make_grid();
        let mut state = TrainTransitState::default();

        state
            .add_station(10, 10, "Near".to_string(), &grid)
            .unwrap();
        state
            .add_station(100, 100, "Far".to_string(), &grid)
            .unwrap();

        let (id, dist) = state.nearest_station(12, 12).unwrap();
        assert_eq!(id, 0);
        assert_eq!(dist, 4);
    }
}

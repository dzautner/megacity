//! TRAF-006: Metro/Subway Transit System
//!
//! Implements an underground metro system for high-capacity transit.
//! Metro stations are placed on grid cells (independent of roads) and
//! connected into named lines. Trains run between stations at 80 km/h
//! with 180-passenger capacity.
//!
//! Key mechanics:
//! - Metro stations placed on grid cells (underground, no road required)
//! - Metro lines connect stations in sequence
//! - Citizens walk to nearest station, ride, then walk to destination station
//! - Metro is immune to surface traffic (separate graph)
//! - Stations boost nearby land value (+15–25 in radius)
//! - Construction and maintenance costs tracked in city budget
//! - Ridership statistics updated every slow tick
//!
//! The `MetroTransitState` resource is the source of truth and is persisted
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

/// Metro train speed in km/h.
pub const METRO_SPEED_KMH: f32 = 80.0;

/// Maximum passengers per train.
pub const TRAIN_CAPACITY: u32 = 180;

/// Walking speed for station access in grid-cells per tick.
/// Citizens walk ~4 km/h ≈ 4000m/h ÷ 16m/cell ≈ 250 cells/h.
pub const WALK_SPEED_CELLS_PER_HOUR: f32 = 250.0;

/// Maximum walking distance to a metro station (in grid cells).
/// ~800m / 16m = 50 cells.
pub const MAX_WALK_TO_STATION_CELLS: u32 = 50;

/// Station construction cost (one-time).
pub const STATION_CONSTRUCTION_COST: f64 = 5000.0;

/// Station weekly maintenance cost.
pub const STATION_WEEKLY_MAINTENANCE: f64 = 500.0;

/// Line weekly maintenance cost.
pub const LINE_WEEKLY_MAINTENANCE: f64 = 1200.0;

/// Land value boost at station center.
pub const STATION_LAND_VALUE_BOOST_CENTER: i32 = 25;

/// Land value boost radius (in grid cells).
pub const STATION_LAND_VALUE_BOOST_RADIUS: i32 = 12;

/// Minimum land value boost at edge of radius.
pub const STATION_LAND_VALUE_BOOST_MIN: i32 = 15;

/// Headway between trains in minutes (game time).
pub const TRAIN_HEADWAY_MINUTES: f32 = 5.0;

/// Average wait time is half the headway.
pub const AVERAGE_WAIT_MINUTES: f32 = TRAIN_HEADWAY_MINUTES / 2.0;

/// Metro speed in grid-cells per game-hour.
/// 80 km/h = 80_000 m/h ÷ 16 m/cell = 5000 cells/h.
pub const METRO_SPEED_CELLS_PER_HOUR: f32 = 5000.0;

// =============================================================================
// Data Structures
// =============================================================================

/// Unique identifier for a metro station.
pub type StationId = u32;

/// Unique identifier for a metro line.
pub type LineId = u32;

/// A single metro station placed underground at a grid cell.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct MetroStation {
    /// Unique station identifier.
    pub id: StationId,
    /// Grid X coordinate.
    pub grid_x: usize,
    /// Grid Y coordinate.
    pub grid_y: usize,
    /// Human-readable station name.
    pub name: String,
    /// Cumulative ridership (total boardings).
    pub total_ridership: u64,
    /// Ridership in the current period (reset each slow tick).
    pub period_ridership: u32,
}

/// A metro line connecting stations in sequence.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct MetroLine {
    /// Unique line identifier.
    pub id: LineId,
    /// Human-readable line name (e.g., "Red Line").
    pub name: String,
    /// Ordered list of station IDs on this line.
    pub station_ids: Vec<StationId>,
    /// Whether this line is currently operational (has >= 2 stations).
    pub operational: bool,
}

/// City-wide metro ridership and cost statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct MetroStats {
    /// Total stations in the network.
    pub total_stations: u32,
    /// Total operational lines.
    pub total_lines: u32,
    /// Estimated daily ridership.
    pub daily_ridership: u32,
    /// Monthly maintenance cost (all stations + lines).
    pub monthly_maintenance_cost: f64,
    /// Total cumulative ridership since city founding.
    pub cumulative_ridership: u64,
}

// =============================================================================
// Resource (Saveable)
// =============================================================================

/// The source of truth for the metro transit system.
///
/// Contains all stations, lines, and aggregate statistics. Persisted via
/// the `Saveable` extension map so no save-system files need modification.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Encode, Decode)]
pub struct MetroTransitState {
    /// All metro stations, keyed by ID.
    pub stations: Vec<MetroStation>,
    /// All metro lines.
    pub lines: Vec<MetroLine>,
    /// Aggregate stats.
    pub stats: MetroStats,
    /// Auto-incrementing station ID counter.
    next_station_id: u32,
    /// Auto-incrementing line ID counter.
    next_line_id: u32,
}

impl MetroTransitState {
    // -------------------------------------------------------------------------
    // Station management
    // -------------------------------------------------------------------------

    /// Add a new metro station at the given grid position.
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

        self.stations.push(MetroStation {
            id,
            grid_x,
            grid_y,
            name,
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
        // Remove from all lines
        for line in &mut self.lines {
            line.station_ids.retain(|&id| id != station_id);
            line.operational = line.station_ids.len() >= 2;
        }
        true
    }

    /// Find a station at the given grid position.
    pub fn station_at(&self, grid_x: usize, grid_y: usize) -> Option<&MetroStation> {
        self.stations
            .iter()
            .find(|s| s.grid_x == grid_x && s.grid_y == grid_y)
    }

    /// Find a station by ID.
    pub fn station_by_id(&self, id: StationId) -> Option<&MetroStation> {
        self.stations.iter().find(|s| s.id == id)
    }

    // -------------------------------------------------------------------------
    // Line management
    // -------------------------------------------------------------------------

    /// Create a new metro line with the given name and station sequence.
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

        self.lines.push(MetroLine {
            id,
            name,
            station_ids: valid,
            operational: true,
        });

        Some(id)
    }

    /// Remove a metro line by ID.
    pub fn remove_line(&mut self, line_id: LineId) -> bool {
        let before = self.lines.len();
        self.lines.retain(|l| l.id != line_id);
        self.lines.len() < before
    }

    /// Add a station to an existing line (appends to the end).
    pub fn extend_line(&mut self, line_id: LineId, station_id: StationId) -> bool {
        if self.station_by_id(station_id).is_none() {
            return false;
        }
        if let Some(line) = self.lines.iter_mut().find(|l| l.id == line_id) {
            line.station_ids.push(station_id);
            line.operational = line.station_ids.len() >= 2;
            true
        } else {
            false
        }
    }

    // -------------------------------------------------------------------------
    // Ridership estimation
    // -------------------------------------------------------------------------

    /// Estimate the travel time (in game-hours) for a citizen to use the metro
    /// between two grid positions. Returns `None` if no viable metro route.
    ///
    /// Travel time = walk_to_station + wait + ride + walk_from_station
    pub fn estimate_travel_time(
        &self,
        from_x: usize,
        from_y: usize,
        to_x: usize,
        to_y: usize,
    ) -> Option<f32> {
        // Find nearest station to origin
        let (origin_station, origin_dist) = self.nearest_station(from_x, from_y)?;
        if origin_dist > MAX_WALK_TO_STATION_CELLS {
            return None;
        }

        // Find nearest station to destination
        let (dest_station, dest_dist) = self.nearest_station(to_x, to_y)?;
        if dest_dist > MAX_WALK_TO_STATION_CELLS {
            return None;
        }

        // Both stations must be on the same line (or connected via transfer)
        if !self.stations_connected(origin_station, dest_station) {
            return None;
        }

        // Walk to origin station (hours)
        let walk_to = origin_dist as f32 / WALK_SPEED_CELLS_PER_HOUR;

        // Wait for train (average half headway, in hours)
        let wait = AVERAGE_WAIT_MINUTES / 60.0;

        // Ride distance (Manhattan distance between stations, in cells)
        let ride_dist = self.route_distance(origin_station, dest_station);
        let ride_time = ride_dist as f32 / METRO_SPEED_CELLS_PER_HOUR;

        // Walk from destination station
        let walk_from = dest_dist as f32 / WALK_SPEED_CELLS_PER_HOUR;

        Some(walk_to + wait + ride_time + walk_from)
    }

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

    /// Check if two stations are connected (on the same line or via transfer).
    pub fn stations_connected(&self, a: StationId, b: StationId) -> bool {
        if a == b {
            return true;
        }
        // Direct: both on same line
        for line in &self.lines {
            if !line.operational {
                continue;
            }
            let has_a = line.station_ids.contains(&a);
            let has_b = line.station_ids.contains(&b);
            if has_a && has_b {
                return true;
            }
        }
        // Transfer: station A is on line L1, station B is on line L2,
        // and L1 and L2 share a common station (transfer point).
        let lines_with_a: Vec<&MetroLine> = self
            .lines
            .iter()
            .filter(|l| l.operational && l.station_ids.contains(&a))
            .collect();
        let lines_with_b: Vec<&MetroLine> = self
            .lines
            .iter()
            .filter(|l| l.operational && l.station_ids.contains(&b))
            .collect();

        for la in &lines_with_a {
            for lb in &lines_with_b {
                // Check if any station is shared between the two lines
                if la.station_ids.iter().any(|s| lb.station_ids.contains(s)) {
                    return true;
                }
            }
        }

        false
    }

    /// Calculate the route distance between two stations (in grid cells).
    /// Uses Manhattan distance between the stations for simplicity.
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

    /// Calculate total monthly maintenance cost for all stations and lines.
    pub fn total_monthly_maintenance(&self) -> f64 {
        let station_cost = self.stations.len() as f64 * STATION_WEEKLY_MAINTENANCE * 4.0;
        let line_cost = self.lines.iter().filter(|l| l.operational).count() as f64
            * LINE_WEEKLY_MAINTENANCE
            * 4.0;
        station_cost + line_cost
    }

    /// Estimate daily ridership based on nearby population and network connectivity.
    fn estimate_daily_ridership(&self, population: u32) -> u32 {
        if self.stations.is_empty() || self.lines.iter().all(|l| !l.operational) {
            return 0;
        }

        let operational_stations = self.stations.len() as f32;
        let operational_lines = self.lines.iter().filter(|l| l.operational).count() as f32;

        // Base ridership: fraction of population that uses metro
        // More stations and lines = higher mode share
        let network_factor = (operational_stations * 0.01 + operational_lines * 0.02).min(0.30);
        let base_ridership = population as f32 * network_factor;

        // Capacity constraint: each line can carry TRAIN_CAPACITY * trips_per_day
        let trips_per_day = (24.0 * 60.0 / TRAIN_HEADWAY_MINUTES) as u32;
        let total_capacity = operational_lines as u32 * TRAIN_CAPACITY * trips_per_day;

        (base_ridership as u32).min(total_capacity)
    }
}

impl Saveable for MetroTransitState {
    const SAVE_KEY: &'static str = "metro_transit";

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

/// Update metro statistics and ridership every slow tick.
///
/// - Counts stations and operational lines
/// - Estimates daily ridership based on population and network size
/// - Calculates maintenance costs
/// - Updates per-station ridership counters
pub fn update_metro_stats(
    slow_timer: Res<SlowTickTimer>,
    mut metro: ResMut<MetroTransitState>,
    city_stats: Res<CityStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let total_stations = metro.stations.len() as u32;
    let total_lines = metro.lines.iter().filter(|l| l.operational).count() as u32;
    let daily_ridership = metro.estimate_daily_ridership(city_stats.population);
    let monthly_maintenance = metro.total_monthly_maintenance();

    // Distribute ridership across stations proportionally
    if total_stations > 0 && daily_ridership > 0 {
        let per_station = daily_ridership / total_stations;
        let remainder = daily_ridership % total_stations;
        for (i, station) in metro.stations.iter_mut().enumerate() {
            let riders = per_station + if (i as u32) < remainder { 1 } else { 0 };
            station.period_ridership = riders;
            station.total_ridership += riders as u64;
        }
    } else {
        for station in &mut metro.stations {
            station.period_ridership = 0;
        }
    }

    metro.stats = MetroStats {
        total_stations,
        total_lines,
        daily_ridership,
        monthly_maintenance_cost: monthly_maintenance,
        cumulative_ridership: metro.stations.iter().map(|s| s.total_ridership).sum(),
    };
}

/// Deduct metro maintenance costs from the city budget every 30 days.
///
/// This runs alongside the main tax collection cycle. The cost is based
/// on the number of stations and operational lines.
pub fn deduct_metro_costs(
    slow_timer: Res<SlowTickTimer>,
    metro: Res<MetroTransitState>,
    mut budget: ResMut<CityBudget>,
    clock: Res<crate::time_of_day::GameClock>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Only deduct every ~30 days, aligned with the budget cycle.
    if !clock.day.is_multiple_of(30) {
        return;
    }

    let cost = metro.total_monthly_maintenance();
    if cost > 0.0 {
        budget.treasury -= cost;
    }
}

/// Boost land value around metro stations.
///
/// Each station provides a +15–25 land value bonus in a radius around it,
/// with the boost diminishing linearly with distance. This runs every
/// slow tick after the main land value update.
pub fn metro_land_value_boost(
    slow_timer: Res<SlowTickTimer>,
    metro: Res<MetroTransitState>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for station in &metro.stations {
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

// =============================================================================
// Plugin
// =============================================================================

pub struct MetroTransitPlugin;

impl Plugin for MetroTransitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetroTransitState>().add_systems(
            FixedUpdate,
            (
                update_metro_stats,
                deduct_metro_costs,
                metro_land_value_boost.after(crate::land_value::update_land_value),
            ),
        );

        // Register for save/load via the extension map
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<MetroTransitState>();
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

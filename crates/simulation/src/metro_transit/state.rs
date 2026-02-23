//! Metro transit state resource and core logic.
//!
//! `MetroTransitState` is the source of truth for the metro transit system.
//! It contains all stations, lines, and aggregate statistics. Persisted via
//! the `Saveable` extension map so no save-system files need modification.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::grid::{CellType, WorldGrid};
use crate::Saveable;

use super::constants::*;
use super::types::*;

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
    pub(crate) next_station_id: u32,
    /// Auto-incrementing line ID counter.
    pub(crate) next_line_id: u32,
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
    pub(crate) fn estimate_daily_ridership(&self, population: u32) -> u32 {
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

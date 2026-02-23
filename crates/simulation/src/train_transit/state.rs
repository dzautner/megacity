//! `TrainTransitState` methods and `Saveable` implementation.

use crate::grid::{CellType, WorldGrid};
use crate::Saveable;

use super::types::*;

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
    pub(crate) fn estimate_daily_ridership(&self, population: u32) -> u32 {
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
// Saveable implementation
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

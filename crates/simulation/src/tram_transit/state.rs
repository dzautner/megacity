//! Tram transit data types, constants, and state management.
//!
//! Contains the core data structures (`TramStop`, `TramLine`, `TramVehicle`,
//! `TramTransitState`, `TramTransitStats`) and public API methods for managing
//! tram stops, lines, and transit time estimation.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::grid::{CellType, WorldGrid};

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
pub(crate) fn manhattan_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> u32 {
    let dx = (x1 as i32 - x2 as i32).unsigned_abs();
    let dy = (y1 as i32 - y2 as i32).unsigned_abs();
    dx + dy
}

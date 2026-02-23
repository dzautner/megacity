//! Data types and constants for the train/rail transit system.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

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
// Resource
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
    pub(crate) next_station_id: u32,
    /// Auto-incrementing line ID counter.
    pub(crate) next_line_id: u32,
    /// Last day costs were deducted.
    pub(crate) last_cost_day: u32,
}

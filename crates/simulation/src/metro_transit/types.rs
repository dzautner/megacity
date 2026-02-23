//! Data structures for the metro transit system.

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

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

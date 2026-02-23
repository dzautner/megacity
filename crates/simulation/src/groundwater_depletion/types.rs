//! Constants and types for groundwater depletion tracking.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

// =============================================================================
// Constants
// =============================================================================

/// Critical groundwater level threshold (20% of 255 ≈ 51).
/// When the average groundwater level falls below this, a city-wide critical
/// depletion warning is triggered.
pub const GROUNDWATER_CRITICAL_LEVEL: u8 = 51;

/// Per-cell subsidence groundwater threshold. Cells with groundwater level
/// below this value start accumulating subsidence ticks.
pub const SUBSIDENCE_THRESHOLD: u8 = 20;

/// Number of slow ticks a cell must remain below `SUBSIDENCE_THRESHOLD`
/// before it suffers permanent land subsidence.
pub const SUBSIDENCE_TICKS: u16 = 50;

/// Groundwater level below which well yield begins to diminish.
/// Wells at or above this level produce at full capacity.
pub const WELL_YIELD_REDUCTION_THRESHOLD: u8 = 50;

/// Number of recharge basin grid cells to count per basin for the recharge
/// boost. Each basin boosts recharge in a local radius.
pub const RECHARGE_BASIN_BOOST: f32 = 5.0;

// =============================================================================
// Groundwater depletion state resource
// =============================================================================

/// City-wide groundwater depletion and sustainability tracking.
///
/// Updated each slow tick by `update_groundwater_depletion`. Downstream systems
/// (well pumps, UI overlays, budget) read these metrics to adjust behaviour.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct GroundwaterDepletionState {
    /// Total extraction rate across the grid (sum of level decreases per tick).
    pub extraction_rate: f32,
    /// Total recharge rate across the grid (sum of level increases per tick).
    pub recharge_rate: f32,
    /// Sustainability ratio: `recharge_rate / extraction_rate`.
    /// Values >= 1.0 are sustainable; < 1.0 means over-extraction.
    /// Set to `f32::INFINITY` when extraction is zero.
    pub sustainability_ratio: f32,
    /// `true` when the average groundwater level is below `GROUNDWATER_CRITICAL_LEVEL`.
    pub critical_depletion: bool,
    /// Number of cells that have experienced permanent subsidence.
    pub subsidence_cells: u32,
    /// Global well yield modifier (0.0..=1.0). Applied as a multiplier to all
    /// well pump output. Computed from average groundwater level.
    pub well_yield_modifier: f32,
    /// Per-cell counter tracking how many consecutive slow ticks each cell has
    /// remained below `SUBSIDENCE_THRESHOLD`. Once a cell reaches
    /// `SUBSIDENCE_TICKS` it is flagged as subsided and the counter freezes.
    #[serde(default)]
    pub ticks_below_threshold: Vec<u16>,
    /// Snapshot of groundwater levels from the previous slow tick, used to
    /// compute per-tick extraction and recharge deltas.
    #[serde(default)]
    pub previous_levels: Vec<u8>,
    /// Number of active recharge basins in the city. Set externally by the
    /// building placement system; this module reads it for metric purposes.
    pub recharge_basin_count: u32,
    /// Average groundwater level across the grid (0.0..=255.0).
    pub avg_groundwater_level: f32,
    /// Number of cells currently below `SUBSIDENCE_THRESHOLD` (at risk).
    pub cells_at_risk: u32,
    /// Number of cells flagged as over-extracted (level < `GROUNDWATER_CRITICAL_LEVEL`).
    pub over_extracted_cells: u32,
}

impl Default for GroundwaterDepletionState {
    fn default() -> Self {
        let total = GRID_WIDTH * GRID_HEIGHT;
        Self {
            extraction_rate: 0.0,
            recharge_rate: 0.0,
            sustainability_ratio: f32::INFINITY,
            critical_depletion: false,
            subsidence_cells: 0,
            well_yield_modifier: 1.0,
            ticks_below_threshold: vec![0u16; total],
            previous_levels: Vec::new(),
            recharge_basin_count: 0,
            avg_groundwater_level: 128.0,
            cells_at_risk: 0,
            over_extracted_cells: 0,
        }
    }
}

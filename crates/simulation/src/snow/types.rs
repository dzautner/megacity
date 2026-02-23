//! Snow types, constants, and resources.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

// =============================================================================
// Constants
// =============================================================================

/// Freezing point in Celsius. Snow accumulates when temperature is below this.
pub(crate) const FREEZING_POINT_C: f32 = 0.0;

/// Snow accumulation rate per slow tick during snow events (inches per tick).
/// Scales with precipitation intensity.
pub(crate) const BASE_SNOW_ACCUMULATION_RATE: f32 = 0.5;

/// Snow melt rate per degree Celsius above freezing per slow tick (inches per tick per degree).
pub(crate) const MELT_RATE_PER_DEGREE: f32 = 0.1;

/// Maximum snow depth in inches for gameplay purposes.
pub(crate) const MAX_SNOW_DEPTH: f32 = 24.0;

/// Speed reduction per inch of snow on roads (fraction).
/// Total reduction is clamped at MAX_SNOW_SPEED_REDUCTION.
pub(crate) const SPEED_REDUCTION_PER_INCH: f32 = 0.05;

/// Maximum speed reduction from snow on roads (fraction of normal speed lost).
/// At 12+ inches, roads are at maximum slowdown (80% reduction).
pub(crate) const MAX_SNOW_SPEED_REDUCTION: f32 = 0.80;

/// Heating demand increase per 6 inches of snow (fraction, i.e. 0.10 = +10%).
pub(crate) const HEATING_INCREASE_PER_6_INCHES: f32 = 0.10;

/// Cost per road cell per plowing event (dollars).
pub(crate) const PLOW_COST_PER_CELL: f64 = 500.0;

/// Amount of snow removed per plowing pass (inches).
pub(crate) const PLOW_REMOVAL_DEPTH: f32 = 6.0;

/// Threshold snow depth (inches) above which plowing is triggered on roads.
pub(crate) const PLOW_TRIGGER_DEPTH: f32 = 2.0;

/// Snowmelt contribution to stormwater runoff per inch melted (arbitrary units).
/// Used for spring flooding risk integration.
pub(crate) const SNOWMELT_RUNOFF_FACTOR: f32 = 0.5;

// =============================================================================
// Resources
// =============================================================================

/// Per-cell snow depth grid (inches). 0.0 = no snow.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SnowGrid {
    pub depths: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for SnowGrid {
    fn default() -> Self {
        Self {
            depths: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl SnowGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.depths[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        self.depths[y * self.width + x] = val;
    }

    /// Average snow depth across all cells (for stats/UI).
    pub fn average_depth(&self) -> f32 {
        if self.depths.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.depths.iter().sum();
        sum / self.depths.len() as f32
    }

    /// Number of cells with snow depth > 0.
    pub fn covered_cells(&self) -> u32 {
        self.depths.iter().filter(|&&d| d > 0.0).count() as u32
    }

    /// Maximum snow depth across all cells.
    pub fn max_depth(&self) -> f32 {
        self.depths.iter().copied().fold(0.0_f32, f32::max)
    }
}

/// Aggregate snow plowing service state and statistics.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SnowPlowingState {
    /// Whether the snow plowing service is enabled (player can toggle).
    pub enabled: bool,
    /// Total cost spent on plowing this season.
    pub season_cost: f64,
    /// Number of cells plowed this season.
    pub cells_plowed_season: u32,
    /// Number of cells plowed in the most recent plowing pass.
    pub cells_plowed_last: u32,
    /// Cost of the most recent plowing pass.
    pub last_plow_cost: f64,
}

impl Default for SnowPlowingState {
    fn default() -> Self {
        Self {
            enabled: true,
            season_cost: 0.0,
            cells_plowed_season: 0,
            cells_plowed_last: 0,
            last_plow_cost: 0.0,
        }
    }
}

/// Aggregate snow statistics for the UI.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SnowStats {
    /// Average snow depth across all cells (inches).
    pub avg_depth: f32,
    /// Maximum snow depth on any cell (inches).
    pub max_depth: f32,
    /// Number of cells covered with snow.
    pub covered_cells: u32,
    /// Current travel speed multiplier due to snow on roads (1.0 = no effect).
    pub road_speed_multiplier: f32,
    /// Current heating demand modifier from snow (1.0 = no effect).
    pub heating_demand_modifier: f32,
    /// Total snowmelt runoff contribution this tick (for flooding).
    pub snowmelt_runoff: f32,
}

impl Default for SnowStats {
    fn default() -> Self {
        Self {
            avg_depth: 0.0,
            max_depth: 0.0,
            covered_cells: 0,
            road_speed_multiplier: 1.0,
            heating_demand_modifier: 1.0,
            snowmelt_runoff: 0.0,
        }
    }
}

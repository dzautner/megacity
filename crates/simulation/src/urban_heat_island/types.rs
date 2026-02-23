use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

// ---------------------------------------------------------------------------
// UhiGrid resource
// ---------------------------------------------------------------------------

/// Per-cell temperature increment grid (in Fahrenheit). A positive value means
/// the cell is warmer than the rural baseline; negative values indicate cooling
/// (e.g. parks, water).
///
/// The final effective temperature for any cell is:
///   `base_weather_temperature + uhi_grid.cells[idx]`
#[derive(Resource, Serialize, Deserialize)]
pub struct UhiGrid {
    pub cells: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for UhiGrid {
    fn default() -> Self {
        Self {
            cells: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl UhiGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x]
        } else {
            0.0
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = val;
        }
    }

    /// Compute the effective temperature at a specific cell by adding the UHI
    /// increment to the base weather temperature.
    pub fn effective_temperature(&self, base_temp: f32, x: usize, y: usize) -> f32 {
        base_temp + self.get(x, y)
    }
}

// ---------------------------------------------------------------------------
// Helper: effective temperature (standalone for external callers)
// ---------------------------------------------------------------------------

/// Convenience function returning the final cell temperature given the base
/// weather temperature and the UHI grid value at `(x, y)`.
pub fn effective_temperature(uhi: &UhiGrid, base_temp: f32, x: usize, y: usize) -> f32 {
    uhi.effective_temperature(base_temp, x, y)
}

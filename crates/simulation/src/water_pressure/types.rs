//! Types, constants, and pure helper functions for the water pressure system.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Constants
// =============================================================================

/// Base elevation limit served by the water distribution system without any
/// booster pump stations.
pub const BASE_PRESSURE_ELEVATION: f32 = 50.0;

/// Additional elevation capacity provided by each booster pump station.
pub const BOOSTER_ELEVATION_GAIN: f32 = 30.0;

/// Cost in dollars to build a booster pump station.
pub const BOOSTER_PUMP_COST: f64 = 200_000.0;

/// Elevation range over which water pressure degrades from full (1.0) to zero.
/// Buildings within this margin above the effective pressure elevation receive
/// reduced service quality (linearly interpolated).
pub const PRESSURE_FALLOFF_RANGE: f32 = 10.0;

// =============================================================================
// Components
// =============================================================================

/// Marker component for booster pump station entities.
///
/// Each booster pump station adds `BOOSTER_ELEVATION_GAIN` to the city's
/// effective pressure elevation. Placed on the grid as a 1x1 building.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct BoosterPumpStation {
    /// Grid X position of this pump station.
    pub grid_x: usize,
    /// Grid Y position of this pump station.
    pub grid_y: usize,
}

// =============================================================================
// Pressure category
// =============================================================================

/// Pressure classification for a building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureCategory {
    /// Full water pressure (factor ~1.0).
    Full,
    /// Reduced water pressure (0.0 < factor < 1.0).
    Reduced,
    /// No water pressure (factor ~0.0).
    None,
}

// =============================================================================
// Helper functions (pure, testable)
// =============================================================================

/// Calculate the effective pressure elevation from the number of booster pump
/// stations.
pub fn effective_pressure_elevation(booster_count: u32) -> f32 {
    BASE_PRESSURE_ELEVATION + booster_count as f32 * BOOSTER_ELEVATION_GAIN
}

/// Calculate the water pressure factor for a building at a given elevation.
///
/// Returns a value between 0.0 and 1.0:
/// - 1.0 if the building is at or below the effective elevation (full pressure).
/// - 0.0 if the building is above `effective_elevation + PRESSURE_FALLOFF_RANGE`.
/// - Linearly interpolated in between.
pub fn pressure_factor(building_elevation: f32, effective_elev: f32) -> f32 {
    if building_elevation <= effective_elev {
        1.0
    } else {
        let excess = building_elevation - effective_elev;
        if excess >= PRESSURE_FALLOFF_RANGE {
            0.0
        } else {
            1.0 - (excess / PRESSURE_FALLOFF_RANGE)
        }
    }
}

/// Classify a pressure factor into one of three categories:
/// - Full pressure: factor >= 1.0 (or very close due to floating point).
/// - No pressure: factor <= 0.0.
/// - Reduced pressure: everything in between.
pub fn classify_pressure(factor: f32) -> PressureCategory {
    if factor >= 1.0 - f32::EPSILON {
        PressureCategory::Full
    } else if factor <= f32::EPSILON {
        PressureCategory::None
    } else {
        PressureCategory::Reduced
    }
}

//! Water pressure zone state resource and save/load support.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::types::BASE_PRESSURE_ELEVATION;

// =============================================================================
// Resource
// =============================================================================

/// City-wide water pressure zone state.
///
/// Tracks the effective pressure elevation (base + booster contributions),
/// the number of active booster pump stations, and statistics about buildings
/// served and underserved by the water pressure system.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct WaterPressureState {
    /// Number of active booster pump stations.
    pub booster_count: u32,
    /// Effective maximum elevation served at full pressure.
    /// Equal to `BASE_PRESSURE_ELEVATION + booster_count * BOOSTER_ELEVATION_GAIN`.
    pub effective_elevation: f32,
    /// Number of buildings with full water pressure (elevation <= effective_elevation).
    pub buildings_full_pressure: u32,
    /// Number of buildings with reduced water pressure (in the falloff range).
    pub buildings_reduced_pressure: u32,
    /// Number of buildings with no water pressure (above effective_elevation + falloff).
    pub buildings_no_pressure: u32,
    /// Average pressure factor across all buildings (0.0 to 1.0).
    pub average_pressure_factor: f32,
    /// Total cost of all booster pump stations.
    pub total_booster_cost: f64,
}

impl Default for WaterPressureState {
    fn default() -> Self {
        Self {
            booster_count: 0,
            effective_elevation: BASE_PRESSURE_ELEVATION,
            buildings_full_pressure: 0,
            buildings_reduced_pressure: 0,
            buildings_no_pressure: 0,
            average_pressure_factor: 1.0,
            total_booster_cost: 0.0,
        }
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for WaterPressureState {
    const SAVE_KEY: &'static str = "water_pressure";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if at default state (no boosters).
        if self.booster_count == 0
            && self.buildings_full_pressure == 0
            && self.buildings_reduced_pressure == 0
            && self.buildings_no_pressure == 0
        {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

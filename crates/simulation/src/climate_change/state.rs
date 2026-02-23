//! `ClimateState` resource: tracks cumulative CO2 emissions and resulting climate effects.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Saveable;

/// Tracks cumulative CO2 emissions and resulting climate effects.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct ClimateState {
    /// Total cumulative CO2 emissions in tons since game start.
    pub cumulative_co2: f64,
    /// CO2 emitted during the most recent yearly assessment.
    pub yearly_co2: f64,
    /// Current temperature increase in Fahrenheit due to climate change.
    pub temperature_increase_f: f32,
    /// Disaster frequency multiplier (1.0 = normal, 1.1 = +10%, etc.).
    pub disaster_frequency_multiplier: f32,
    /// Whether sea level rise flooding has been applied.
    pub sea_level_rise_applied: bool,
    /// Number of cells permanently flooded by sea level rise.
    pub flooded_cells_count: u32,
    /// Environmental score (0-100, higher = better/cleaner).
    pub environmental_score: f32,
    /// Last game day a yearly assessment was performed.
    pub last_assessment_day: u32,
    /// Drought duration multiplier (1.0 = normal, higher = longer droughts).
    pub drought_duration_multiplier: f32,
}

impl Default for ClimateState {
    fn default() -> Self {
        Self {
            cumulative_co2: 0.0,
            yearly_co2: 0.0,
            temperature_increase_f: 0.0,
            disaster_frequency_multiplier: 1.0,
            sea_level_rise_applied: false,
            flooded_cells_count: 0,
            environmental_score: 100.0,
            last_assessment_day: 0,
            drought_duration_multiplier: 1.0,
        }
    }
}

impl Saveable for ClimateState {
    const SAVE_KEY: &'static str = "climate_change";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if at default state (no emissions yet)
        if self.cumulative_co2 == 0.0 && self.last_assessment_day == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

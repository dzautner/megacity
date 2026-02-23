//! `HeatMitigationState` resource: aggregate state for all heat wave mitigation
//! measures, including player-controlled toggles, derived effects, and cost
//! tracking.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Aggregate state for all heat wave mitigation measures.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct HeatMitigationState {
    // --- Player-controlled toggles ---
    /// Whether cooling centers are enabled (activate during heat waves).
    pub cooling_centers_enabled: bool,
    /// Whether emergency water distribution is enabled (activate during heat waves).
    pub emergency_water_enabled: bool,
    /// Number of misting stations placed by the player.
    pub misting_station_count: u32,
    /// Number of buildings upgraded with light-colored roofs.
    pub light_roof_count: u32,

    // --- Derived effects (computed each tick) ---
    /// Mortality reduction factor from all active mitigations (0.0 = no reduction, 1.0 = all prevented).
    pub mortality_reduction: f32,
    /// Aggregate temperature reduction from green canopy (Fahrenheit).
    pub green_canopy_temp_reduction: f32,
    /// Temperature reduction from light-colored roofs (Fahrenheit, city-wide average).
    pub light_roof_temp_reduction: f32,
    /// Perceived temperature reduction from misting stations (Fahrenheit).
    pub misting_temp_reduction: f32,
    /// Whether dehydration deaths are prevented (emergency water active during heat wave).
    pub dehydration_prevented: bool,

    // --- Cost tracking ---
    /// Total cost accumulated from mitigation measures this season.
    pub season_cost: f64,
    /// Cost incurred in the most recent update tick.
    pub last_tick_cost: f64,
    /// Total spent on light-colored roof upgrades (cumulative).
    pub light_roof_upgrade_total_cost: f64,

    // --- Internal tracking ---
    /// Last game day for which daily costs were applied.
    pub last_cost_day: u32,
}

impl Default for HeatMitigationState {
    fn default() -> Self {
        Self {
            cooling_centers_enabled: false,
            emergency_water_enabled: false,
            misting_station_count: 0,
            light_roof_count: 0,
            mortality_reduction: 0.0,
            green_canopy_temp_reduction: 0.0,
            light_roof_temp_reduction: 0.0,
            misting_temp_reduction: 0.0,
            dehydration_prevented: false,
            season_cost: 0.0,
            last_tick_cost: 0.0,
            light_roof_upgrade_total_cost: 0.0,
            last_cost_day: 0,
        }
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for HeatMitigationState {
    const SAVE_KEY: &'static str = "heat_mitigation";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if all toggles are off and no stations/upgrades placed
        if !self.cooling_centers_enabled
            && !self.emergency_water_enabled
            && self.misting_station_count == 0
            && self.light_roof_count == 0
            && self.season_cost == 0.0
        {
            return None;
        }
        // Manual binary serialization of persistent fields only.
        // Layout: [cooling:u8, water:u8, misting:u32, roofs:u32,
        //          season_cost:f64, roof_cost:f64, last_day:u32]
        // Total: 2 + 4 + 4 + 8 + 8 + 4 = 30 bytes
        let mut buf = Vec::with_capacity(30);
        buf.push(self.cooling_centers_enabled as u8);
        buf.push(self.emergency_water_enabled as u8);
        buf.extend_from_slice(&self.misting_station_count.to_le_bytes());
        buf.extend_from_slice(&self.light_roof_count.to_le_bytes());
        buf.extend_from_slice(&self.season_cost.to_le_bytes());
        buf.extend_from_slice(&self.light_roof_upgrade_total_cost.to_le_bytes());
        buf.extend_from_slice(&self.last_cost_day.to_le_bytes());
        Some(buf)
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 30 {
            warn!(
                "Saveable {}: expected >= 30 bytes, got {}, falling back to default",
                Self::SAVE_KEY,
                bytes.len()
            );
            return Self::default();
        }
        let cooling = bytes[0] != 0;
        let water = bytes[1] != 0;
        let misting = u32::from_le_bytes(bytes[2..6].try_into().unwrap_or([0; 4]));
        let roofs = u32::from_le_bytes(bytes[6..10].try_into().unwrap_or([0; 4]));
        let season_cost = f64::from_le_bytes(bytes[10..18].try_into().unwrap_or([0; 8]));
        let roof_cost = f64::from_le_bytes(bytes[18..26].try_into().unwrap_or([0; 8]));
        let last_day = u32::from_le_bytes(bytes[26..30].try_into().unwrap_or([0; 4]));
        Self {
            cooling_centers_enabled: cooling,
            emergency_water_enabled: water,
            misting_station_count: misting,
            light_roof_count: roofs,
            season_cost,
            light_roof_upgrade_total_cost: roof_cost,
            last_cost_day: last_day,
            // Derived fields are recomputed by the system
            ..Default::default()
        }
    }
}

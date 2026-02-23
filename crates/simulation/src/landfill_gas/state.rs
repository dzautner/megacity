//! City-wide landfill gas state resource.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::constants::*;

/// City-wide landfill gas generation and collection state.
///
/// Tracks gas generation from all landfills, collection infrastructure status,
/// electricity output, uncaptured methane emissions, and fire/explosion risk.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct LandfillGasState {
    /// Total gas generation across all landfills in cubic feet per year.
    pub total_gas_generation_cf_per_year: f64,
    /// Fraction of generated gas that is methane (nominally 0.50).
    pub methane_fraction: f32,
    /// Fraction of generated gas that is CO2 (nominally 0.50).
    pub co2_fraction: f32,
    /// Whether gas collection infrastructure exists in the city.
    pub collection_active: bool,
    /// Fraction of generated gas that is captured when collection is active (0.0-1.0).
    pub collection_efficiency: f32,
    /// Electricity generated from captured landfill gas in megawatts.
    pub electricity_generated_mw: f32,
    /// Uncaptured methane escaping to atmosphere in cubic feet per year.
    pub uncaptured_methane_cf: f32,
    /// Total capital cost of all collection systems installed.
    pub infrastructure_cost: f64,
    /// Total annual maintenance cost for all collection systems.
    pub maintenance_cost_per_year: f64,
    /// Annual probability of fire/explosion (aggregated across landfills without collection).
    pub fire_explosion_risk: f32,
    /// Number of landfills that have gas collection infrastructure.
    pub landfills_with_collection: u32,
    /// Total number of landfill service buildings in the city.
    pub total_landfills: u32,
}

impl Default for LandfillGasState {
    fn default() -> Self {
        Self {
            total_gas_generation_cf_per_year: 0.0,
            methane_fraction: METHANE_FRACTION,
            co2_fraction: CO2_FRACTION,
            collection_active: false,
            collection_efficiency: COLLECTION_EFFICIENCY_DEFAULT,
            electricity_generated_mw: 0.0,
            uncaptured_methane_cf: 0.0,
            infrastructure_cost: 0.0,
            maintenance_cost_per_year: 0.0,
            fire_explosion_risk: 0.0,
            landfills_with_collection: 0,
            total_landfills: 0,
        }
    }
}

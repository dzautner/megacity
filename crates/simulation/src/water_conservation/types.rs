use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// City-wide water conservation policy state.
///
/// Each boolean field represents an individually toggleable conservation policy.
/// Other simulation systems (e.g. `water_demand`) read `demand_reduction_pct` to
/// apply the aggregate reduction. This resource does NOT modify the `Policy` enum;
/// it is a standalone system with its own policy toggles.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct WaterConservationState {
    /// Low-flow fixtures installed in residential buildings (-20% residential demand).
    pub low_flow_fixtures: bool,
    /// Xeriscaping programme active (-10% total demand from reduced irrigation).
    pub xeriscaping: bool,
    /// Tiered water pricing in effect (-15% total demand from behavioural change).
    pub tiered_pricing: bool,
    /// Greywater recycling enabled (-15% demand AND -30% sewage volume).
    pub greywater_recycling: bool,
    /// Rainwater harvesting systems installed (-10% demand when raining).
    pub rainwater_harvesting: bool,
    /// Aggregate demand reduction percentage from all active policies (0.0 .. 0.60).
    pub demand_reduction_pct: f32,
    /// Sewage volume reduction percentage from greywater recycling (0.0 or 0.30).
    pub sewage_reduction_pct: f32,
    /// Cumulative dollar cost of all building retrofits.
    pub total_retrofit_cost: f64,
    /// Estimated annual water savings in gallons based on current policies and building count.
    pub annual_savings_gallons: f64,
    /// Number of buildings that have been (or would be) retrofitted.
    pub buildings_retrofitted: u32,
}

impl Default for WaterConservationState {
    fn default() -> Self {
        Self {
            low_flow_fixtures: false,
            xeriscaping: false,
            tiered_pricing: false,
            greywater_recycling: false,
            rainwater_harvesting: false,
            demand_reduction_pct: 0.0,
            sewage_reduction_pct: 0.0,
            total_retrofit_cost: 0.0,
            annual_savings_gallons: 0.0,
            buildings_retrofitted: 0,
        }
    }
}

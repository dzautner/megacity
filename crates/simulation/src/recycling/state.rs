//! City-wide recycling program state tracking.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::tiers::RecyclingTier;

/// City-wide recycling program state, updated each slow tick.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct RecyclingState {
    /// Current recycling tier selected by the player.
    pub tier: RecyclingTier,
    /// Tons diverted from landfill this period.
    pub daily_tons_diverted: f64,
    /// Tons contaminated (waste in recycling stream sent to landfill) this period.
    pub daily_tons_contaminated: f64,
    /// Revenue from selling recyclable materials this period.
    pub daily_revenue: f64,
    /// Program operating costs this period (collection + processing).
    pub daily_cost: f64,
    /// Cumulative revenue since game start.
    pub total_revenue: f64,
    /// Cumulative costs since game start.
    pub total_cost: f64,
    /// Number of households participating.
    pub participating_households: u32,
}

impl Default for RecyclingState {
    fn default() -> Self {
        Self {
            tier: RecyclingTier::None,
            daily_tons_diverted: 0.0,
            daily_tons_contaminated: 0.0,
            daily_revenue: 0.0,
            daily_cost: 0.0,
            total_revenue: 0.0,
            total_cost: 0.0,
            participating_households: 0,
        }
    }
}

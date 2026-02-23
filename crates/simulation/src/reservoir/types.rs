use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Constants
// =============================================================================

/// Rainfall intensity (in/hr) to MGD conversion factor for catchment areas.
pub(crate) const CATCHMENT_FACTOR: f32 = 0.001;

/// Base evaporation rate in MGD per reservoir per day.
pub(crate) const BASE_EVAPORATION_RATE: f32 = 0.005;

/// Additional evaporation per degree Celsius above 20C (MGD per reservoir).
pub(crate) const TEMPERATURE_EVAP_FACTOR: f32 = 0.03;

/// Minimum reserve percentage (below this triggers Critical tier).
pub(crate) const MIN_RESERVE_PCT: f32 = 0.20;

/// Gallons per million gallons.
pub(crate) const MGD_TO_GPD: f32 = 1_000_000.0;

// =============================================================================
// Types
// =============================================================================

/// Warning tier based on reservoir fill percentage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ReservoirWarningTier {
    /// Fill > 50%: operating normally.
    #[default]
    Normal,
    /// Fill 30%-50%: conservation advisories.
    Watch,
    /// Fill 20%-30%: mandatory restrictions.
    Warning,
    /// Fill <= 20% (at or below min reserve): emergency measures.
    Critical,
}

impl ReservoirWarningTier {
    /// Human-readable name for UI display.
    pub fn name(self) -> &'static str {
        match self {
            ReservoirWarningTier::Normal => "Normal",
            ReservoirWarningTier::Watch => "Watch",
            ReservoirWarningTier::Warning => "Warning",
            ReservoirWarningTier::Critical => "Critical",
        }
    }
}

/// Determine the warning tier from a fill percentage (0.0 to 1.0).
pub fn warning_tier_from_fill(fill_pct: f32) -> ReservoirWarningTier {
    if fill_pct > 0.50 {
        ReservoirWarningTier::Normal
    } else if fill_pct > 0.30 {
        ReservoirWarningTier::Watch
    } else if fill_pct > MIN_RESERVE_PCT {
        ReservoirWarningTier::Warning
    } else {
        ReservoirWarningTier::Critical
    }
}

/// Event fired when the reservoir warning tier changes.
#[derive(Event, Debug, Clone)]
pub struct ReservoirWarningEvent {
    /// The previous warning tier.
    pub old_tier: ReservoirWarningTier,
    /// The new warning tier.
    pub new_tier: ReservoirWarningTier,
    /// Current fill percentage when the event was fired (0.0 to 1.0).
    pub fill_pct: f32,
}

/// City-wide reservoir statistics resource.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ReservoirState {
    /// Total storage capacity across all reservoirs (million gallons).
    pub total_storage_capacity_mg: f32,
    /// Current total water level across all reservoirs (million gallons).
    pub current_level_mg: f32,
    /// Inflow rate from rainfall catchment (million gallons per day).
    pub inflow_rate_mgd: f32,
    /// Outflow rate from water demand extraction (million gallons per day).
    pub outflow_rate_mgd: f32,
    /// Evaporation rate (million gallons per day).
    pub evaporation_rate_mgd: f32,
    /// Net change = inflow - outflow - evaporation (million gallons per day).
    pub net_change_mgd: f32,
    /// Days of supply remaining at current demand rate.
    pub storage_days: f32,
    /// Number of active reservoir entities.
    pub reservoir_count: u32,
    /// Current warning tier based on fill percentage.
    pub warning_tier: ReservoirWarningTier,
    /// Minimum reserve percentage threshold (default 0.20 = 20%).
    pub min_reserve_pct: f32,
}

impl Default for ReservoirState {
    fn default() -> Self {
        Self {
            total_storage_capacity_mg: 0.0,
            current_level_mg: 0.0,
            inflow_rate_mgd: 0.0,
            outflow_rate_mgd: 0.0,
            evaporation_rate_mgd: 0.0,
            net_change_mgd: 0.0,
            storage_days: 0.0,
            reservoir_count: 0,
            warning_tier: ReservoirWarningTier::Normal,
            min_reserve_pct: MIN_RESERVE_PCT,
        }
    }
}

impl ReservoirState {
    /// Current fill percentage (0.0 to 1.0). Returns 0.0 if no capacity.
    pub fn fill_pct(&self) -> f32 {
        if self.total_storage_capacity_mg > 0.0 {
            self.current_level_mg / self.total_storage_capacity_mg
        } else {
            0.0
        }
    }
}

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Warning tier indicating how much landfill capacity remains.
///
/// Ordered from least severe (Normal) to most severe (Emergency).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LandfillWarningTier {
    /// More than 25% capacity remaining. No warnings needed.
    #[default]
    Normal,
    /// 10%--25% capacity remaining. Advisory warning.
    Low,
    /// 5%--10% capacity remaining. Urgent warning.
    Critical,
    /// 0%--5% capacity remaining. Severe warning.
    VeryLow,
    /// 0% capacity remaining. Collection halted.
    Emergency,
}

impl LandfillWarningTier {
    /// Returns a human-readable label for the tier.
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Low => "Low Capacity",
            Self::Critical => "Critical",
            Self::VeryLow => "Very Low",
            Self::Emergency => "Emergency",
        }
    }
}

/// Event fired whenever the landfill warning tier changes.
#[derive(Event, Debug, Clone)]
pub struct LandfillWarningEvent {
    /// The new warning tier after the change.
    pub tier: LandfillWarningTier,
    /// Remaining capacity as a percentage (0.0 to 100.0).
    pub remaining_pct: f32,
}

/// City-wide landfill capacity tracking resource.
///
/// Updated each slow tick by `update_landfill_capacity`. Other systems can read
/// `collection_halted` to stop waste collection when capacity is exhausted.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct LandfillCapacityState {
    /// Total landfill capacity across all Landfill buildings (tons).
    pub total_capacity: f64,
    /// Current fill level (tons). Increases each slow tick by daily input.
    pub current_fill: f64,
    /// Daily waste input rate going to landfills (tons/day).
    pub daily_input_rate: f64,
    /// Estimated days until capacity is exhausted at current fill rate.
    pub days_remaining: f32,
    /// Estimated years until capacity is exhausted (days_remaining / 365).
    pub years_remaining: f32,
    /// Remaining capacity as a percentage (0.0 to 100.0).
    pub remaining_pct: f32,
    /// Current warning tier derived from remaining_pct.
    pub current_tier: LandfillWarningTier,
    /// When true (Emergency tier), waste collection should be halted by
    /// downstream systems.
    pub collection_halted: bool,
    /// Number of Landfill service buildings in the city.
    pub landfill_count: u32,
}

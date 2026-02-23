//! Constants, per-district configuration, and global state resource for inclusionary zoning.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use std::collections::HashMap;

// =============================================================================
// Constants
// =============================================================================

/// Minimum affordable housing percentage (10%).
pub const MIN_AFFORDABLE_PERCENTAGE: f32 = 0.10;

/// Maximum affordable housing percentage (20%).
pub const MAX_AFFORDABLE_PERCENTAGE: f32 = 0.20;

/// Default affordable housing percentage when policy is first enabled (15%).
pub const DEFAULT_AFFORDABLE_PERCENTAGE: f32 = 0.15;

/// Minimum FAR bonus granted to offset affordable unit cost (10%).
pub const MIN_FAR_BONUS: f32 = 0.10;

/// Maximum FAR bonus granted to offset affordable unit cost (20%).
pub const MAX_FAR_BONUS: f32 = 0.20;

/// Construction rate penalty multiplier when inclusionary zoning is active.
/// Developers build slightly slower due to reduced profitability.
pub const CONSTRUCTION_RATE_PENALTY: f32 = 0.90;

/// Monthly administrative cost per district with inclusionary zoning.
pub const MONTHLY_ADMIN_COST_PER_DISTRICT: f64 = 8_000.0;

// =============================================================================
// Per-district configuration
// =============================================================================

/// Per-district inclusionary zoning configuration.
#[derive(Debug, Clone, Encode, Decode)]
pub struct DistrictInclusionaryConfig {
    /// Whether inclusionary zoning is enabled for this district.
    pub enabled: bool,
    /// Percentage of units that must be affordable (0.10 - 0.20).
    pub affordable_percentage: f32,
}

impl Default for DistrictInclusionaryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            affordable_percentage: DEFAULT_AFFORDABLE_PERCENTAGE,
        }
    }
}

// =============================================================================
// Resource: Inclusionary Zoning State
// =============================================================================

/// Tracks inclusionary zoning policy state across all player-defined districts.
#[derive(Resource, Debug, Clone, Default, Encode, Decode)]
pub struct InclusionaryZoningState {
    /// Per-district inclusionary zoning configurations.
    /// Key is the district index in the DistrictMap.
    pub district_configs: HashMap<usize, DistrictInclusionaryConfig>,
    /// Total affordable units across all districts (computed).
    pub total_affordable_units: u32,
    /// Total residential units in affected districts (computed).
    pub total_affected_units: u32,
    /// Total monthly admin cost (computed).
    pub total_monthly_cost: f64,
}

impl InclusionaryZoningState {
    /// Enable inclusionary zoning for a district with the default percentage.
    pub fn enable(&mut self, district_idx: usize) {
        let config = self.district_configs.entry(district_idx).or_default();
        config.enabled = true;
    }

    /// Disable inclusionary zoning for a district.
    pub fn disable(&mut self, district_idx: usize) {
        if let Some(config) = self.district_configs.get_mut(&district_idx) {
            config.enabled = false;
        }
    }

    /// Check if inclusionary zoning is enabled for a district.
    pub fn is_enabled(&self, district_idx: usize) -> bool {
        self.district_configs
            .get(&district_idx)
            .is_some_and(|c| c.enabled)
    }

    /// Set the affordable percentage for a district (clamped to valid range).
    pub fn set_affordable_percentage(&mut self, district_idx: usize, pct: f32) {
        let clamped = pct.clamp(MIN_AFFORDABLE_PERCENTAGE, MAX_AFFORDABLE_PERCENTAGE);
        let config = self.district_configs.entry(district_idx).or_default();
        config.affordable_percentage = clamped;
    }

    /// Get the affordable percentage for a district (returns 0.0 if not enabled).
    pub fn affordable_percentage(&self, district_idx: usize) -> f32 {
        self.district_configs
            .get(&district_idx)
            .filter(|c| c.enabled)
            .map(|c| c.affordable_percentage)
            .unwrap_or(0.0)
    }

    /// Get the number of enabled districts.
    pub fn enabled_district_count(&self) -> usize {
        self.district_configs.values().filter(|c| c.enabled).count()
    }
}

//! Core types and constants for per-district zone policies.

use bitcode::{Decode, Encode};
use std::collections::HashMap;

use bevy::prelude::*;

// =============================================================================
// Constants
// =============================================================================

/// Default service budget multiplier (1.0 = 100% funded).
pub const DEFAULT_SERVICE_BUDGET_MULTIPLIER: f32 = 1.0;

/// Minimum service budget multiplier.
pub const MIN_SERVICE_BUDGET_MULTIPLIER: f32 = 0.0;

/// Maximum service budget multiplier.
pub const MAX_SERVICE_BUDGET_MULTIPLIER: f32 = 2.0;

/// Maximum building level when high-rise ban is active in a district.
pub const HIGH_RISE_BAN_MAX_LEVEL: u8 = 2;

/// Normal maximum building level.
pub const NORMAL_MAX_LEVEL: u8 = 3;

/// Commercial demand bonus from small business incentive.
pub const SMALL_BUSINESS_DEMAND_BONUS: f32 = 0.15;

/// Monthly cost per district for small business incentive.
pub const SMALL_BUSINESS_MONTHLY_COST: f64 = 20.0;

/// Noise reduction factor from noise ordinance (multiplier on noise penalty).
pub const NOISE_ORDINANCE_REDUCTION: f32 = 0.5;

/// Monthly cost per district for noise ordinance enforcement.
pub const NOISE_ORDINANCE_MONTHLY_COST: f64 = 10.0;

/// Park effectiveness bonus from green space mandate.
pub const GREEN_SPACE_MANDATE_BONUS: f32 = 0.25;

/// Monthly cost per district for green space mandate.
pub const GREEN_SPACE_MANDATE_MONTHLY_COST: f64 = 15.0;

// =============================================================================
// Per-district policy flags and overrides
// =============================================================================

/// Policy overrides that can be applied to an individual district.
///
/// Each field is optional; `None` means the city-wide default applies.
/// Boolean policies default to `false` (inactive).
#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct DistrictPolicyOverrides {
    /// Override residential tax rate for this district.
    pub residential_tax: Option<f32>,
    /// Override commercial tax rate for this district.
    pub commercial_tax: Option<f32>,
    /// Override industrial tax rate for this district.
    pub industrial_tax: Option<f32>,
    /// Override office tax rate for this district.
    pub office_tax: Option<f32>,
    /// Whether high-rises are banned in this district (caps level at 2).
    pub high_rise_ban: bool,
    /// Whether heavy industry is banned in this district.
    pub heavy_industry_ban: bool,
    /// Whether the small business incentive is active in this district.
    pub small_business_incentive: bool,
    /// Whether a noise ordinance is active in this district.
    pub noise_ordinance: bool,
    /// Whether a green space mandate is active in this district.
    pub green_space_mandate: bool,
    /// Service budget multiplier for this district (None = use city-wide).
    pub service_budget_multiplier: Option<f32>,
}

impl DistrictPolicyOverrides {
    /// Returns true if all policy settings are at their default (no overrides).
    pub fn is_default(&self) -> bool {
        self.residential_tax.is_none()
            && self.commercial_tax.is_none()
            && self.industrial_tax.is_none()
            && self.office_tax.is_none()
            && !self.high_rise_ban
            && !self.heavy_industry_ban
            && !self.small_business_incentive
            && !self.noise_ordinance
            && !self.green_space_mandate
            && self.service_budget_multiplier.is_none()
    }

    /// Count the number of active boolean policies in this district.
    pub fn active_policy_count(&self) -> u32 {
        let mut count = 0u32;
        if self.high_rise_ban {
            count += 1;
        }
        if self.heavy_industry_ban {
            count += 1;
        }
        if self.small_business_incentive {
            count += 1;
        }
        if self.noise_ordinance {
            count += 1;
        }
        if self.green_space_mandate {
            count += 1;
        }
        count
    }

    /// Calculate the monthly cost of active policies in this district.
    pub fn monthly_cost(&self) -> f64 {
        let mut cost = 0.0;
        if self.small_business_incentive {
            cost += SMALL_BUSINESS_MONTHLY_COST;
        }
        if self.noise_ordinance {
            cost += NOISE_ORDINANCE_MONTHLY_COST;
        }
        if self.green_space_mandate {
            cost += GREEN_SPACE_MANDATE_MONTHLY_COST;
        }
        cost
    }
}

// =============================================================================
// Resource: district policy state
// =============================================================================

/// Stores per-district policy overrides for all player-defined districts.
///
/// Keyed by district index (matching `DistrictMap.districts` indices).
/// Districts without entries use city-wide defaults.
#[derive(Resource, Debug, Clone, Default, Encode, Decode)]
pub struct DistrictPolicyState {
    /// Per-district policy overrides, keyed by district index.
    pub overrides: HashMap<usize, DistrictPolicyOverrides>,
    /// Total monthly cost of all district policies (computed each slow tick).
    pub total_monthly_cost: f64,
    /// Total number of active district-level policies (computed each slow tick).
    pub total_active_policies: u32,
}

impl DistrictPolicyState {
    /// Get the policy overrides for a district, or `None` if no overrides exist.
    pub fn get(&self, district_idx: usize) -> Option<&DistrictPolicyOverrides> {
        self.overrides.get(&district_idx)
    }

    /// Get a mutable reference to the policy overrides for a district,
    /// creating a default entry if one doesn't exist.
    pub fn get_or_create_mut(&mut self, district_idx: usize) -> &mut DistrictPolicyOverrides {
        self.overrides.entry(district_idx).or_default()
    }

    /// Remove all overrides for a district, reverting to city-wide defaults.
    pub fn clear_overrides(&mut self, district_idx: usize) {
        self.overrides.remove(&district_idx);
    }

    /// Set a district-level tax rate override.
    pub fn set_residential_tax(&mut self, district_idx: usize, rate: f32) {
        self.get_or_create_mut(district_idx).residential_tax = Some(rate.clamp(0.0, 0.30));
    }

    /// Set a district-level commercial tax rate override.
    pub fn set_commercial_tax(&mut self, district_idx: usize, rate: f32) {
        self.get_or_create_mut(district_idx).commercial_tax = Some(rate.clamp(0.0, 0.30));
    }

    /// Set a district-level industrial tax rate override.
    pub fn set_industrial_tax(&mut self, district_idx: usize, rate: f32) {
        self.get_or_create_mut(district_idx).industrial_tax = Some(rate.clamp(0.0, 0.30));
    }

    /// Set a district-level office tax rate override.
    pub fn set_office_tax(&mut self, district_idx: usize, rate: f32) {
        self.get_or_create_mut(district_idx).office_tax = Some(rate.clamp(0.0, 0.30));
    }

    /// Set the service budget multiplier for a district.
    pub fn set_service_budget_multiplier(&mut self, district_idx: usize, multiplier: f32) {
        self.get_or_create_mut(district_idx)
            .service_budget_multiplier =
            Some(multiplier.clamp(MIN_SERVICE_BUDGET_MULTIPLIER, MAX_SERVICE_BUDGET_MULTIPLIER));
    }

    /// Toggle a boolean policy for a district.
    pub fn toggle_high_rise_ban(&mut self, district_idx: usize) {
        let o = self.get_or_create_mut(district_idx);
        o.high_rise_ban = !o.high_rise_ban;
    }

    /// Toggle heavy industry ban for a district.
    pub fn toggle_heavy_industry_ban(&mut self, district_idx: usize) {
        let o = self.get_or_create_mut(district_idx);
        o.heavy_industry_ban = !o.heavy_industry_ban;
    }

    /// Toggle small business incentive for a district.
    pub fn toggle_small_business_incentive(&mut self, district_idx: usize) {
        let o = self.get_or_create_mut(district_idx);
        o.small_business_incentive = !o.small_business_incentive;
    }

    /// Toggle noise ordinance for a district.
    pub fn toggle_noise_ordinance(&mut self, district_idx: usize) {
        let o = self.get_or_create_mut(district_idx);
        o.noise_ordinance = !o.noise_ordinance;
    }

    /// Toggle green space mandate for a district.
    pub fn toggle_green_space_mandate(&mut self, district_idx: usize) {
        let o = self.get_or_create_mut(district_idx);
        o.green_space_mandate = !o.green_space_mandate;
    }
}

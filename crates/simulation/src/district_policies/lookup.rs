//! Precomputed lookup tables and pure helper functions for district policies.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::budget::ZoneTaxRates;
use crate::districts::DistrictMap;

use super::types::*;

// =============================================================================
// Resource: precomputed lookup for per-district effective values
// =============================================================================

/// Precomputed per-district effective policy values, updated each slow tick.
///
/// Other systems query this resource to get the effective tax rate, building
/// height limit, etc. for a given grid cell, taking district overrides into
/// account and falling back to city-wide defaults.
#[derive(Resource, Debug, Clone, Default)]
pub struct DistrictPolicyLookup {
    /// Per-district effective tax rates (district_idx -> effective rates).
    pub effective_taxes: HashMap<usize, ZoneTaxRates>,
    /// Per-district max building level (district_idx -> max level).
    pub max_building_level: HashMap<usize, u8>,
    /// Per-district heavy industry ban flag (district_idx -> banned).
    pub heavy_industry_banned: HashMap<usize, bool>,
    /// Per-district commercial demand bonus (district_idx -> bonus).
    pub commercial_demand_bonus: HashMap<usize, f32>,
    /// Per-district noise reduction multiplier (district_idx -> multiplier).
    pub noise_multiplier: HashMap<usize, f32>,
    /// Per-district park effectiveness multiplier (district_idx -> multiplier).
    pub park_multiplier: HashMap<usize, f32>,
    /// Per-district service budget multiplier (district_idx -> multiplier).
    pub service_budget_multiplier: HashMap<usize, f32>,
}

impl DistrictPolicyLookup {
    /// Get the effective residential tax rate for a cell.
    /// Returns the district override if the cell is in a district with one,
    /// otherwise falls back to the city-wide rate.
    pub fn effective_residential_tax(
        &self,
        x: usize,
        y: usize,
        district_map: &DistrictMap,
        city_wide: &ZoneTaxRates,
    ) -> f32 {
        if let Some(di) = district_map.get_district_index_at(x, y) {
            if let Some(taxes) = self.effective_taxes.get(&di) {
                return taxes.residential;
            }
        }
        city_wide.residential
    }

    /// Get the effective commercial tax rate for a cell.
    pub fn effective_commercial_tax(
        &self,
        x: usize,
        y: usize,
        district_map: &DistrictMap,
        city_wide: &ZoneTaxRates,
    ) -> f32 {
        if let Some(di) = district_map.get_district_index_at(x, y) {
            if let Some(taxes) = self.effective_taxes.get(&di) {
                return taxes.commercial;
            }
        }
        city_wide.commercial
    }

    /// Get the effective industrial tax rate for a cell.
    pub fn effective_industrial_tax(
        &self,
        x: usize,
        y: usize,
        district_map: &DistrictMap,
        city_wide: &ZoneTaxRates,
    ) -> f32 {
        if let Some(di) = district_map.get_district_index_at(x, y) {
            if let Some(taxes) = self.effective_taxes.get(&di) {
                return taxes.industrial;
            }
        }
        city_wide.industrial
    }

    /// Get the effective office tax rate for a cell.
    pub fn effective_office_tax(
        &self,
        x: usize,
        y: usize,
        district_map: &DistrictMap,
        city_wide: &ZoneTaxRates,
    ) -> f32 {
        if let Some(di) = district_map.get_district_index_at(x, y) {
            if let Some(taxes) = self.effective_taxes.get(&di) {
                return taxes.office;
            }
        }
        city_wide.office
    }

    /// Get the effective max building level for a cell.
    /// Returns the district limit if set, otherwise the normal max.
    pub fn effective_max_building_level(
        &self,
        x: usize,
        y: usize,
        district_map: &DistrictMap,
    ) -> u8 {
        if let Some(di) = district_map.get_district_index_at(x, y) {
            if let Some(&max_level) = self.max_building_level.get(&di) {
                return max_level;
            }
        }
        NORMAL_MAX_LEVEL
    }

    /// Check if heavy industry is banned at a cell.
    pub fn is_heavy_industry_banned(&self, x: usize, y: usize, district_map: &DistrictMap) -> bool {
        if let Some(di) = district_map.get_district_index_at(x, y) {
            if let Some(&banned) = self.heavy_industry_banned.get(&di) {
                return banned;
            }
        }
        false
    }

    /// Get the commercial demand bonus for a cell from district policies.
    pub fn district_commercial_bonus(&self, x: usize, y: usize, district_map: &DistrictMap) -> f32 {
        if let Some(di) = district_map.get_district_index_at(x, y) {
            if let Some(&bonus) = self.commercial_demand_bonus.get(&di) {
                return bonus;
            }
        }
        0.0
    }

    /// Get the noise multiplier for a cell (1.0 = normal, lower = less noise impact).
    pub fn district_noise_multiplier(&self, x: usize, y: usize, district_map: &DistrictMap) -> f32 {
        if let Some(di) = district_map.get_district_index_at(x, y) {
            if let Some(&mult) = self.noise_multiplier.get(&di) {
                return mult;
            }
        }
        1.0
    }

    /// Get the park effectiveness multiplier for a cell.
    pub fn district_park_multiplier(&self, x: usize, y: usize, district_map: &DistrictMap) -> f32 {
        if let Some(di) = district_map.get_district_index_at(x, y) {
            if let Some(&mult) = self.park_multiplier.get(&di) {
                return mult;
            }
        }
        1.0
    }

    /// Get the service budget multiplier for a cell.
    pub fn district_service_multiplier(
        &self,
        x: usize,
        y: usize,
        district_map: &DistrictMap,
    ) -> f32 {
        if let Some(di) = district_map.get_district_index_at(x, y) {
            if let Some(&mult) = self.service_budget_multiplier.get(&di) {
                return mult;
            }
        }
        DEFAULT_SERVICE_BUDGET_MULTIPLIER
    }
}

// =============================================================================
// Pure helper functions (testable without ECS)
// =============================================================================

/// Compute the effective tax rates for a district, given overrides and city-wide defaults.
pub fn compute_effective_taxes(
    overrides: &DistrictPolicyOverrides,
    city_wide: &ZoneTaxRates,
) -> ZoneTaxRates {
    ZoneTaxRates {
        residential: overrides.residential_tax.unwrap_or(city_wide.residential),
        commercial: overrides.commercial_tax.unwrap_or(city_wide.commercial),
        industrial: overrides.industrial_tax.unwrap_or(city_wide.industrial),
        office: overrides.office_tax.unwrap_or(city_wide.office),
    }
}

/// Compute the effective max building level for a district.
pub fn compute_max_building_level(overrides: &DistrictPolicyOverrides) -> u8 {
    if overrides.high_rise_ban {
        HIGH_RISE_BAN_MAX_LEVEL
    } else {
        NORMAL_MAX_LEVEL
    }
}

/// Compute the commercial demand bonus for a district.
pub fn compute_commercial_bonus(overrides: &DistrictPolicyOverrides) -> f32 {
    if overrides.small_business_incentive {
        SMALL_BUSINESS_DEMAND_BONUS
    } else {
        0.0
    }
}

/// Compute the noise multiplier for a district.
pub fn compute_noise_multiplier(overrides: &DistrictPolicyOverrides) -> f32 {
    if overrides.noise_ordinance {
        NOISE_ORDINANCE_REDUCTION
    } else {
        1.0
    }
}

/// Compute the park effectiveness multiplier for a district.
pub fn compute_park_multiplier(overrides: &DistrictPolicyOverrides) -> f32 {
    if overrides.green_space_mandate {
        1.0 + GREEN_SPACE_MANDATE_BONUS
    } else {
        1.0
    }
}

/// Compute the effective service budget multiplier for a district.
pub fn compute_service_multiplier(overrides: &DistrictPolicyOverrides) -> f32 {
    overrides
        .service_budget_multiplier
        .unwrap_or(DEFAULT_SERVICE_BUDGET_MULTIPLIER)
}

/// Compute the total monthly cost across all districts.
pub fn compute_total_monthly_cost(overrides: &HashMap<usize, DistrictPolicyOverrides>) -> f64 {
    overrides.values().map(|o| o.monthly_cost()).sum()
}

/// Compute the total active policy count across all districts.
pub fn compute_total_active_policies(overrides: &HashMap<usize, DistrictPolicyOverrides>) -> u32 {
    overrides.values().map(|o| o.active_policy_count()).sum()
}

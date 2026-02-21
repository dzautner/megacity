//! Per-District Zone Policies (ZONE-015).
//!
//! Allows policies to be applied per-district instead of city-wide only.
//! Each district can have different tax rate overrides, building height limits,
//! heavy industry bans, service budget multipliers, and more.
//!
//! Supported per-district policies:
//! - **Tax rate overrides**: residential, commercial, industrial, office tax rates
//! - **High-rise ban**: caps building level at 2 in the district
//! - **Heavy industry ban**: prevents industrial zoning in the district
//! - **Small business incentive**: boosts commercial demand in the district
//! - **Noise ordinance**: reduces happiness penalty from noise in the district
//! - **Green space mandate**: boosts park effectiveness in the district
//! - **Service budget multiplier**: scales service effectiveness in the district
//!
//! The system runs on the slow tick timer to compute per-district effective
//! policy values that other systems can query via `DistrictPolicyLookup`.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use std::collections::HashMap;

use crate::budget::ZoneTaxRates;
use crate::districts::DistrictMap;
use crate::SlowTickTimer;

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

// =============================================================================
// System
// =============================================================================

/// System: update district policy lookup tables every slow tick.
///
/// Reads `DistrictPolicyState` and city-wide `ExtendedBudget` to compute
/// effective per-district values, writing them to `DistrictPolicyLookup`.
pub fn update_district_policies(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<DistrictPolicyState>,
    budget: Res<crate::budget::ExtendedBudget>,
    district_map: Res<DistrictMap>,
    mut lookup: ResMut<DistrictPolicyLookup>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let city_wide_taxes = &budget.zone_taxes;
    let num_districts = district_map.districts.len();

    // Clear previous lookup data
    lookup.effective_taxes.clear();
    lookup.max_building_level.clear();
    lookup.heavy_industry_banned.clear();
    lookup.commercial_demand_bonus.clear();
    lookup.noise_multiplier.clear();
    lookup.park_multiplier.clear();
    lookup.service_budget_multiplier.clear();

    // Compute effective values for each district that has overrides
    for (&di, overrides) in &state.overrides {
        if di >= num_districts {
            continue;
        }

        // Tax rates
        let effective = compute_effective_taxes(overrides, city_wide_taxes);
        lookup.effective_taxes.insert(di, effective);

        // Building level
        let max_level = compute_max_building_level(overrides);
        if max_level != NORMAL_MAX_LEVEL {
            lookup.max_building_level.insert(di, max_level);
        }

        // Heavy industry ban
        if overrides.heavy_industry_ban {
            lookup.heavy_industry_banned.insert(di, true);
        }

        // Commercial demand bonus
        let bonus = compute_commercial_bonus(overrides);
        if bonus > 0.0 {
            lookup.commercial_demand_bonus.insert(di, bonus);
        }

        // Noise multiplier
        let noise_mult = compute_noise_multiplier(overrides);
        if (noise_mult - 1.0).abs() > f32::EPSILON {
            lookup.noise_multiplier.insert(di, noise_mult);
        }

        // Park multiplier
        let park_mult = compute_park_multiplier(overrides);
        if (park_mult - 1.0).abs() > f32::EPSILON {
            lookup.park_multiplier.insert(di, park_mult);
        }

        // Service budget multiplier
        let service_mult = compute_service_multiplier(overrides);
        if (service_mult - DEFAULT_SERVICE_BUDGET_MULTIPLIER).abs() > f32::EPSILON {
            lookup.service_budget_multiplier.insert(di, service_mult);
        }
    }

    // Update aggregate stats
    state.total_monthly_cost = compute_total_monthly_cost(&state.overrides);
    state.total_active_policies = compute_total_active_policies(&state.overrides);
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for DistrictPolicyState {
    const SAVE_KEY: &'static str = "district_policies";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no districts have any overrides
        if self.overrides.is_empty() {
            return None;
        }
        // Also skip if all overrides are at defaults
        if self.overrides.values().all(|o| o.is_default()) {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct DistrictPoliciesPlugin;

impl Plugin for DistrictPoliciesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DistrictPolicyState>()
            .init_resource::<DistrictPolicyLookup>()
            .add_systems(
                FixedUpdate,
                update_district_policies
                    .after(crate::districts::district_stats)
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<DistrictPolicyState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Default state tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state_empty() {
        let state = DistrictPolicyState::default();
        assert!(state.overrides.is_empty());
        assert_eq!(state.total_monthly_cost, 0.0);
        assert_eq!(state.total_active_policies, 0);
    }

    #[test]
    fn test_default_overrides_are_default() {
        let overrides = DistrictPolicyOverrides::default();
        assert!(overrides.is_default());
        assert_eq!(overrides.active_policy_count(), 0);
        assert!(overrides.monthly_cost().abs() < f64::EPSILON);
    }

    #[test]
    fn test_default_lookup_empty() {
        let lookup = DistrictPolicyLookup::default();
        assert!(lookup.effective_taxes.is_empty());
        assert!(lookup.max_building_level.is_empty());
        assert!(lookup.heavy_industry_banned.is_empty());
    }

    // -------------------------------------------------------------------------
    // DistrictPolicyOverrides tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_overrides_is_not_default_with_tax() {
        let mut o = DistrictPolicyOverrides::default();
        o.residential_tax = Some(0.15);
        assert!(!o.is_default());
    }

    #[test]
    fn test_overrides_is_not_default_with_policy() {
        let mut o = DistrictPolicyOverrides::default();
        o.high_rise_ban = true;
        assert!(!o.is_default());
    }

    #[test]
    fn test_overrides_active_policy_count() {
        let mut o = DistrictPolicyOverrides::default();
        assert_eq!(o.active_policy_count(), 0);

        o.high_rise_ban = true;
        assert_eq!(o.active_policy_count(), 1);

        o.heavy_industry_ban = true;
        assert_eq!(o.active_policy_count(), 2);

        o.small_business_incentive = true;
        assert_eq!(o.active_policy_count(), 3);

        o.noise_ordinance = true;
        assert_eq!(o.active_policy_count(), 4);

        o.green_space_mandate = true;
        assert_eq!(o.active_policy_count(), 5);
    }

    #[test]
    fn test_overrides_monthly_cost_none() {
        let o = DistrictPolicyOverrides::default();
        assert!(o.monthly_cost().abs() < f64::EPSILON);
    }

    #[test]
    fn test_overrides_monthly_cost_all() {
        let mut o = DistrictPolicyOverrides::default();
        o.small_business_incentive = true;
        o.noise_ordinance = true;
        o.green_space_mandate = true;
        let expected = SMALL_BUSINESS_MONTHLY_COST
            + NOISE_ORDINANCE_MONTHLY_COST
            + GREEN_SPACE_MANDATE_MONTHLY_COST;
        assert!((o.monthly_cost() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_overrides_monthly_cost_free_policies() {
        let mut o = DistrictPolicyOverrides::default();
        o.high_rise_ban = true;
        o.heavy_industry_ban = true;
        assert!(o.monthly_cost().abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // State mutation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_set_residential_tax() {
        let mut state = DistrictPolicyState::default();
        state.set_residential_tax(0, 0.15);
        let o = state.get(0).unwrap();
        assert!((o.residential_tax.unwrap() - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_tax_clamped() {
        let mut state = DistrictPolicyState::default();
        state.set_residential_tax(0, 0.50); // Over 30% limit
        let o = state.get(0).unwrap();
        assert!((o.residential_tax.unwrap() - 0.30).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_tax_clamped_negative() {
        let mut state = DistrictPolicyState::default();
        state.set_residential_tax(0, -0.10);
        let o = state.get(0).unwrap();
        assert!(o.residential_tax.unwrap().abs() < f32::EPSILON);
    }

    #[test]
    fn test_toggle_high_rise_ban() {
        let mut state = DistrictPolicyState::default();
        state.toggle_high_rise_ban(0);
        assert!(state.get(0).unwrap().high_rise_ban);
        state.toggle_high_rise_ban(0);
        assert!(!state.get(0).unwrap().high_rise_ban);
    }

    #[test]
    fn test_toggle_heavy_industry_ban() {
        let mut state = DistrictPolicyState::default();
        state.toggle_heavy_industry_ban(0);
        assert!(state.get(0).unwrap().heavy_industry_ban);
    }

    #[test]
    fn test_toggle_small_business() {
        let mut state = DistrictPolicyState::default();
        state.toggle_small_business_incentive(0);
        assert!(state.get(0).unwrap().small_business_incentive);
    }

    #[test]
    fn test_toggle_noise_ordinance() {
        let mut state = DistrictPolicyState::default();
        state.toggle_noise_ordinance(0);
        assert!(state.get(0).unwrap().noise_ordinance);
    }

    #[test]
    fn test_toggle_green_space_mandate() {
        let mut state = DistrictPolicyState::default();
        state.toggle_green_space_mandate(0);
        assert!(state.get(0).unwrap().green_space_mandate);
    }

    #[test]
    fn test_set_service_budget_multiplier() {
        let mut state = DistrictPolicyState::default();
        state.set_service_budget_multiplier(0, 1.5);
        let o = state.get(0).unwrap();
        assert!((o.service_budget_multiplier.unwrap() - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_service_budget_multiplier_clamped() {
        let mut state = DistrictPolicyState::default();
        state.set_service_budget_multiplier(0, 5.0);
        let o = state.get(0).unwrap();
        assert!(
            (o.service_budget_multiplier.unwrap() - MAX_SERVICE_BUDGET_MULTIPLIER).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_clear_overrides() {
        let mut state = DistrictPolicyState::default();
        state.toggle_high_rise_ban(0);
        state.set_residential_tax(0, 0.15);
        assert!(state.get(0).is_some());

        state.clear_overrides(0);
        assert!(state.get(0).is_none());
    }

    #[test]
    fn test_multiple_districts() {
        let mut state = DistrictPolicyState::default();
        state.toggle_high_rise_ban(0);
        state.toggle_heavy_industry_ban(1);
        state.set_residential_tax(2, 0.05);

        assert!(state.get(0).unwrap().high_rise_ban);
        assert!(!state.get(0).unwrap().heavy_industry_ban);
        assert!(state.get(1).unwrap().heavy_industry_ban);
        assert!(!state.get(1).unwrap().high_rise_ban);
        assert!((state.get(2).unwrap().residential_tax.unwrap() - 0.05).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Pure helper function tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_compute_effective_taxes_no_overrides() {
        let overrides = DistrictPolicyOverrides::default();
        let city_wide = ZoneTaxRates {
            residential: 0.10,
            commercial: 0.12,
            industrial: 0.08,
            office: 0.11,
        };
        let effective = compute_effective_taxes(&overrides, &city_wide);
        assert!((effective.residential - 0.10).abs() < f32::EPSILON);
        assert!((effective.commercial - 0.12).abs() < f32::EPSILON);
        assert!((effective.industrial - 0.08).abs() < f32::EPSILON);
        assert!((effective.office - 0.11).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_effective_taxes_with_overrides() {
        let mut overrides = DistrictPolicyOverrides::default();
        overrides.residential_tax = Some(0.05);
        overrides.industrial_tax = Some(0.20);
        let city_wide = ZoneTaxRates::default();

        let effective = compute_effective_taxes(&overrides, &city_wide);
        assert!((effective.residential - 0.05).abs() < f32::EPSILON);
        assert!((effective.commercial - city_wide.commercial).abs() < f32::EPSILON);
        assert!((effective.industrial - 0.20).abs() < f32::EPSILON);
        assert!((effective.office - city_wide.office).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_max_building_level_normal() {
        let overrides = DistrictPolicyOverrides::default();
        assert_eq!(compute_max_building_level(&overrides), NORMAL_MAX_LEVEL);
    }

    #[test]
    fn test_compute_max_building_level_banned() {
        let mut overrides = DistrictPolicyOverrides::default();
        overrides.high_rise_ban = true;
        assert_eq!(
            compute_max_building_level(&overrides),
            HIGH_RISE_BAN_MAX_LEVEL
        );
    }

    #[test]
    fn test_compute_commercial_bonus_none() {
        let overrides = DistrictPolicyOverrides::default();
        assert!(compute_commercial_bonus(&overrides).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_commercial_bonus_active() {
        let mut overrides = DistrictPolicyOverrides::default();
        overrides.small_business_incentive = true;
        assert!(
            (compute_commercial_bonus(&overrides) - SMALL_BUSINESS_DEMAND_BONUS).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_compute_noise_multiplier_normal() {
        let overrides = DistrictPolicyOverrides::default();
        assert!((compute_noise_multiplier(&overrides) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_noise_multiplier_ordinance() {
        let mut overrides = DistrictPolicyOverrides::default();
        overrides.noise_ordinance = true;
        assert!(
            (compute_noise_multiplier(&overrides) - NOISE_ORDINANCE_REDUCTION).abs() < f32::EPSILON
        );
    }

    #[test]
    fn test_compute_park_multiplier_normal() {
        let overrides = DistrictPolicyOverrides::default();
        assert!((compute_park_multiplier(&overrides) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_park_multiplier_mandate() {
        let mut overrides = DistrictPolicyOverrides::default();
        overrides.green_space_mandate = true;
        let expected = 1.0 + GREEN_SPACE_MANDATE_BONUS;
        assert!((compute_park_multiplier(&overrides) - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_service_multiplier_default() {
        let overrides = DistrictPolicyOverrides::default();
        assert!(
            (compute_service_multiplier(&overrides) - DEFAULT_SERVICE_BUDGET_MULTIPLIER).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_compute_service_multiplier_custom() {
        let mut overrides = DistrictPolicyOverrides::default();
        overrides.service_budget_multiplier = Some(1.5);
        assert!((compute_service_multiplier(&overrides) - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_total_monthly_cost_empty() {
        let overrides = HashMap::new();
        assert!(compute_total_monthly_cost(&overrides).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_total_monthly_cost_multiple_districts() {
        let mut overrides = HashMap::new();
        let mut o1 = DistrictPolicyOverrides::default();
        o1.small_business_incentive = true;
        let mut o2 = DistrictPolicyOverrides::default();
        o2.noise_ordinance = true;
        o2.green_space_mandate = true;
        overrides.insert(0, o1);
        overrides.insert(1, o2);

        let expected = SMALL_BUSINESS_MONTHLY_COST
            + NOISE_ORDINANCE_MONTHLY_COST
            + GREEN_SPACE_MANDATE_MONTHLY_COST;
        assert!((compute_total_monthly_cost(&overrides) - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_total_active_policies() {
        let mut overrides = HashMap::new();
        let mut o1 = DistrictPolicyOverrides::default();
        o1.high_rise_ban = true;
        o1.heavy_industry_ban = true;
        let mut o2 = DistrictPolicyOverrides::default();
        o2.noise_ordinance = true;
        overrides.insert(0, o1);
        overrides.insert(1, o2);

        assert_eq!(compute_total_active_policies(&overrides), 3);
    }

    // -------------------------------------------------------------------------
    // Lookup tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_lookup_fallback_to_city_wide() {
        let lookup = DistrictPolicyLookup::default();
        let district_map = DistrictMap::default();
        let city_wide = ZoneTaxRates {
            residential: 0.12,
            commercial: 0.14,
            industrial: 0.08,
            office: 0.10,
        };

        // Cell not in any district
        assert!(
            (lookup.effective_residential_tax(10, 10, &district_map, &city_wide) - 0.12).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_lookup_district_override() {
        let mut lookup = DistrictPolicyLookup::default();
        let mut district_map = DistrictMap::default();
        let city_wide = ZoneTaxRates::default();

        // Assign cell to district 0
        district_map.assign_cell_to_district(10, 10, 0);

        // Set override for district 0
        lookup.effective_taxes.insert(
            0,
            ZoneTaxRates {
                residential: 0.05,
                commercial: 0.05,
                industrial: 0.05,
                office: 0.05,
            },
        );

        assert!(
            (lookup.effective_residential_tax(10, 10, &district_map, &city_wide) - 0.05).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_lookup_max_building_level_default() {
        let lookup = DistrictPolicyLookup::default();
        let district_map = DistrictMap::default();
        assert_eq!(
            lookup.effective_max_building_level(10, 10, &district_map),
            NORMAL_MAX_LEVEL
        );
    }

    #[test]
    fn test_lookup_max_building_level_banned() {
        let mut lookup = DistrictPolicyLookup::default();
        let mut district_map = DistrictMap::default();

        district_map.assign_cell_to_district(10, 10, 0);
        lookup.max_building_level.insert(0, HIGH_RISE_BAN_MAX_LEVEL);

        assert_eq!(
            lookup.effective_max_building_level(10, 10, &district_map),
            HIGH_RISE_BAN_MAX_LEVEL
        );
    }

    #[test]
    fn test_lookup_heavy_industry_default() {
        let lookup = DistrictPolicyLookup::default();
        let district_map = DistrictMap::default();
        assert!(!lookup.is_heavy_industry_banned(10, 10, &district_map));
    }

    #[test]
    fn test_lookup_heavy_industry_banned() {
        let mut lookup = DistrictPolicyLookup::default();
        let mut district_map = DistrictMap::default();

        district_map.assign_cell_to_district(10, 10, 0);
        lookup.heavy_industry_banned.insert(0, true);

        assert!(lookup.is_heavy_industry_banned(10, 10, &district_map));
    }

    #[test]
    fn test_lookup_commercial_bonus_default() {
        let lookup = DistrictPolicyLookup::default();
        let district_map = DistrictMap::default();
        assert!(
            lookup
                .district_commercial_bonus(10, 10, &district_map)
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_lookup_noise_multiplier_default() {
        let lookup = DistrictPolicyLookup::default();
        let district_map = DistrictMap::default();
        assert!(
            (lookup.district_noise_multiplier(10, 10, &district_map) - 1.0).abs() < f32::EPSILON
        );
    }

    #[test]
    fn test_lookup_park_multiplier_default() {
        let lookup = DistrictPolicyLookup::default();
        let district_map = DistrictMap::default();
        assert!(
            (lookup.district_park_multiplier(10, 10, &district_map) - 1.0).abs() < f32::EPSILON
        );
    }

    #[test]
    fn test_lookup_service_multiplier_default() {
        let lookup = DistrictPolicyLookup::default();
        let district_map = DistrictMap::default();
        assert!(
            (lookup.district_service_multiplier(10, 10, &district_map)
                - DEFAULT_SERVICE_BUDGET_MULTIPLIER)
                .abs()
                < f32::EPSILON
        );
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = DistrictPolicyState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_skips_all_default_overrides() {
        use crate::Saveable;
        let mut state = DistrictPolicyState::default();
        // Insert an entry but leave all fields at default
        state
            .overrides
            .insert(0, DistrictPolicyOverrides::default());
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_active() {
        use crate::Saveable;
        let mut state = DistrictPolicyState::default();
        state.toggle_high_rise_ban(0);
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = DistrictPolicyState::default();
        state.set_residential_tax(0, 0.15);
        state.toggle_high_rise_ban(0);
        state.toggle_heavy_industry_ban(1);
        state.toggle_small_business_incentive(1);
        state.toggle_noise_ordinance(2);
        state.toggle_green_space_mandate(2);
        state.set_service_budget_multiplier(3, 1.5);

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = DistrictPolicyState::load_from_bytes(&bytes);

        let o0 = restored.get(0).unwrap();
        assert!((o0.residential_tax.unwrap() - 0.15).abs() < f32::EPSILON);
        assert!(o0.high_rise_ban);

        let o1 = restored.get(1).unwrap();
        assert!(o1.heavy_industry_ban);
        assert!(o1.small_business_incentive);

        let o2 = restored.get(2).unwrap();
        assert!(o2.noise_ordinance);
        assert!(o2.green_space_mandate);

        let o3 = restored.get(3).unwrap();
        assert!((o3.service_budget_multiplier.unwrap() - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(DistrictPolicyState::SAVE_KEY, "district_policies");
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_are_reasonable() {
        assert!(DEFAULT_SERVICE_BUDGET_MULTIPLIER > 0.0);
        assert!(MIN_SERVICE_BUDGET_MULTIPLIER >= 0.0);
        assert!(MAX_SERVICE_BUDGET_MULTIPLIER > DEFAULT_SERVICE_BUDGET_MULTIPLIER);
        assert!(HIGH_RISE_BAN_MAX_LEVEL < NORMAL_MAX_LEVEL);
        assert!(SMALL_BUSINESS_DEMAND_BONUS > 0.0);
        assert!(NOISE_ORDINANCE_REDUCTION > 0.0);
        assert!(NOISE_ORDINANCE_REDUCTION < 1.0);
        assert!(GREEN_SPACE_MANDATE_BONUS > 0.0);
    }

    #[test]
    fn test_monthly_costs_are_positive() {
        assert!(SMALL_BUSINESS_MONTHLY_COST > 0.0);
        assert!(NOISE_ORDINANCE_MONTHLY_COST > 0.0);
        assert!(GREEN_SPACE_MANDATE_MONTHLY_COST > 0.0);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_five_policies_per_district() {
        let mut o = DistrictPolicyOverrides::default();
        o.high_rise_ban = true;
        o.heavy_industry_ban = true;
        o.small_business_incentive = true;
        o.noise_ordinance = true;
        o.green_space_mandate = true;
        assert_eq!(o.active_policy_count(), 5);
    }

    #[test]
    fn test_full_district_policy_setup() {
        let mut state = DistrictPolicyState::default();

        // Downtown: high taxes, high-rise ban
        state.set_residential_tax(0, 0.15);
        state.set_commercial_tax(0, 0.20);
        state.toggle_high_rise_ban(0);

        // Industrial district: low taxes, heavy industry allowed
        state.set_industrial_tax(1, 0.05);
        state.toggle_small_business_incentive(1);

        // Suburbs: noise ordinance, green space mandate, lower service budget
        state.toggle_noise_ordinance(2);
        state.toggle_green_space_mandate(2);
        state.set_service_budget_multiplier(2, 0.8);

        assert_eq!(state.overrides.len(), 3);

        // Verify each district
        let downtown = state.get(0).unwrap();
        assert!((downtown.residential_tax.unwrap() - 0.15).abs() < f32::EPSILON);
        assert!((downtown.commercial_tax.unwrap() - 0.20).abs() < f32::EPSILON);
        assert!(downtown.high_rise_ban);

        let industrial = state.get(1).unwrap();
        assert!((industrial.industrial_tax.unwrap() - 0.05).abs() < f32::EPSILON);
        assert!(industrial.small_business_incentive);

        let suburbs = state.get(2).unwrap();
        assert!(suburbs.noise_ordinance);
        assert!(suburbs.green_space_mandate);
        assert!((suburbs.service_budget_multiplier.unwrap() - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_different_districts_different_taxes() {
        let mut state = DistrictPolicyState::default();
        state.set_residential_tax(0, 0.05);
        state.set_residential_tax(1, 0.20);

        let city_wide = ZoneTaxRates::default();
        let o0 = state.get(0).unwrap();
        let o1 = state.get(1).unwrap();

        let eff0 = compute_effective_taxes(o0, &city_wide);
        let eff1 = compute_effective_taxes(o1, &city_wide);

        assert!((eff0.residential - 0.05).abs() < f32::EPSILON);
        assert!((eff1.residential - 0.20).abs() < f32::EPSILON);
        // Both should fall back to city-wide for other rates
        assert!((eff0.commercial - city_wide.commercial).abs() < f32::EPSILON);
        assert!((eff1.commercial - city_wide.commercial).abs() < f32::EPSILON);
    }
}

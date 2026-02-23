//! Unit tests for district policy types and state mutations.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::budget::ZoneTaxRates;
    use crate::district_policies::lookup::*;
    use crate::district_policies::types::*;

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
}

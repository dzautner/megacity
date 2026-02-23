//! Unit tests for district policy lookup and save/load.

#[cfg(test)]
mod tests {
    use crate::budget::ZoneTaxRates;
    use crate::district_policies::lookup::*;
    use crate::district_policies::types::*;
    use crate::districts::DistrictMap;

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
    // Integration-style tests
    // -------------------------------------------------------------------------

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

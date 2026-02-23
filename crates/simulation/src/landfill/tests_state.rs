//! Tests for LandfillState, waste distribution, helper functions, and save/load.

#[cfg(test)]
mod tests {
    use crate::landfill::constants::*;
    use crate::landfill::state::*;
    use crate::landfill::types::*;

    // -------------------------------------------------------------------------
    // LandfillState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_default() {
        let state = LandfillState::default();
        assert!(state.sites.is_empty());
        assert_eq!(state.next_id, 0);
        assert!((state.total_capacity_tons).abs() < f64::EPSILON);
        assert!((state.total_fill_tons).abs() < f64::EPSILON);
        assert_eq!(state.active_sites, 0);
        assert_eq!(state.closed_sites, 0);
        assert_eq!(state.park_sites, 0);
    }

    #[test]
    fn test_add_site() {
        let mut state = LandfillState::default();
        let id = state.add_site(10, 20);
        assert_eq!(id, 0);
        assert_eq!(state.sites.len(), 1);
        assert_eq!(state.next_id, 1);
        assert_eq!(state.sites[0].grid_x, 10);
        assert_eq!(state.sites[0].grid_y, 20);
    }

    #[test]
    fn test_add_site_increments_id() {
        let mut state = LandfillState::default();
        let id1 = state.add_site(0, 0);
        let id2 = state.add_site(1, 1);
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(state.sites.len(), 2);
    }

    #[test]
    fn test_add_site_with_options() {
        let mut state = LandfillState::default();
        let id =
            state.add_site_with_options(5, 10, 1_000_000.0, LandfillLinerType::LinedWithCollection);
        assert_eq!(id, 0);
        let site = state.get_site(0).unwrap();
        assert!((site.total_capacity_tons - 1_000_000.0).abs() < f64::EPSILON);
        assert_eq!(site.liner_type, LandfillLinerType::LinedWithCollection);
    }

    #[test]
    fn test_get_site() {
        let mut state = LandfillState::default();
        state.add_site(10, 20);
        let site = state.get_site(0);
        assert!(site.is_some());
        assert_eq!(site.unwrap().grid_x, 10);
    }

    #[test]
    fn test_get_site_not_found() {
        let state = LandfillState::default();
        assert!(state.get_site(99).is_none());
    }

    #[test]
    fn test_get_site_mut() {
        let mut state = LandfillState::default();
        state.add_site(10, 20);
        let site = state.get_site_mut(0).unwrap();
        site.current_fill_tons = 1000.0;
        assert!((state.sites[0].current_fill_tons - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recompute_aggregates_empty() {
        let mut state = LandfillState::default();
        state.recompute_aggregates();
        assert!((state.total_capacity_tons).abs() < f64::EPSILON);
        assert_eq!(state.active_sites, 0);
        assert_eq!(state.remaining_pct, 0.0);
    }

    #[test]
    fn test_recompute_aggregates_active_sites() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        state.add_site(1, 1);
        state.sites[0].daily_input_tons = 100.0;
        state.sites[1].daily_input_tons = 200.0;
        state.sites[0].current_fill_tons = 100_000.0;

        state.recompute_aggregates();

        assert!(
            (state.total_capacity_tons - 2.0 * DEFAULT_LANDFILL_CAPACITY_TONS).abs() < f64::EPSILON
        );
        assert!((state.total_fill_tons - 100_000.0).abs() < f64::EPSILON);
        assert_eq!(state.active_sites, 2);
        assert!((state.total_daily_input_tons - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recompute_aggregates_closed_sites() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        state.sites[0].status = LandfillStatus::Closed {
            days_since_closure: 100,
        };

        state.recompute_aggregates();

        assert_eq!(state.active_sites, 0);
        assert_eq!(state.closed_sites, 1);
        // Closed sites don't contribute to active capacity
        assert!((state.total_capacity_tons).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recompute_aggregates_park_sites() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        state.sites[0].status = LandfillStatus::ConvertedToPark;

        state.recompute_aggregates();

        assert_eq!(state.park_sites, 1);
        assert_eq!(state.active_sites, 0);
        assert_eq!(state.closed_sites, 0);
    }

    #[test]
    fn test_recompute_gas_electricity() {
        let mut state = LandfillState::default();
        state.add_site_with_options(0, 0, 1_000_000.0, LandfillLinerType::LinedWithCollection);
        state.sites[0].daily_input_tons = 1000.0;

        state.recompute_aggregates();

        // 1000 tons/day * 1.0 MW / 1000 = 1.0 MW
        assert!((state.total_gas_electricity_mw - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recompute_remaining_pct() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        state.sites[0].current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS * 0.75;

        state.recompute_aggregates();

        assert!((state.remaining_pct - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_recompute_years_remaining() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        state.sites[0].daily_input_tons = 1000.0;

        state.recompute_aggregates();

        // 500,000 tons / 1000 tons/day / 365 days/year = ~1.37 years
        let expected = 500_000.0 / 1000.0 / 365.0;
        assert!((state.estimated_years_remaining - expected as f32).abs() < 0.01);
    }

    #[test]
    fn test_recompute_years_remaining_no_input() {
        let mut state = LandfillState::default();
        state.add_site(0, 0);

        state.recompute_aggregates();

        assert!(state.estimated_years_remaining.is_infinite());
    }

    // -------------------------------------------------------------------------
    // distribute_waste tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_distribute_waste_single_site() {
        let mut sites = vec![LandfillSite::new(0, 0, 0)];
        distribute_waste(&mut sites, 100.0);
        assert!((sites[0].current_fill_tons - 100.0).abs() < f64::EPSILON);
        assert!((sites[0].daily_input_tons - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distribute_waste_two_equal_sites() {
        let mut sites = vec![LandfillSite::new(0, 0, 0), LandfillSite::new(1, 1, 1)];
        distribute_waste(&mut sites, 200.0);
        // Both have equal remaining capacity, so waste should split evenly
        assert!((sites[0].current_fill_tons - 100.0).abs() < 0.01);
        assert!((sites[1].current_fill_tons - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_distribute_waste_proportional() {
        let mut sites = vec![
            LandfillSite::with_capacity_and_liner(0, 0, 0, 300_000.0, LandfillLinerType::Unlined),
            LandfillSite::with_capacity_and_liner(1, 1, 1, 100_000.0, LandfillLinerType::Unlined),
        ];
        distribute_waste(&mut sites, 400.0);
        // 300k / 400k = 75% -> 300 tons
        // 100k / 400k = 25% -> 100 tons
        assert!((sites[0].current_fill_tons - 300.0).abs() < 0.01);
        assert!((sites[1].current_fill_tons - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_distribute_waste_skips_closed_sites() {
        let mut sites = vec![LandfillSite::new(0, 0, 0), LandfillSite::new(1, 1, 1)];
        sites[0].status = LandfillStatus::Closed {
            days_since_closure: 0,
        };
        distribute_waste(&mut sites, 100.0);
        assert!((sites[0].current_fill_tons).abs() < f64::EPSILON);
        assert!((sites[1].current_fill_tons - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distribute_waste_no_active_sites() {
        let mut sites = vec![LandfillSite::new(0, 0, 0)];
        sites[0].status = LandfillStatus::ConvertedToPark;
        distribute_waste(&mut sites, 100.0);
        // Nothing should change
        assert!((sites[0].current_fill_tons).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distribute_waste_zero_input() {
        let mut sites = vec![LandfillSite::new(0, 0, 0)];
        distribute_waste(&mut sites, 0.0);
        assert!((sites[0].current_fill_tons).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // environmental_effects tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_environmental_effects_unlined() {
        let (odor, penalty, pollution) = environmental_effects(LandfillLinerType::Unlined);
        assert_eq!(odor, 15);
        assert!((penalty - 0.40).abs() < f32::EPSILON);
        assert!((pollution - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn test_environmental_effects_lined() {
        let (odor, penalty, pollution) = environmental_effects(LandfillLinerType::Lined);
        assert_eq!(odor, 10);
        assert!((penalty - 0.25).abs() < f32::EPSILON);
        assert!((pollution - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_environmental_effects_lined_collection() {
        let (odor, penalty, pollution) =
            environmental_effects(LandfillLinerType::LinedWithCollection);
        assert_eq!(odor, 5);
        assert!((penalty - 0.15).abs() < f32::EPSILON);
        assert!((pollution - 0.05).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // calculate_gas_electricity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_gas_electricity_no_collection_fn() {
        let mw = calculate_gas_electricity(1000.0, false);
        assert!((mw).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gas_electricity_with_collection_fn() {
        let mw = calculate_gas_electricity(1000.0, true);
        assert!((mw - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gas_electricity_2000_tons() {
        let mw = calculate_gas_electricity(2000.0, true);
        assert!((mw - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gas_electricity_zero_input() {
        let mw = calculate_gas_electricity(0.0, true);
        assert!((mw).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(LandfillState::SAVE_KEY, "landfill_state");
    }

    #[test]
    fn test_saveable_skips_empty() {
        use crate::Saveable;
        let state = LandfillState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_with_sites() {
        use crate::Saveable;
        let mut state = LandfillState::default();
        state.add_site(0, 0);
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = LandfillState::default();
        state.add_site_with_options(5, 10, 1_000_000.0, LandfillLinerType::LinedWithCollection);
        state.sites[0].current_fill_tons = 500_000.0;
        state.sites[0].daily_input_tons = 100.0;
        state.add_site(20, 30);
        state.sites[1].current_fill_tons = 200_000.0;
        state.sites[1].status = LandfillStatus::Closed {
            days_since_closure: 365,
        };
        state.recompute_aggregates();

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = LandfillState::load_from_bytes(&bytes);

        assert_eq!(restored.sites.len(), 2);
        assert_eq!(restored.next_id, 2);
        assert!((restored.sites[0].current_fill_tons - 500_000.0).abs() < f64::EPSILON);
        assert_eq!(
            restored.sites[0].liner_type,
            LandfillLinerType::LinedWithCollection
        );
        assert!(!restored.sites[1].status.is_active());
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_lifecycle_fill_close_monitor_park() {
        let mut site =
            LandfillSite::with_capacity_and_liner(0, 0, 0, 1000.0, LandfillLinerType::Lined);

        // Fill the landfill over time
        for _ in 0..9 {
            site.advance_fill(100.0);
            assert!(site.status.is_active());
        }
        assert!((site.current_fill_tons - 900.0).abs() < f64::EPSILON);

        // Fill to capacity - should trigger closure
        site.advance_fill(100.0);
        assert!(!site.status.is_active());
        assert!((site.current_fill_tons - 1000.0).abs() < f64::EPSILON);

        // Advance through 30 years of monitoring
        let monitoring_days = (POST_CLOSURE_MONITORING_YEARS as f32 * DAYS_PER_YEAR) as u32;
        for _ in 0..monitoring_days - 1 {
            site.advance_closure();
            assert!(!site.status.is_active());
            assert_ne!(site.status, LandfillStatus::ConvertedToPark);
        }

        // Last day of monitoring - should convert to park
        site.advance_closure();
        assert_eq!(site.status, LandfillStatus::ConvertedToPark);
    }

    #[test]
    fn test_multiple_sites_one_fills_other_takes_over() {
        let mut sites = vec![
            LandfillSite::with_capacity_and_liner(0, 0, 0, 100.0, LandfillLinerType::Unlined),
            LandfillSite::with_capacity_and_liner(1, 1, 1, 1000.0, LandfillLinerType::Lined),
        ];

        // First distribution: proportional to remaining capacity
        distribute_waste(&mut sites, 200.0);

        // Site 0 should get ~100/1100 * 200 = ~18.18 tons
        // Site 1 should get ~1000/1100 * 200 = ~181.82 tons
        let total = sites[0].current_fill_tons + sites[1].current_fill_tons;
        assert!((total - 200.0).abs() < 0.01);

        // Fill site 0 completely
        sites[0].current_fill_tons = 100.0;
        sites[0].status = LandfillStatus::Closed {
            days_since_closure: 0,
        };

        // Now all waste goes to site 1
        distribute_waste(&mut sites, 100.0);
        // Site 0 should not receive more waste
        assert!((sites[0].current_fill_tons - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_constant_values_match_spec() {
        // Verify all constants match the issue specification
        assert_eq!(ODOR_RADIUS_UNLINED, 15);
        assert_eq!(ODOR_RADIUS_LINED, 10);
        assert_eq!(ODOR_RADIUS_LINED_COLLECTION, 5);
        assert!((LAND_VALUE_PENALTY_UNLINED - 0.40).abs() < f32::EPSILON);
        assert!((LAND_VALUE_PENALTY_LINED_COLLECTION - 0.15).abs() < f32::EPSILON);
        assert_eq!(POST_CLOSURE_MONITORING_YEARS, 30);
        assert!((GAS_COLLECTION_MW_PER_1000_TONS_DAY - 1.0).abs() < f64::EPSILON);
    }
}

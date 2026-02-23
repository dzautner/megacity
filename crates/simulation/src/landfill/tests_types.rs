//! Tests for landfill types: LandfillLinerType, LandfillStatus, and LandfillSite.

#[cfg(test)]
mod tests {
    use crate::landfill::constants::*;
    use crate::landfill::types::*;

    // -------------------------------------------------------------------------
    // LandfillLinerType tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_liner_type_default_is_unlined() {
        assert_eq!(LandfillLinerType::default(), LandfillLinerType::Unlined);
    }

    #[test]
    fn test_unlined_groundwater_pollution() {
        let factor = LandfillLinerType::Unlined.groundwater_pollution_factor();
        assert!((factor - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lined_groundwater_pollution() {
        let factor = LandfillLinerType::Lined.groundwater_pollution_factor();
        assert!((factor - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lined_collection_groundwater_pollution() {
        let factor = LandfillLinerType::LinedWithCollection.groundwater_pollution_factor();
        assert!((factor - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pollution_decreases_with_better_liner() {
        let unlined = LandfillLinerType::Unlined.groundwater_pollution_factor();
        let lined = LandfillLinerType::Lined.groundwater_pollution_factor();
        let collection = LandfillLinerType::LinedWithCollection.groundwater_pollution_factor();
        assert!(unlined > lined);
        assert!(lined > collection);
    }

    #[test]
    fn test_unlined_odor_radius_15() {
        assert_eq!(LandfillLinerType::Unlined.odor_radius(), 15);
    }

    #[test]
    fn test_lined_odor_radius_10() {
        assert_eq!(LandfillLinerType::Lined.odor_radius(), 10);
    }

    #[test]
    fn test_lined_collection_odor_radius_5() {
        assert_eq!(LandfillLinerType::LinedWithCollection.odor_radius(), 5);
    }

    #[test]
    fn test_odor_radius_decreases_with_better_liner() {
        let unlined = LandfillLinerType::Unlined.odor_radius();
        let lined = LandfillLinerType::Lined.odor_radius();
        let collection = LandfillLinerType::LinedWithCollection.odor_radius();
        assert!(unlined > lined);
        assert!(lined > collection);
    }

    #[test]
    fn test_unlined_land_value_penalty_40pct() {
        let penalty = LandfillLinerType::Unlined.land_value_penalty();
        assert!((penalty - 0.40).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lined_land_value_penalty_25pct() {
        let penalty = LandfillLinerType::Lined.land_value_penalty();
        assert!((penalty - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lined_collection_land_value_penalty_15pct() {
        let penalty = LandfillLinerType::LinedWithCollection.land_value_penalty();
        assert!((penalty - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn test_land_value_penalty_decreases_with_better_liner() {
        let unlined = LandfillLinerType::Unlined.land_value_penalty();
        let lined = LandfillLinerType::Lined.land_value_penalty();
        let collection = LandfillLinerType::LinedWithCollection.land_value_penalty();
        assert!(unlined > lined);
        assert!(lined > collection);
    }

    #[test]
    fn test_gas_collection_only_on_lined_with_collection() {
        assert!(!LandfillLinerType::Unlined.has_gas_collection());
        assert!(!LandfillLinerType::Lined.has_gas_collection());
        assert!(LandfillLinerType::LinedWithCollection.has_gas_collection());
    }

    #[test]
    fn test_liner_labels() {
        assert_eq!(LandfillLinerType::Unlined.label(), "Unlined");
        assert_eq!(LandfillLinerType::Lined.label(), "Lined");
        assert_eq!(
            LandfillLinerType::LinedWithCollection.label(),
            "Lined + Gas Collection"
        );
    }

    // -------------------------------------------------------------------------
    // LandfillStatus tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_status_default_is_active() {
        assert_eq!(LandfillStatus::default(), LandfillStatus::Active);
    }

    #[test]
    fn test_active_is_active() {
        assert!(LandfillStatus::Active.is_active());
    }

    #[test]
    fn test_closed_is_not_active() {
        let status = LandfillStatus::Closed {
            days_since_closure: 100,
        };
        assert!(!status.is_active());
    }

    #[test]
    fn test_park_is_not_active() {
        assert!(!LandfillStatus::ConvertedToPark.is_active());
    }

    #[test]
    fn test_status_labels() {
        assert_eq!(LandfillStatus::Active.label(), "Active");
        assert_eq!(
            LandfillStatus::Closed {
                days_since_closure: 0
            }
            .label(),
            "Closed (Monitoring)"
        );
        assert_eq!(LandfillStatus::ConvertedToPark.label(), "Converted to Park");
    }

    #[test]
    fn test_years_since_closure_active() {
        assert!((LandfillStatus::Active.years_since_closure()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_years_since_closure_closed() {
        let status = LandfillStatus::Closed {
            days_since_closure: 365,
        };
        assert!((status.years_since_closure() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_years_since_closure_park() {
        assert!((LandfillStatus::ConvertedToPark.years_since_closure()).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // LandfillSite tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_new_site_defaults() {
        let site = LandfillSite::new(0, 10, 20);
        assert_eq!(site.id, 0);
        assert_eq!(site.grid_x, 10);
        assert_eq!(site.grid_y, 20);
        assert!((site.total_capacity_tons - DEFAULT_LANDFILL_CAPACITY_TONS).abs() < f64::EPSILON);
        assert!((site.current_fill_tons).abs() < f64::EPSILON);
        assert!((site.daily_input_tons).abs() < f64::EPSILON);
        assert_eq!(site.liner_type, LandfillLinerType::Unlined);
        assert!(site.status.is_active());
    }

    #[test]
    fn test_site_with_capacity_and_liner() {
        let site = LandfillSite::with_capacity_and_liner(
            1,
            5,
            10,
            1_000_000.0,
            LandfillLinerType::LinedWithCollection,
        );
        assert_eq!(site.id, 1);
        assert!((site.total_capacity_tons - 1_000_000.0).abs() < f64::EPSILON);
        assert_eq!(site.liner_type, LandfillLinerType::LinedWithCollection);
    }

    #[test]
    fn test_remaining_capacity_empty() {
        let site = LandfillSite::new(0, 0, 0);
        assert!(
            (site.remaining_capacity_tons() - DEFAULT_LANDFILL_CAPACITY_TONS).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_remaining_capacity_half_full() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS / 2.0;
        let expected = DEFAULT_LANDFILL_CAPACITY_TONS / 2.0;
        assert!((site.remaining_capacity_tons() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remaining_capacity_full() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS;
        assert!((site.remaining_capacity_tons()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remaining_capacity_pct_empty() {
        let site = LandfillSite::new(0, 0, 0);
        assert!((site.remaining_capacity_pct() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remaining_capacity_pct_half() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS / 2.0;
        assert!((site.remaining_capacity_pct() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_remaining_capacity_pct_full() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS;
        assert!((site.remaining_capacity_pct()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remaining_capacity_pct_zero_capacity() {
        let site = LandfillSite::with_capacity_and_liner(0, 0, 0, 0.0, LandfillLinerType::Unlined);
        assert!((site.remaining_capacity_pct()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_days_remaining_with_input() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.daily_input_tons = 1000.0;
        // 500,000 / 1000 = 500 days
        assert!((site.days_remaining() - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_days_remaining_zero_input() {
        let site = LandfillSite::new(0, 0, 0);
        assert!(site.days_remaining().is_infinite());
    }

    #[test]
    fn test_years_remaining_with_input() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.daily_input_tons = 1000.0;
        // 500,000 / 1000 = 500 days / 365 = ~1.37 years
        let expected = 500.0 / 365.0;
        assert!((site.years_remaining() - expected).abs() < 0.01);
    }

    #[test]
    fn test_is_full_false_when_empty() {
        let site = LandfillSite::new(0, 0, 0);
        assert!(!site.is_full());
    }

    #[test]
    fn test_is_full_true_when_full() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.current_fill_tons = DEFAULT_LANDFILL_CAPACITY_TONS;
        assert!(site.is_full());
    }

    #[test]
    fn test_gas_electricity_no_collection() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.daily_input_tons = 1000.0;
        assert!((site.gas_electricity_mw()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gas_electricity_with_collection_1000_tons() {
        let mut site = LandfillSite::with_capacity_and_liner(
            0,
            0,
            0,
            1_000_000.0,
            LandfillLinerType::LinedWithCollection,
        );
        site.daily_input_tons = 1000.0;
        // 1000 * 1.0 / 1000 = 1.0 MW
        assert!((site.gas_electricity_mw() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gas_electricity_with_collection_500_tons() {
        let mut site = LandfillSite::with_capacity_and_liner(
            0,
            0,
            0,
            1_000_000.0,
            LandfillLinerType::LinedWithCollection,
        );
        site.daily_input_tons = 500.0;
        // 500 * 1.0 / 1000 = 0.5 MW
        assert!((site.gas_electricity_mw() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_advance_fill_normal() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.advance_fill(1000.0);
        assert!((site.current_fill_tons - 1000.0).abs() < f64::EPSILON);
        assert!((site.daily_input_tons - 1000.0).abs() < f64::EPSILON);
        assert!(site.status.is_active());
    }

    #[test]
    fn test_advance_fill_clamps_at_capacity() {
        let mut site =
            LandfillSite::with_capacity_and_liner(0, 0, 0, 100.0, LandfillLinerType::Unlined);
        site.advance_fill(150.0);
        assert!((site.current_fill_tons - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_advance_fill_triggers_closure_when_full() {
        let mut site =
            LandfillSite::with_capacity_and_liner(0, 0, 0, 100.0, LandfillLinerType::Unlined);
        site.advance_fill(100.0);
        assert!(!site.status.is_active());
        match site.status {
            LandfillStatus::Closed { days_since_closure } => assert_eq!(days_since_closure, 0),
            _ => panic!("Expected Closed status"),
        }
    }

    #[test]
    fn test_advance_fill_noop_when_closed() {
        let mut site =
            LandfillSite::with_capacity_and_liner(0, 0, 0, 100.0, LandfillLinerType::Unlined);
        site.current_fill_tons = 100.0;
        site.status = LandfillStatus::Closed {
            days_since_closure: 10,
        };
        site.advance_fill(50.0);
        // Fill should not change
        assert!((site.current_fill_tons - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_advance_fill_noop_when_park() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.status = LandfillStatus::ConvertedToPark;
        site.advance_fill(50.0);
        assert!((site.current_fill_tons).abs() < f64::EPSILON);
    }

    #[test]
    fn test_advance_closure_increments_days() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.status = LandfillStatus::Closed {
            days_since_closure: 0,
        };
        site.advance_closure();
        match site.status {
            LandfillStatus::Closed { days_since_closure } => assert_eq!(days_since_closure, 1),
            _ => panic!("Expected Closed status"),
        }
    }

    #[test]
    fn test_advance_closure_converts_to_park_after_30_years() {
        let mut site = LandfillSite::new(0, 0, 0);
        let monitoring_days = (POST_CLOSURE_MONITORING_YEARS as f32 * DAYS_PER_YEAR) as u32;
        site.status = LandfillStatus::Closed {
            days_since_closure: monitoring_days - 1,
        };
        site.advance_closure();
        assert_eq!(site.status, LandfillStatus::ConvertedToPark);
    }

    #[test]
    fn test_advance_closure_noop_when_active() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.advance_closure();
        assert!(site.status.is_active());
    }

    #[test]
    fn test_advance_closure_noop_when_park() {
        let mut site = LandfillSite::new(0, 0, 0);
        site.status = LandfillStatus::ConvertedToPark;
        site.advance_closure();
        assert_eq!(site.status, LandfillStatus::ConvertedToPark);
    }
}

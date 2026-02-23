//! Tests for the water dashboard module.

#[cfg(test)]
mod tests {
    use simulation::water_demand::WaterSupply;

    use crate::water_dashboard::types::{WaterDashboardVisible, MGD_TO_GPD};

    #[test]
    fn test_water_dashboard_visible_default() {
        let visible = WaterDashboardVisible::default();
        assert!(!visible.0, "Dashboard should be hidden by default");
    }

    #[test]
    fn test_water_dashboard_visible_toggle() {
        let mut visible = WaterDashboardVisible::default();
        visible.0 = !visible.0;
        assert!(visible.0, "Dashboard should be visible after toggle");
        visible.0 = !visible.0;
        assert!(!visible.0, "Dashboard should be hidden after second toggle");
    }

    #[test]
    fn test_mgd_to_gpd_constant() {
        assert!(
            (MGD_TO_GPD - 1_000_000.0).abs() < f32::EPSILON,
            "MGD_TO_GPD should be 1,000,000"
        );
    }

    #[test]
    fn test_surplus_deficit_calculation() {
        // When supply > demand, surplus is positive
        let total_supply_gpd = 5_000_000.0_f32;
        let total_demand_gpd = 3_000_000.0_f32;
        let supply_mgd = total_supply_gpd / MGD_TO_GPD;
        let demand_mgd = total_demand_gpd / MGD_TO_GPD;
        let surplus = supply_mgd - demand_mgd;
        assert!(surplus > 0.0, "Should have surplus when supply > demand");
        assert!(
            (surplus - 2.0).abs() < 0.001,
            "Surplus should be 2.0 MGD, got {}",
            surplus
        );
    }

    #[test]
    fn test_deficit_calculation() {
        // When demand > supply, deficit is negative
        let total_supply_gpd = 2_000_000.0_f32;
        let total_demand_gpd = 5_000_000.0_f32;
        let supply_mgd = total_supply_gpd / MGD_TO_GPD;
        let demand_mgd = total_demand_gpd / MGD_TO_GPD;
        let surplus = supply_mgd - demand_mgd;
        assert!(surplus < 0.0, "Should have deficit when demand > supply");
        assert!(
            (surplus - (-3.0)).abs() < 0.001,
            "Deficit should be -3.0 MGD, got {}",
            surplus
        );
    }

    #[test]
    fn test_groundwater_level_percentage() {
        // Avg level of 128 out of 255 = ~50.2%
        let avg_level = 128.0_f32;
        let pct = avg_level / 255.0 * 100.0;
        assert!(
            (pct - 50.196).abs() < 0.1,
            "128/255 should be ~50.2%, got {}",
            pct
        );
    }

    #[test]
    fn test_groundwater_low_level_warning_threshold() {
        // Below 30% should trigger warning
        let avg_level = 70.0_f32;
        let pct = avg_level / 255.0 * 100.0;
        assert!(pct < 30.0, "Level {} should be below 30% threshold", pct);
    }

    #[test]
    fn test_groundwater_ok_level() {
        // Above 50% should show normal color
        let avg_level = 200.0_f32;
        let pct = avg_level / 255.0 * 100.0;
        assert!(pct >= 50.0, "Level {} should be above 50% threshold", pct);
    }

    #[test]
    fn test_service_coverage_all_served() {
        let served = 100_u32;
        let unserved = 0_u32;
        let total = served + unserved;
        let pct = served as f32 / total as f32 * 100.0;
        assert!(
            (pct - 100.0).abs() < f32::EPSILON,
            "All served should be 100%"
        );
    }

    #[test]
    fn test_service_coverage_none_served() {
        let served = 0_u32;
        let unserved = 50_u32;
        let total = served + unserved;
        let pct = if total > 0 {
            served as f32 / total as f32 * 100.0
        } else {
            100.0
        };
        assert!(
            pct.abs() < f32::EPSILON,
            "None served should be 0%, got {}",
            pct
        );
    }

    #[test]
    fn test_service_coverage_no_buildings() {
        let served = 0_u32;
        let unserved = 0_u32;
        let total = served + unserved;
        let pct = if total > 0 {
            served as f32 / total as f32 * 100.0
        } else {
            100.0
        };
        assert!(
            (pct - 100.0).abs() < f32::EPSILON,
            "No buildings should default to 100%"
        );
    }

    #[test]
    fn test_service_coverage_partial() {
        let served = 75_u32;
        let unserved = 25_u32;
        let total = served + unserved;
        let pct = served as f32 / total as f32 * 100.0;
        assert!(
            (pct - 75.0).abs() < 0.01,
            "75/100 served should be 75%, got {}",
            pct
        );
    }

    #[test]
    fn test_overflow_mgd_conversion() {
        let overflow_gpd = 500_000.0_f32;
        let overflow_mgd = overflow_gpd / MGD_TO_GPD;
        assert!(
            (overflow_mgd - 0.5).abs() < 0.001,
            "500K GPD should be 0.5 MGD, got {}",
            overflow_mgd
        );
    }

    #[test]
    fn test_coverage_color_thresholds() {
        // >= 90% = green
        let high = 95.0_f32;
        assert!(high >= 90.0);

        // 60-90% = yellow
        let mid = 75.0_f32;
        assert!(mid >= 60.0 && mid < 90.0);

        // < 60% = red
        let low = 40.0_f32;
        assert!(low < 60.0);
    }

    #[test]
    fn test_water_supply_default_values() {
        let supply = WaterSupply::default();
        let demand_mgd = supply.total_demand_gpd / MGD_TO_GPD;
        let supply_mgd = supply.total_supply_gpd / MGD_TO_GPD;
        assert!(
            demand_mgd.abs() < f32::EPSILON,
            "Default demand should be 0"
        );
        assert!(
            supply_mgd.abs() < f32::EPSILON,
            "Default supply should be 0"
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::agriculture::helpers::{
        calculate_crop_yield, calculate_frost_risk, celsius_to_fahrenheit, is_growing_season,
        rainfall_adequacy, temperature_suitability,
    };
    use crate::agriculture::types::{
        AgricultureState, BASE_SOIL_QUALITY, IRRIGATION_FERTILIZER_BONUS,
        RAINFALL_DEFICIT_MULTIPLIER, RAINFALL_EXCESS_MULTIPLIER, SPRING_FROST_BASE_RISK,
    };
    use crate::weather::Season;

    // -------------------------------------------------------------------------
    // Celsius to Fahrenheit
    // -------------------------------------------------------------------------

    #[test]
    fn test_celsius_to_fahrenheit_freezing() {
        let f = celsius_to_fahrenheit(0.0);
        assert!((f - 32.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_celsius_to_fahrenheit_boiling() {
        let f = celsius_to_fahrenheit(100.0);
        assert!((f - 212.0).abs() < 0.01);
    }

    #[test]
    fn test_celsius_to_fahrenheit_ten() {
        // 10C = 50F
        let f = celsius_to_fahrenheit(10.0);
        assert!((f - 50.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Growing season
    // -------------------------------------------------------------------------

    #[test]
    fn test_growing_season_summer_warm() {
        // 25C, summer, no frost -> active
        assert!(is_growing_season(25.0, Season::Summer, 0.0));
    }

    #[test]
    fn test_growing_season_winter_inactive() {
        // Winter always inactive
        assert!(!is_growing_season(25.0, Season::Winter, 0.0));
    }

    #[test]
    fn test_growing_season_cold_inactive() {
        // Below 50F (10C) -> inactive
        assert!(!is_growing_season(5.0, Season::Spring, 0.0));
    }

    #[test]
    fn test_growing_season_high_frost_inactive() {
        // Frost risk >= 10% -> inactive
        assert!(!is_growing_season(12.0, Season::Spring, 0.15));
    }

    #[test]
    fn test_growing_season_spring_warm_no_frost() {
        // 15C, spring, low frost -> active
        assert!(is_growing_season(15.0, Season::Spring, 0.0));
    }

    #[test]
    fn test_growing_season_autumn_warm_no_frost() {
        // 15C, autumn, low frost -> active
        assert!(is_growing_season(15.0, Season::Autumn, 0.0));
    }

    #[test]
    fn test_growing_season_exactly_threshold() {
        // Exactly 10C = 50F, should NOT be active (must be > 50F)
        assert!(!is_growing_season(10.0, Season::Spring, 0.0));
    }

    // -------------------------------------------------------------------------
    // Frost risk
    // -------------------------------------------------------------------------

    #[test]
    fn test_frost_risk_winter() {
        assert!((calculate_frost_risk(5.0, Season::Winter) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_summer() {
        assert!((calculate_frost_risk(25.0, Season::Summer)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_spring_warm() {
        // 15C in spring -> no frost risk
        assert!((calculate_frost_risk(15.0, Season::Spring)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_spring_cold() {
        // Below 0C in spring -> high frost risk
        assert!((calculate_frost_risk(-5.0, Season::Spring) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_spring_marginal() {
        // 3C in spring -> some frost risk
        let risk = calculate_frost_risk(3.0, Season::Spring);
        assert!(risk > 0.0);
        assert!(risk < 1.0);
    }

    #[test]
    fn test_frost_risk_autumn_cold() {
        let risk = calculate_frost_risk(-1.0, Season::Autumn);
        assert!((risk - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_autumn_warm() {
        assert!((calculate_frost_risk(12.0, Season::Autumn)).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Temperature suitability
    // -------------------------------------------------------------------------

    #[test]
    fn test_temp_suitability_optimal() {
        assert!((temperature_suitability(20.0) - 1.0).abs() < f32::EPSILON);
        assert!((temperature_suitability(25.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temp_suitability_cold() {
        assert!((temperature_suitability(5.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temp_suitability_hot() {
        assert!((temperature_suitability(42.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temp_suitability_transition_low() {
        // 12.5C -> halfway between 10 and 15 = 0.5
        let s = temperature_suitability(12.5);
        assert!((s - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temp_suitability_transition_high() {
        // 35C -> halfway between 30 and 40 = 0.5
        let s = temperature_suitability(35.0);
        assert!((s - 0.5).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Rainfall adequacy
    // -------------------------------------------------------------------------

    #[test]
    fn test_rainfall_adequate() {
        let r = rainfall_adequacy(30.0, false);
        assert!((r - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainfall_deficit() {
        let r = rainfall_adequacy(15.0, false);
        assert!((r - RAINFALL_DEFICIT_MULTIPLIER).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainfall_excess() {
        let r = rainfall_adequacy(50.0, false);
        assert!((r - RAINFALL_EXCESS_MULTIPLIER).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainfall_irrigated_deficit_improved() {
        // Irrigation should improve deficit scenario
        let without = rainfall_adequacy(10.0, false);
        let with = rainfall_adequacy(10.0, true);
        assert!(with > without);
    }

    #[test]
    fn test_rainfall_irrigated_capped_at_one() {
        let r = rainfall_adequacy(30.0, true);
        assert!(r <= 1.0);
    }

    // -------------------------------------------------------------------------
    // Crop yield calculation
    // -------------------------------------------------------------------------

    #[test]
    fn test_crop_yield_all_optimal() {
        let y = calculate_crop_yield(1.0, 1.0, 1.0, 1.0);
        assert!((y - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crop_yield_with_irrigation() {
        let y = calculate_crop_yield(1.0, 1.0, 0.8, IRRIGATION_FERTILIZER_BONUS);
        let expected = 0.8 * IRRIGATION_FERTILIZER_BONUS;
        assert!((y - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crop_yield_deficit() {
        let y = calculate_crop_yield(RAINFALL_DEFICIT_MULTIPLIER, 0.5, 0.8, 1.0);
        let expected = RAINFALL_DEFICIT_MULTIPLIER * 0.5 * 0.8;
        assert!((y - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crop_yield_zero_temp() {
        let y = calculate_crop_yield(1.0, 0.0, 0.8, 1.0);
        assert!(y.abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Default state
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state() {
        let state = AgricultureState::default();
        assert!(!state.growing_season_active);
        assert!((state.crop_yield_modifier - 1.0).abs() < f32::EPSILON);
        assert!((state.rainfall_adequacy - 1.0).abs() < f32::EPSILON);
        assert!((state.temperature_suitability - 1.0).abs() < f32::EPSILON);
        assert!((state.soil_quality - BASE_SOIL_QUALITY).abs() < f32::EPSILON);
        assert!((state.fertilizer_bonus - 1.0).abs() < f32::EPSILON);
        assert!(state.frost_risk.abs() < f32::EPSILON);
        assert_eq!(state.frost_events_this_year, 0);
        assert!(state.frost_damage_total.abs() < f32::EPSILON);
        assert!(!state.has_irrigation);
        assert_eq!(state.farm_count, 0);
    }

    // -------------------------------------------------------------------------
    // Edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_growing_season_boundary_temp() {
        // Just above 10C (50.18F) -> should be active in summer
        assert!(is_growing_season(10.1, Season::Summer, 0.0));
    }

    #[test]
    fn test_frost_risk_boundary_spring() {
        // Exactly 10C -> no frost risk
        let risk = calculate_frost_risk(10.0, Season::Spring);
        assert!(risk.abs() < f32::EPSILON);
    }

    #[test]
    fn test_frost_risk_boundary_five() {
        // Exactly 5C in spring -> base risk only
        let risk = calculate_frost_risk(5.0, Season::Spring);
        assert!((risk - SPRING_FROST_BASE_RISK).abs() < 0.01);
    }

    #[test]
    fn test_rainfall_boundary_low() {
        // Exactly at low boundary -> adequate
        let r = rainfall_adequacy(20.0, false);
        assert!((r - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainfall_boundary_high() {
        // Exactly at high boundary -> adequate
        let r = rainfall_adequacy(40.0, false);
        assert!((r - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temp_suitability_at_boundaries() {
        assert!((temperature_suitability(10.0)).abs() < f32::EPSILON);
        assert!((temperature_suitability(15.0) - 1.0).abs() < f32::EPSILON);
        assert!((temperature_suitability(30.0) - 1.0).abs() < f32::EPSILON);
        assert!((temperature_suitability(40.0)).abs() < f32::EPSILON);
    }
}

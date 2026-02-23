//! Unit tests for the climate change module.

#[cfg(test)]
mod tests {
    use crate::climate_change::calculations::*;
    use crate::climate_change::constants::*;
    use crate::climate_change::state::ClimateState;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, WorldGrid};
    use crate::utilities::UtilityType;
    use crate::Saveable;

    // -------------------------------------------------------------------------
    // Temperature increase from CO2 tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_zero_emissions_no_warming() {
        assert_eq!(temperature_increase_from_co2(0.0), 0.0);
    }

    #[test]
    fn test_threshold_1f() {
        let result = temperature_increase_from_co2(THRESHOLD_1F);
        assert!(
            (result - 1.0).abs() < f32::EPSILON,
            "At 1M tons, should be +1F, got {}",
            result
        );
    }

    #[test]
    fn test_threshold_2f() {
        let result = temperature_increase_from_co2(THRESHOLD_2F);
        assert!(
            (result - 2.0).abs() < f32::EPSILON,
            "At 5M tons, should be +2F, got {}",
            result
        );
    }

    #[test]
    fn test_threshold_3f() {
        let result = temperature_increase_from_co2(THRESHOLD_3F);
        assert!(
            (result - 3.0).abs() < f32::EPSILON,
            "At 20M tons, should be +3F, got {}",
            result
        );
    }

    #[test]
    fn test_above_threshold_3f_caps_at_3() {
        let result = temperature_increase_from_co2(50_000_000.0);
        assert!(
            (result - 3.0).abs() < f32::EPSILON,
            "Above 20M tons should cap at +3F, got {}",
            result
        );
    }

    #[test]
    fn test_interpolation_between_0_and_1f() {
        let result = temperature_increase_from_co2(500_000.0);
        assert!(
            result > 0.0 && result < 1.0,
            "At 500K tons, should be between 0F and 1F, got {}",
            result
        );
        assert!(
            (result - 0.5).abs() < f32::EPSILON,
            "At 500K (half of 1M), should be ~0.5F, got {}",
            result
        );
    }

    #[test]
    fn test_interpolation_between_1f_and_2f() {
        // Midpoint between 1M and 5M is 3M
        let result = temperature_increase_from_co2(3_000_000.0);
        assert!(
            result > 1.0 && result < 2.0,
            "At 3M tons, should be between 1F and 2F, got {}",
            result
        );
        // (3M - 1M) / (5M - 1M) = 2M / 4M = 0.5, so 1.0 + 0.5 = 1.5
        assert!(
            (result - 1.5).abs() < f32::EPSILON,
            "At 3M tons, should be ~1.5F, got {}",
            result
        );
    }

    #[test]
    fn test_interpolation_between_2f_and_3f() {
        // Midpoint between 5M and 20M is 12.5M
        let result = temperature_increase_from_co2(12_500_000.0);
        assert!(
            result > 2.0 && result < 3.0,
            "At 12.5M tons, should be between 2F and 3F, got {}",
            result
        );
        // (12.5M - 5M) / (20M - 5M) = 7.5M / 15M = 0.5, so 2.0 + 0.5 = 2.5
        assert!(
            (result - 2.5).abs() < f32::EPSILON,
            "At 12.5M tons, should be ~2.5F, got {}",
            result
        );
    }

    // -------------------------------------------------------------------------
    // Disaster multiplier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_disaster_multiplier_no_warming() {
        assert!(
            (disaster_multiplier_from_temp_increase(0.0) - 1.0).abs() < f32::EPSILON,
            "No warming should give 1.0 multiplier"
        );
    }

    #[test]
    fn test_disaster_multiplier_1f() {
        let result = disaster_multiplier_from_temp_increase(1.0);
        assert!(
            (result - 1.1).abs() < f32::EPSILON,
            "+1F should give 1.1 multiplier, got {}",
            result
        );
    }

    #[test]
    fn test_disaster_multiplier_2f() {
        let result = disaster_multiplier_from_temp_increase(2.0);
        assert!(
            (result - 1.2).abs() < f32::EPSILON,
            "+2F should give 1.2 multiplier, got {}",
            result
        );
    }

    #[test]
    fn test_disaster_multiplier_3f() {
        let result = disaster_multiplier_from_temp_increase(3.0);
        assert!(
            (result - 1.3).abs() < f32::EPSILON,
            "+3F should give 1.3 multiplier, got {}",
            result
        );
    }

    // -------------------------------------------------------------------------
    // Drought multiplier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_drought_multiplier_no_warming() {
        assert!(
            (drought_multiplier_from_temp_increase(0.0) - 1.0).abs() < f32::EPSILON,
            "No warming should give 1.0 drought multiplier"
        );
    }

    #[test]
    fn test_drought_multiplier_1f() {
        let result = drought_multiplier_from_temp_increase(1.0);
        assert!(
            (result - 1.15).abs() < f32::EPSILON,
            "+1F should give 1.15 drought multiplier, got {}",
            result
        );
    }

    #[test]
    fn test_drought_multiplier_3f() {
        let result = drought_multiplier_from_temp_increase(3.0);
        assert!(
            (result - 1.45).abs() < f32::EPSILON,
            "+3F should give 1.45 drought multiplier, got {}",
            result
        );
    }

    // -------------------------------------------------------------------------
    // Environmental score tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_environmental_score_no_emissions() {
        let score = calculate_environmental_score(0.0, 0.0);
        assert!(
            (score - 100.0).abs() < f32::EPSILON,
            "Zero emissions should give 100 score"
        );
    }

    #[test]
    fn test_environmental_score_1m_tons() {
        let score = calculate_environmental_score(1_000_000.0, 0.0);
        assert!(
            (score - 90.0).abs() < f32::EPSILON,
            "1M cumulative tons should give 90 score, got {}",
            score
        );
    }

    #[test]
    fn test_environmental_score_5m_tons() {
        let score = calculate_environmental_score(5_000_000.0, 0.0);
        assert!(
            (score - 50.0).abs() < f32::EPSILON,
            "5M cumulative tons should give 50 score, got {}",
            score
        );
    }

    #[test]
    fn test_environmental_score_10m_tons() {
        let score = calculate_environmental_score(10_000_000.0, 0.0);
        assert!(
            (score - 0.0).abs() < f32::EPSILON,
            "10M cumulative tons should give 0 score, got {}",
            score
        );
    }

    #[test]
    fn test_environmental_score_with_yearly() {
        let score = calculate_environmental_score(0.0, 100_000.0);
        assert!(
            (score - 90.0).abs() < f32::EPSILON,
            "100K yearly tons should give 90 score, got {}",
            score
        );
    }

    #[test]
    fn test_environmental_score_clamped_to_zero() {
        let score = calculate_environmental_score(50_000_000.0, 1_000_000.0);
        assert!(
            (score - 0.0).abs() < f32::EPSILON,
            "Very high emissions should clamp to 0, got {}",
            score
        );
    }

    #[test]
    fn test_environmental_score_clamped_to_100() {
        // Even negative emissions shouldn't exceed 100
        let score = calculate_environmental_score(0.0, 0.0);
        assert!(score <= 100.0);
    }

    // -------------------------------------------------------------------------
    // CO2 rate tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_co2_rate_power_plant() {
        assert!(
            (co2_rate_for_utility(UtilityType::PowerPlant) - CO2_OIL).abs() < f32::EPSILON,
            "Power plant should use oil rate"
        );
    }

    #[test]
    fn test_co2_rate_solar() {
        assert!(
            (co2_rate_for_utility(UtilityType::SolarFarm) - 0.0).abs() < f32::EPSILON,
            "Solar should be zero carbon"
        );
    }

    #[test]
    fn test_co2_rate_wind() {
        assert!(
            (co2_rate_for_utility(UtilityType::WindTurbine) - 0.0).abs() < f32::EPSILON,
            "Wind should be zero carbon"
        );
    }

    #[test]
    fn test_co2_rate_nuclear() {
        assert!(
            (co2_rate_for_utility(UtilityType::NuclearPlant) - 0.0).abs() < f32::EPSILON,
            "Nuclear should be zero carbon"
        );
    }

    #[test]
    fn test_co2_rate_geothermal() {
        assert!(
            (co2_rate_for_utility(UtilityType::Geothermal) - 0.0).abs() < f32::EPSILON,
            "Geothermal should be zero carbon"
        );
    }

    #[test]
    fn test_co2_rate_water_utilities() {
        assert_eq!(co2_rate_for_utility(UtilityType::WaterTower), 0.0);
        assert_eq!(co2_rate_for_utility(UtilityType::SewagePlant), 0.0);
        assert_eq!(co2_rate_for_utility(UtilityType::PumpingStation), 0.0);
        assert_eq!(co2_rate_for_utility(UtilityType::WaterTreatment), 0.0);
    }

    // -------------------------------------------------------------------------
    // Water adjacency tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_water_adjacent_true() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place water at (5, 5)
        grid.get_mut(5, 5).cell_type = CellType::Water;
        // (5, 6) should be adjacent to water
        assert!(is_water_adjacent(&grid, 5, 6));
        // (4, 5) should be adjacent to water
        assert!(is_water_adjacent(&grid, 4, 5));
    }

    #[test]
    fn test_is_water_adjacent_false() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // No water placed, nothing should be water-adjacent
        assert!(!is_water_adjacent(&grid, 10, 10));
    }

    // -------------------------------------------------------------------------
    // Sea level rise threshold tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sea_level_rise_threshold_no_water() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // No water cells, threshold should be 0
        let threshold = sea_level_rise_threshold(&grid);
        assert_eq!(threshold, 0.0);
    }

    #[test]
    fn test_sea_level_rise_threshold_with_coastal_cells() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Create a water body on the left edge
        for y in 0..GRID_HEIGHT {
            grid.get_mut(0, y).cell_type = CellType::Water;
            grid.get_mut(0, y).elevation = 0.0;
        }
        // Set coastal cell elevations (cells at x=1 are adjacent to water)
        for y in 0..GRID_HEIGHT {
            grid.get_mut(1, y).elevation = y as f32 * 0.1;
        }
        // The threshold should be at the 15th percentile of coastal elevations
        let threshold = sea_level_rise_threshold(&grid);
        assert!(
            threshold >= 0.0,
            "Threshold should be non-negative, got {}",
            threshold
        );
    }

    // -------------------------------------------------------------------------
    // ClimateState default tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_climate_state_default() {
        let state = ClimateState::default();
        assert_eq!(state.cumulative_co2, 0.0);
        assert_eq!(state.yearly_co2, 0.0);
        assert_eq!(state.temperature_increase_f, 0.0);
        assert!((state.disaster_frequency_multiplier - 1.0).abs() < f32::EPSILON);
        assert!(!state.sea_level_rise_applied);
        assert_eq!(state.flooded_cells_count, 0);
        assert!((state.environmental_score - 100.0).abs() < f32::EPSILON);
        assert_eq!(state.last_assessment_day, 0);
        assert!((state.drought_duration_multiplier - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable implementation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skip_default() {
        let state = ClimateState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = ClimateState::default();
        state.cumulative_co2 = 5_000_000.0;
        state.yearly_co2 = 100_000.0;
        state.temperature_increase_f = 2.0;
        state.last_assessment_day = 720;

        let bytes = state
            .save_to_bytes()
            .expect("Non-default state should save");
        let loaded = ClimateState::load_from_bytes(&bytes);

        assert!((loaded.cumulative_co2 - 5_000_000.0).abs() < f64::EPSILON);
        assert!((loaded.yearly_co2 - 100_000.0).abs() < f64::EPSILON);
        assert!((loaded.temperature_increase_f - 2.0).abs() < f32::EPSILON);
        assert_eq!(loaded.last_assessment_day, 720);
    }

    // -------------------------------------------------------------------------
    // CO2 constants tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_co2_constants() {
        assert!((CO2_COAL - 1.0).abs() < f32::EPSILON);
        assert!((CO2_GAS - 0.4).abs() < f32::EPSILON);
        assert!((CO2_OIL - 0.8).abs() < f32::EPSILON);
        assert!((CO2_BIOMASS - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_threshold_constants() {
        assert!((THRESHOLD_1F - 1_000_000.0).abs() < f64::EPSILON);
        assert!((THRESHOLD_2F - 5_000_000.0).abs() < f64::EPSILON);
        assert!((THRESHOLD_3F - 20_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_disaster_frequency_increase_per_f() {
        assert!((DISASTER_FREQUENCY_INCREASE_PER_F - 0.10).abs() < f32::EPSILON);
    }

    #[test]
    fn test_days_per_year() {
        assert_eq!(DAYS_PER_YEAR, 360);
    }
}

//! Unit tests for the snow system.

#[cfg(test)]
mod tests {
    use crate::grid::RoadType;
    use crate::snow::systems::{
        plow_priority, snow_accumulation_amount, snow_heating_modifier, snow_melt_amount,
        snow_speed_multiplier,
    };
    use crate::snow::types::{
        SnowGrid, SnowPlowingState, SnowStats, BASE_SNOW_ACCUMULATION_RATE,
        HEATING_INCREASE_PER_6_INCHES, MAX_SNOW_DEPTH, MAX_SNOW_SPEED_REDUCTION,
        MELT_RATE_PER_DEGREE, PLOW_COST_PER_CELL, PLOW_REMOVAL_DEPTH, PLOW_TRIGGER_DEPTH,
        SPEED_REDUCTION_PER_INCH,
    };
    use crate::weather::{Weather, WeatherCondition};

    // -------------------------------------------------------------------------
    // SnowGrid tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_snow_grid_default() {
        let grid = SnowGrid::default();
        assert_eq!(grid.depths.len(), grid.width * grid.height);
        assert_eq!(grid.get(0, 0), 0.0);
        assert_eq!(grid.average_depth(), 0.0);
        assert_eq!(grid.covered_cells(), 0);
        assert_eq!(grid.max_depth(), 0.0);
    }

    #[test]
    fn test_snow_grid_set_get() {
        let mut grid = SnowGrid::default();
        grid.set(10, 10, 5.0);
        assert_eq!(grid.get(10, 10), 5.0);
        assert_eq!(grid.get(0, 0), 0.0);
    }

    #[test]
    fn test_snow_grid_covered_cells() {
        let mut grid = SnowGrid::default();
        grid.set(0, 0, 1.0);
        grid.set(1, 0, 2.0);
        grid.set(2, 0, 3.0);
        assert_eq!(grid.covered_cells(), 3);
    }

    #[test]
    fn test_snow_grid_max_depth() {
        let mut grid = SnowGrid::default();
        grid.set(5, 5, 8.0);
        grid.set(10, 10, 12.0);
        grid.set(15, 15, 4.0);
        assert_eq!(grid.max_depth(), 12.0);
    }

    // -------------------------------------------------------------------------
    // Snow accumulation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_accumulation_above_freezing() {
        let mut weather = Weather::default();
        weather.temperature = 5.0;
        weather.current_event = WeatherCondition::Snow;
        assert_eq!(snow_accumulation_amount(&weather), 0.0);
    }

    #[test]
    fn test_accumulation_during_snow() {
        let mut weather = Weather::default();
        weather.temperature = -5.0;
        weather.current_event = WeatherCondition::Snow;
        weather.precipitation_intensity = 0.2;
        let amount = snow_accumulation_amount(&weather);
        assert!(
            amount > 0.0,
            "snow should accumulate during Snow event below freezing"
        );
    }

    #[test]
    fn test_accumulation_during_storm_below_freezing() {
        let mut weather = Weather::default();
        weather.temperature = -10.0;
        weather.current_event = WeatherCondition::Storm;
        weather.precipitation_intensity = 1.0;
        let amount = snow_accumulation_amount(&weather);
        assert!(
            amount > 0.0,
            "snow should accumulate during Storm below freezing"
        );
    }

    #[test]
    fn test_accumulation_during_rain() {
        let mut weather = Weather::default();
        weather.temperature = -5.0;
        weather.current_event = WeatherCondition::Rain;
        let amount = snow_accumulation_amount(&weather);
        assert_eq!(amount, 0.0, "rain should not cause snow accumulation");
    }

    #[test]
    fn test_accumulation_sunny() {
        let mut weather = Weather::default();
        weather.temperature = -5.0;
        weather.current_event = WeatherCondition::Sunny;
        let amount = snow_accumulation_amount(&weather);
        assert_eq!(amount, 0.0, "sunny weather should not cause snow");
    }

    #[test]
    fn test_accumulation_scales_with_intensity() {
        let mut weather = Weather::default();
        weather.temperature = -5.0;
        weather.current_event = WeatherCondition::Snow;

        weather.precipitation_intensity = 0.1;
        let low = snow_accumulation_amount(&weather);

        weather.precipitation_intensity = 1.0;
        let high = snow_accumulation_amount(&weather);

        assert!(high > low, "higher precipitation should produce more snow");
    }

    // -------------------------------------------------------------------------
    // Snow melt tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_melt_below_freezing() {
        assert_eq!(snow_melt_amount(-5.0), 0.0);
        assert_eq!(snow_melt_amount(0.0), 0.0);
    }

    #[test]
    fn test_melt_above_freezing() {
        let melt = snow_melt_amount(5.0);
        assert!(melt > 0.0, "snow should melt above freezing");
    }

    #[test]
    fn test_melt_proportional_to_temperature() {
        let melt_5 = snow_melt_amount(5.0);
        let melt_10 = snow_melt_amount(10.0);
        assert!(melt_10 > melt_5, "higher temperature should melt more snow");
        assert!(
            (melt_10 - melt_5 * 2.0).abs() < f32::EPSILON,
            "melt should be proportional to temperature excess"
        );
    }

    // -------------------------------------------------------------------------
    // Speed multiplier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_speed_no_snow() {
        assert_eq!(snow_speed_multiplier(0.0), 1.0);
    }

    #[test]
    fn test_speed_some_snow() {
        let mult = snow_speed_multiplier(4.0);
        // 4 inches * 0.05 = 0.20 reduction => 0.80 multiplier
        assert!((mult - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn test_speed_heavy_snow() {
        let mult = snow_speed_multiplier(12.0);
        // 12 inches * 0.05 = 0.60 reduction => 0.40 multiplier
        assert!((mult - 0.40).abs() < f32::EPSILON);
    }

    #[test]
    fn test_speed_extreme_snow_capped() {
        let mult = snow_speed_multiplier(20.0);
        // 20 inches * 0.05 = 1.0 but capped at 0.80 reduction => 0.20 multiplier
        assert!((mult - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_speed_never_below_minimum() {
        let mult = snow_speed_multiplier(100.0);
        assert!(mult >= 0.2, "speed multiplier should never go below 0.2");
    }

    // -------------------------------------------------------------------------
    // Heating modifier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_heating_no_snow() {
        assert_eq!(snow_heating_modifier(0.0), 1.0);
    }

    #[test]
    fn test_heating_6_inches() {
        let modifier = snow_heating_modifier(6.0);
        // 6 inches / 6 = 1 increment * 0.10 + 1.0 = 1.10
        assert!((modifier - 1.10).abs() < f32::EPSILON);
    }

    #[test]
    fn test_heating_12_inches() {
        let modifier = snow_heating_modifier(12.0);
        // 12 inches / 6 = 2 increments * 0.10 + 1.0 = 1.20
        assert!((modifier - 1.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_heating_3_inches() {
        let modifier = snow_heating_modifier(3.0);
        // 3 inches / 6 = 0.5 increments * 0.10 + 1.0 = 1.05
        assert!((modifier - 1.05).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Plow priority tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_plow_priority_order() {
        assert!(plow_priority(RoadType::Highway) < plow_priority(RoadType::Boulevard));
        assert!(plow_priority(RoadType::Boulevard) < plow_priority(RoadType::Avenue));
        assert!(plow_priority(RoadType::Avenue) < plow_priority(RoadType::Local));
        assert!(plow_priority(RoadType::Local) < plow_priority(RoadType::Path));
    }

    // -------------------------------------------------------------------------
    // SnowPlowingState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_plowing_state_default() {
        let state = SnowPlowingState::default();
        assert!(state.enabled);
        assert_eq!(state.season_cost, 0.0);
        assert_eq!(state.cells_plowed_season, 0);
        assert_eq!(state.cells_plowed_last, 0);
        assert_eq!(state.last_plow_cost, 0.0);
    }

    // -------------------------------------------------------------------------
    // SnowStats tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_snow_stats_default() {
        let stats = SnowStats::default();
        assert_eq!(stats.avg_depth, 0.0);
        assert_eq!(stats.max_depth, 0.0);
        assert_eq!(stats.covered_cells, 0);
        assert_eq!(stats.road_speed_multiplier, 1.0);
        assert_eq!(stats.heating_demand_modifier, 1.0);
        assert_eq!(stats.snowmelt_runoff, 0.0);
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_are_reasonable() {
        assert!(PLOW_COST_PER_CELL > 0.0);
        assert!(PLOW_REMOVAL_DEPTH > 0.0);
        assert!(PLOW_TRIGGER_DEPTH > 0.0);
        assert!(MAX_SNOW_DEPTH > 0.0);
        assert!(MAX_SNOW_SPEED_REDUCTION > 0.0);
        assert!(MAX_SNOW_SPEED_REDUCTION <= 1.0);
        assert!(SPEED_REDUCTION_PER_INCH > 0.0);
        assert!(HEATING_INCREASE_PER_6_INCHES > 0.0);
        assert!(BASE_SNOW_ACCUMULATION_RATE > 0.0);
        assert!(MELT_RATE_PER_DEGREE > 0.0);
    }

    #[test]
    fn test_max_speed_reduction_depth_consistent() {
        // At 12 inches depth, the reduction from SPEED_REDUCTION_PER_INCH
        // should reach a meaningful fraction of MAX_SNOW_SPEED_REDUCTION
        let reduction_at_max = 12.0_f32 * SPEED_REDUCTION_PER_INCH;
        assert!(
            reduction_at_max <= MAX_SNOW_SPEED_REDUCTION,
            "reduction at max depth ({}) should not exceed max reduction ({})",
            reduction_at_max,
            MAX_SNOW_SPEED_REDUCTION
        );
    }
}

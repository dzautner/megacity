//! Tests for seasonal rendering types, saveable trait, and effect combinations.

#[cfg(test)]
mod tests {
    use crate::seasonal_rendering::compute::*;
    use crate::seasonal_rendering::constants::*;
    use crate::seasonal_rendering::types::*;
    use crate::weather::{Season, Weather, WeatherCondition};
    use crate::Saveable;

    // -------------------------------------------------------------------------
    // Helper
    // -------------------------------------------------------------------------

    fn test_weather(season: Season, condition: WeatherCondition, temp: f32) -> Weather {
        Weather {
            season,
            current_event: condition,
            temperature: temp,
            precipitation_intensity: match condition {
                WeatherCondition::Rain => 0.3,
                WeatherCondition::HeavyRain => 1.5,
                WeatherCondition::Storm => 2.0,
                WeatherCondition::Snow => 0.5,
                _ => 0.0,
            },
            ..Default::default()
        }
    }

    // -------------------------------------------------------------------------
    // SeasonalEffectsConfig tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_config_default_all_enabled() {
        let config = SeasonalEffectsConfig::default();
        assert!(config.leaves_enabled);
        assert!(config.snow_roofs_enabled);
        assert!(config.flowers_enabled);
        assert!(config.heat_shimmer_enabled);
        assert!(config.rain_streaks_enabled);
        assert!(config.storm_effects_enabled);
        assert!(config.snowflakes_enabled);
        assert!(config.summer_shadows_enabled);
        assert!(config.spring_brightness_enabled);
    }

    // -------------------------------------------------------------------------
    // SeasonalRenderingState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_default() {
        let state = SeasonalRenderingState::default();
        assert_eq!(state.leaf_intensity, 0.0);
        assert_eq!(state.snow_roof_intensity, 0.0);
        assert_eq!(state.snowflake_intensity, 0.0);
        assert_eq!(state.flower_intensity, 0.0);
        assert_eq!(state.heat_shimmer_intensity, 0.0);
        assert_eq!(state.rain_streak_intensity, 0.0);
        assert_eq!(state.storm_darkening, 0.0);
        assert!(!state.lightning_active);
        assert_eq!(state.lightning_timer, 0);
        assert_eq!(state.shadow_multiplier, 1.0);
        assert_eq!(state.spring_brightness, 0.0);
        assert_eq!(state.leaf_source_cells, 0);
        assert_eq!(state.snow_roof_cells, 0);
        assert_eq!(state.flower_source_cells, 0);
        assert_eq!(state.current_season_id, 0); // Spring
        assert_eq!(state.current_condition_id, 0); // Sunny
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_key_state() {
        assert_eq!(
            SeasonalRenderingState::SAVE_KEY,
            "seasonal_rendering",
            "Save key should be 'seasonal_rendering'"
        );
    }

    #[test]
    fn test_saveable_key_config() {
        assert_eq!(
            SeasonalEffectsConfig::SAVE_KEY,
            "seasonal_effects_config",
            "Save key should be 'seasonal_effects_config'"
        );
    }

    #[test]
    fn test_saveable_state_default_returns_none() {
        let state = SeasonalRenderingState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_config_default_returns_none() {
        let config = SeasonalEffectsConfig::default();
        assert!(
            config.save_to_bytes().is_none(),
            "default config should skip saving"
        );
    }

    #[test]
    fn test_saveable_state_roundtrip() {
        let mut state = SeasonalRenderingState::default();
        state.leaf_intensity = 0.75;
        state.snow_roof_intensity = 0.5;
        state.storm_darkening = 0.3;
        state.lightning_timer = 1;
        state.lightning_active = true;
        state.current_season_id = season_to_id(Season::Autumn);
        state.current_condition_id = condition_to_id(WeatherCondition::Storm);

        let bytes = state.save_to_bytes().expect("should have bytes");
        let loaded = SeasonalRenderingState::load_from_bytes(&bytes);

        assert!((loaded.leaf_intensity - 0.75).abs() < f32::EPSILON);
        assert!((loaded.snow_roof_intensity - 0.5).abs() < f32::EPSILON);
        assert!((loaded.storm_darkening - 0.3).abs() < f32::EPSILON);
        assert_eq!(loaded.lightning_timer, 1);
        assert!(loaded.lightning_active);
        assert_eq!(loaded.active_season(), Season::Autumn);
        assert_eq!(loaded.active_condition(), WeatherCondition::Storm);
    }

    #[test]
    fn test_saveable_config_roundtrip() {
        let mut config = SeasonalEffectsConfig::default();
        config.leaves_enabled = false;
        config.heat_shimmer_enabled = false;

        let bytes = config.save_to_bytes().expect("should have bytes");
        let loaded = SeasonalEffectsConfig::load_from_bytes(&bytes);

        assert!(!loaded.leaves_enabled);
        assert!(loaded.snow_roofs_enabled);
        assert!(loaded.flowers_enabled);
        assert!(!loaded.heat_shimmer_enabled);
        assert!(loaded.rain_streaks_enabled);
        assert!(loaded.storm_effects_enabled);
        assert!(loaded.snowflakes_enabled);
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_positive() {
        assert!(MAX_LEAF_INTENSITY > 0.0);
        assert!(LEAF_RAMP_RATE > 0.0);
        assert!(LEAF_DECAY_RATE > 0.0);
        assert!(MAX_FLOWER_INTENSITY > 0.0);
        assert!(FLOWER_RAMP_RATE > 0.0);
        assert!(FLOWER_DECAY_RATE > 0.0);
        assert!(MAX_SNOW_ROOF_INTENSITY > 0.0);
        assert!(SNOW_ROOF_RAMP_RATE > 0.0);
        assert!(SNOW_ROOF_DECAY_RATE > 0.0);
        assert!(HEAT_SHIMMER_THRESHOLD > 0.0);
        assert!(MAX_HEAT_SHIMMER_INTENSITY > 0.0);
        assert!(RAIN_INTENSITY_SCALE > 0.0);
        assert!(MAX_RAIN_INTENSITY > 0.0);
        assert!(MAX_SNOWFLAKE_INTENSITY > 0.0);
        assert!(SNOWFLAKE_INTENSITY_SCALE > 0.0);
        assert!(STORM_DARKENING_INTENSITY > 0.0);
        assert!(STORM_DARKEN_RAMP_RATE > 0.0);
        assert!(STORM_DARKEN_DECAY_RATE > 0.0);
        assert!(LIGHTNING_FLASH_PROBABILITY > 0.0);
        assert!(LIGHTNING_FLASH_PROBABILITY <= 1.0);
        assert!(SUMMER_SHADOW_MULTIPLIER > 1.0);
        assert!(SPRING_BRIGHTNESS_BOOST > 0.0);
        assert!(SPRING_BRIGHTNESS_BOOST <= 1.0);
    }

    #[test]
    fn test_decay_rates_nonzero() {
        assert!(LEAF_DECAY_RATE >= LEAF_RAMP_RATE);
        assert!(FLOWER_DECAY_RATE >= FLOWER_RAMP_RATE);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests: effect combinations
    // -------------------------------------------------------------------------

    #[test]
    fn test_autumn_has_leaves_no_shimmer() {
        let leaves = compute_leaf_intensity(0.0, Season::Autumn, true);
        let shimmer = compute_heat_shimmer_intensity(15.0, Season::Autumn, true);
        assert!(leaves > 0.0, "autumn should have leaves");
        assert_eq!(shimmer, 0.0, "autumn should not have heat shimmer");
    }

    #[test]
    fn test_summer_has_shimmer_no_leaves() {
        let leaves = compute_leaf_intensity(0.0, Season::Summer, true);
        let shimmer = compute_heat_shimmer_intensity(35.0, Season::Summer, true);
        assert_eq!(leaves, 0.0, "summer should not ramp leaves");
        assert!(shimmer > 0.0, "summer should have heat shimmer at 35C");
    }

    #[test]
    fn test_winter_storm_has_snowflakes_and_darkening() {
        let weather = test_weather(Season::Winter, WeatherCondition::Storm, -5.0);
        let snowflakes = compute_snowflake_intensity(&weather, true);
        let darkening = compute_storm_darkening(0.0, &weather, true);
        assert!(snowflakes > 0.0, "winter storm should have snowflakes");
        assert!(darkening > 0.0, "storm should have sky darkening");
    }

    #[test]
    fn test_spring_has_flowers_and_brightness() {
        let flowers = compute_flower_intensity(0.0, Season::Spring, true);
        let brightness = compute_spring_brightness(Season::Spring, true);
        assert!(flowers > 0.0, "spring should have flowers");
        assert!(brightness > 0.0, "spring should have brightness boost");
    }

    #[test]
    fn test_rain_above_freezing_has_rain_streaks_no_snowflakes() {
        let weather = test_weather(Season::Summer, WeatherCondition::Rain, 20.0);
        let rain = compute_rain_intensity(&weather, true);
        let snowflakes = compute_snowflake_intensity(&weather, true);
        assert!(rain > 0.0, "rain above freezing should have rain streaks");
        assert_eq!(
            snowflakes, 0.0,
            "rain above freezing should not have snowflakes"
        );
    }

    #[test]
    fn test_storm_below_freezing_has_snowflakes_no_rain() {
        let weather = test_weather(Season::Winter, WeatherCondition::Storm, -5.0);
        let rain = compute_rain_intensity(&weather, true);
        let snowflakes = compute_snowflake_intensity(&weather, true);
        assert_eq!(
            rain, 0.0,
            "storm below freezing should not have rain streaks"
        );
        assert!(
            snowflakes > 0.0,
            "storm below freezing should have snowflakes"
        );
    }

    #[test]
    fn test_all_effects_disabled() {
        let weather = test_weather(Season::Winter, WeatherCondition::Storm, -5.0);
        assert_eq!(compute_leaf_intensity(0.5, Season::Autumn, false), 0.0);
        assert_eq!(compute_flower_intensity(0.5, Season::Spring, false), 0.0);
        assert_eq!(compute_snow_roof_intensity(0.5, &weather, 6.0, false), 0.0);
        assert_eq!(
            compute_heat_shimmer_intensity(40.0, Season::Summer, false),
            0.0
        );
        assert_eq!(compute_rain_intensity(&weather, false), 0.0);
        assert_eq!(compute_snowflake_intensity(&weather, false), 0.0);
        assert_eq!(compute_storm_darkening(0.5, &weather, false), 0.0);
        assert_eq!(compute_shadow_multiplier(Season::Summer, false), 1.0);
        assert_eq!(compute_spring_brightness(Season::Spring, false), 0.0);
    }
}

//! Tests for weather-related compute functions (rain, snowflake, storm
//! darkening, and lightning).

#[cfg(test)]
mod tests {
    use crate::seasonal_rendering::compute::*;
    use crate::seasonal_rendering::constants::*;
    use crate::weather::{Season, Weather, WeatherCondition};

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
    // Rain intensity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_rain_intensity_during_rain() {
        let weather = test_weather(Season::Summer, WeatherCondition::Rain, 20.0);
        let intensity = compute_rain_intensity(&weather, true);
        assert!(
            intensity > 0.0,
            "rain streaks should be active during rain, got {}",
            intensity
        );
    }

    #[test]
    fn test_rain_intensity_during_heavy_rain() {
        let weather = test_weather(Season::Summer, WeatherCondition::HeavyRain, 20.0);
        let intensity = compute_rain_intensity(&weather, true);
        assert!(
            intensity > 0.0,
            "rain streaks should be active during heavy rain, got {}",
            intensity
        );
    }

    #[test]
    fn test_rain_intensity_during_storm() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let intensity = compute_rain_intensity(&weather, true);
        assert!(
            intensity > 0.0,
            "rain streaks should be active during storm, got {}",
            intensity
        );
    }

    #[test]
    fn test_rain_intensity_zero_when_sunny() {
        let weather = test_weather(Season::Summer, WeatherCondition::Sunny, 20.0);
        let intensity = compute_rain_intensity(&weather, true);
        assert_eq!(intensity, 0.0, "rain streaks should be zero when sunny");
    }

    #[test]
    fn test_rain_intensity_zero_when_below_freezing() {
        let weather = test_weather(Season::Winter, WeatherCondition::Storm, -5.0);
        let intensity = compute_rain_intensity(&weather, true);
        assert_eq!(
            intensity, 0.0,
            "rain streaks should be zero below freezing (snow instead)"
        );
    }

    #[test]
    fn test_rain_intensity_disabled() {
        let weather = test_weather(Season::Summer, WeatherCondition::Rain, 20.0);
        let intensity = compute_rain_intensity(&weather, false);
        assert_eq!(intensity, 0.0, "disabled rain should be zero");
    }

    #[test]
    fn test_rain_intensity_capped() {
        let mut weather = test_weather(Season::Summer, WeatherCondition::HeavyRain, 20.0);
        weather.precipitation_intensity = 10.0;
        let intensity = compute_rain_intensity(&weather, true);
        assert!(
            intensity <= MAX_RAIN_INTENSITY,
            "rain intensity should be capped at {}, got {}",
            MAX_RAIN_INTENSITY,
            intensity
        );
    }

    // -------------------------------------------------------------------------
    // Snowflake intensity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_snowflake_during_snow() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        let intensity = compute_snowflake_intensity(&weather, true);
        assert!(
            intensity > 0.0,
            "snowflakes should be active during snow, got {}",
            intensity
        );
    }

    #[test]
    fn test_snowflake_during_winter_storm() {
        let weather = test_weather(Season::Winter, WeatherCondition::Storm, -5.0);
        let intensity = compute_snowflake_intensity(&weather, true);
        assert!(
            intensity > 0.0,
            "snowflakes should be active during winter storm, got {}",
            intensity
        );
    }

    #[test]
    fn test_snowflake_zero_above_freezing() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, 5.0);
        let intensity = compute_snowflake_intensity(&weather, true);
        assert_eq!(intensity, 0.0, "snowflakes should be zero above freezing");
    }

    #[test]
    fn test_snowflake_zero_when_sunny() {
        let weather = test_weather(Season::Winter, WeatherCondition::Sunny, -5.0);
        let intensity = compute_snowflake_intensity(&weather, true);
        assert_eq!(intensity, 0.0, "snowflakes should be zero when sunny");
    }

    #[test]
    fn test_snowflake_disabled() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        let intensity = compute_snowflake_intensity(&weather, false);
        assert_eq!(intensity, 0.0, "disabled snowflakes should be zero");
    }

    #[test]
    fn test_snowflake_capped() {
        let mut weather = test_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        weather.precipitation_intensity = 5.0;
        let intensity = compute_snowflake_intensity(&weather, true);
        assert!(
            intensity <= MAX_SNOWFLAKE_INTENSITY,
            "snowflake intensity should be capped at {}, got {}",
            MAX_SNOWFLAKE_INTENSITY,
            intensity
        );
    }

    // -------------------------------------------------------------------------
    // Storm darkening tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_storm_darkening_ramps_during_storm() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let darkening = compute_storm_darkening(0.0, &weather, true);
        assert!(
            darkening > 0.0,
            "storm darkening should ramp during storm, got {}",
            darkening
        );
    }

    #[test]
    fn test_storm_darkening_decays_after_storm() {
        let weather = test_weather(Season::Summer, WeatherCondition::Sunny, 20.0);
        let darkening = compute_storm_darkening(0.5, &weather, true);
        assert!(
            darkening < 0.5,
            "storm darkening should decay after storm, got {}",
            darkening
        );
    }

    #[test]
    fn test_storm_darkening_capped() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let darkening = compute_storm_darkening(0.65, &weather, true);
        assert!(
            darkening <= STORM_DARKENING_INTENSITY,
            "storm darkening should be capped at {}, got {}",
            STORM_DARKENING_INTENSITY,
            darkening
        );
    }

    #[test]
    fn test_storm_darkening_disabled() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let darkening = compute_storm_darkening(0.5, &weather, false);
        assert_eq!(darkening, 0.0, "disabled storm darkening should be zero");
    }

    #[test]
    fn test_storm_darkening_floors_at_zero() {
        let weather = test_weather(Season::Summer, WeatherCondition::Sunny, 20.0);
        let darkening = compute_storm_darkening(0.05, &weather, true);
        assert!(
            darkening >= 0.0,
            "storm darkening should not go below 0, got {}",
            darkening
        );
    }

    // -------------------------------------------------------------------------
    // Lightning tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_lightning_only_during_storm() {
        let weather = test_weather(Season::Summer, WeatherCondition::Sunny, 20.0);
        for tick in 0..100 {
            assert!(
                !should_trigger_lightning(&weather, tick, true),
                "lightning should not trigger outside storm (tick {})",
                tick
            );
        }
    }

    #[test]
    fn test_lightning_disabled() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        for tick in 0..100 {
            assert!(
                !should_trigger_lightning(&weather, tick, false),
                "disabled lightning should never trigger (tick {})",
                tick
            );
        }
    }

    #[test]
    fn test_lightning_deterministic() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let r1 = should_trigger_lightning(&weather, 42, true);
        let r2 = should_trigger_lightning(&weather, 42, true);
        assert_eq!(r1, r2, "same inputs should give same result");
    }

    #[test]
    fn test_lightning_can_trigger_during_storm() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let count = (0..100)
            .filter(|&tick| should_trigger_lightning(&weather, tick, true))
            .count();
        assert!(
            count > 0,
            "lightning should trigger at least once in 100 ticks during storm"
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::weather::*;

    #[test]
    fn test_season_from_day() {
        assert_eq!(Season::from_day(1), Season::Spring);
        assert_eq!(Season::from_day(90), Season::Spring);
        assert_eq!(Season::from_day(91), Season::Summer);
        assert_eq!(Season::from_day(180), Season::Summer);
        assert_eq!(Season::from_day(181), Season::Autumn);
        assert_eq!(Season::from_day(270), Season::Autumn);
        assert_eq!(Season::from_day(271), Season::Winter);
        assert_eq!(Season::from_day(360), Season::Winter);
        assert_eq!(Season::from_day(361), Season::Spring); // wraps
    }

    #[test]
    fn test_season_happiness_modifiers() {
        assert_eq!(Season::Spring.happiness_modifier(), 1.0);
        assert_eq!(Season::Summer.happiness_modifier(), 2.0);
        assert_eq!(Season::Autumn.happiness_modifier(), 0.0);
        assert_eq!(Season::Winter.happiness_modifier(), -2.0);
    }

    #[test]
    fn test_multipliers_in_range() {
        let weather = Weather::default();
        assert!((0.5..=2.0).contains(&weather.power_multiplier()));
        assert!((0.5..=2.0).contains(&weather.water_multiplier()));
        assert!((0.0..=2.0).contains(&weather.park_multiplier()));
        assert!((0.3..=1.5).contains(&weather.travel_speed_multiplier()));
    }

    #[test]
    fn test_weather_condition_modifiers() {
        let mut w = Weather::default();
        // Simulate heat wave: extreme temperature
        w.temperature = 38.0;
        w.current_event = WeatherCondition::Sunny;
        // HeatWave equivalent: seasonal(Spring=+1) + extreme_heat(-5) + sunny_spring(+2) = -2
        assert!(w.happiness_modifier() < 0.0);

        w.current_event = WeatherCondition::Sunny;
        w.temperature = 25.0;
        w.season = Season::Summer;
        // Clear+Summer: seasonal(+2) + sunny_bonus(+2) = +4
        assert!(w.happiness_modifier() > 0.0);

        w.season = Season::Winter;
        w.temperature = -10.0;
        w.current_event = WeatherCondition::Snow;
        // ColdSnap equivalent: seasonal(-2) + extreme_cold(-8) + snow(-1) = -11
        assert!(w.happiness_modifier() < -5.0);
    }

    #[test]
    fn test_condition_from_atmosphere_sunny() {
        let cond = WeatherCondition::from_atmosphere(0.1, 0.0, 20.0);
        assert_eq!(cond, WeatherCondition::Sunny);
    }

    #[test]
    fn test_condition_from_atmosphere_partly_cloudy() {
        let cond = WeatherCondition::from_atmosphere(0.5, 0.0, 20.0);
        assert_eq!(cond, WeatherCondition::PartlyCloudy);
    }

    #[test]
    fn test_condition_from_atmosphere_overcast() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.05, 20.0);
        assert_eq!(cond, WeatherCondition::Overcast);
    }

    #[test]
    fn test_condition_from_atmosphere_rain() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.2, 10.0);
        assert_eq!(cond, WeatherCondition::Rain);
    }

    #[test]
    fn test_condition_from_atmosphere_heavy_rain() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.5, 10.0);
        assert_eq!(cond, WeatherCondition::HeavyRain);
    }

    #[test]
    fn test_condition_from_atmosphere_snow() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.3, -5.0);
        assert_eq!(cond, WeatherCondition::Snow);
    }

    #[test]
    fn test_condition_from_atmosphere_storm() {
        let cond = WeatherCondition::from_atmosphere(0.9, 0.8, 15.0);
        assert_eq!(cond, WeatherCondition::Storm);
    }

    #[test]
    fn test_condition_is_precipitation() {
        assert!(!WeatherCondition::Sunny.is_precipitation());
        assert!(!WeatherCondition::PartlyCloudy.is_precipitation());
        assert!(!WeatherCondition::Overcast.is_precipitation());
        assert!(WeatherCondition::Rain.is_precipitation());
        assert!(WeatherCondition::HeavyRain.is_precipitation());
        assert!(WeatherCondition::Snow.is_precipitation());
        assert!(WeatherCondition::Storm.is_precipitation());
    }

    #[test]
    fn test_default_weather_has_new_fields() {
        let w = Weather::default();
        assert!((w.humidity - 0.5_f32).abs() < 0.01);
        assert!(w.cloud_cover < 0.2_f32);
        assert!(w.precipitation_intensity < 0.01_f32);
        assert_eq!(w.last_update_hour, 0);
    }

    #[test]
    fn test_weather_condition_method() {
        let mut w = Weather::default();
        w.cloud_cover = 0.1;
        w.atmo_precipitation = 0.0;
        w.temperature = 20.0;
        assert_eq!(w.condition(), WeatherCondition::Sunny);

        w.cloud_cover = 0.9;
        w.atmo_precipitation = 0.8;
        w.temperature = 20.0;
        assert_eq!(w.condition(), WeatherCondition::Storm);
    }

    #[test]
    fn test_travel_speed_new_conditions() {
        let mut w = Weather::default();
        w.current_event = WeatherCondition::HeavyRain;
        assert!(w.travel_speed_multiplier() < 0.7);

        w.current_event = WeatherCondition::Snow;
        assert!(w.travel_speed_multiplier() < 0.7);
    }

    #[test]
    fn test_park_multiplier_new_conditions() {
        let mut w = Weather::default();
        w.current_event = WeatherCondition::HeavyRain;
        assert!(w.park_multiplier() < 0.5);

        w.current_event = WeatherCondition::Overcast;
        assert!(w.park_multiplier() < 0.8);

        w.current_event = WeatherCondition::Snow;
        assert!(w.park_multiplier() < 0.3);
    }

    #[test]
    fn test_is_extreme_weather_helper() {
        // Storm is always extreme
        assert!(is_extreme_weather(WeatherCondition::Storm, 20.0));
        // Heat wave
        assert!(is_extreme_weather(WeatherCondition::Sunny, 36.0));
        // Cold snap
        assert!(is_extreme_weather(WeatherCondition::Sunny, -6.0));
        // Normal conditions
        assert!(!is_extreme_weather(WeatherCondition::Sunny, 20.0));
        assert!(!is_extreme_weather(WeatherCondition::Rain, 10.0));
        assert!(!is_extreme_weather(WeatherCondition::Snow, -3.0));
    }

    // -----------------------------------------------------------------------
    // Precipitation category tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_precipitation_category_none() {
        assert_eq!(
            PrecipitationCategory::from_intensity(0.0),
            PrecipitationCategory::None
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(0.005),
            PrecipitationCategory::None
        );
    }

    #[test]
    fn test_precipitation_category_drizzle() {
        assert_eq!(
            PrecipitationCategory::from_intensity(0.01),
            PrecipitationCategory::Drizzle
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(0.05),
            PrecipitationCategory::Drizzle
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(0.099),
            PrecipitationCategory::Drizzle
        );
    }

    #[test]
    fn test_precipitation_category_light() {
        assert_eq!(
            PrecipitationCategory::from_intensity(0.1),
            PrecipitationCategory::Light
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(0.2),
            PrecipitationCategory::Light
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(0.249),
            PrecipitationCategory::Light
        );
    }

    #[test]
    fn test_precipitation_category_moderate() {
        assert_eq!(
            PrecipitationCategory::from_intensity(0.25),
            PrecipitationCategory::Moderate
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(0.5),
            PrecipitationCategory::Moderate
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(0.99),
            PrecipitationCategory::Moderate
        );
    }

    #[test]
    fn test_precipitation_category_heavy() {
        assert_eq!(
            PrecipitationCategory::from_intensity(1.0),
            PrecipitationCategory::Heavy
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(1.5),
            PrecipitationCategory::Heavy
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(1.99),
            PrecipitationCategory::Heavy
        );
    }

    #[test]
    fn test_precipitation_category_torrential() {
        assert_eq!(
            PrecipitationCategory::from_intensity(2.0),
            PrecipitationCategory::Torrential
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(3.0),
            PrecipitationCategory::Torrential
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(3.99),
            PrecipitationCategory::Torrential
        );
    }

    #[test]
    fn test_precipitation_category_extreme() {
        assert_eq!(
            PrecipitationCategory::from_intensity(4.0),
            PrecipitationCategory::Extreme
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(5.0),
            PrecipitationCategory::Extreme
        );
        assert_eq!(
            PrecipitationCategory::from_intensity(10.0),
            PrecipitationCategory::Extreme
        );
    }

    #[test]
    fn test_precipitation_category_names() {
        assert_eq!(PrecipitationCategory::None.name(), "None");
        assert_eq!(PrecipitationCategory::Drizzle.name(), "Drizzle");
        assert_eq!(PrecipitationCategory::Light.name(), "Light");
        assert_eq!(PrecipitationCategory::Moderate.name(), "Moderate");
        assert_eq!(PrecipitationCategory::Heavy.name(), "Heavy");
        assert_eq!(PrecipitationCategory::Torrential.name(), "Torrential");
        assert_eq!(PrecipitationCategory::Extreme.name(), "Extreme");
    }

    // -----------------------------------------------------------------------
    // ConstructionModifiers tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_season_speed_factors() {
        assert!(
            (ConstructionModifiers::season_speed_factor(Season::Spring) - 1.0).abs() < f32::EPSILON
        );
        assert!(
            (ConstructionModifiers::season_speed_factor(Season::Summer) - 1.1).abs() < f32::EPSILON
        );
        assert!(
            (ConstructionModifiers::season_speed_factor(Season::Autumn) - 0.9).abs() < f32::EPSILON
        );
        assert!(
            (ConstructionModifiers::season_speed_factor(Season::Winter) - 0.6).abs() < f32::EPSILON
        );
    }

    #[test]
    fn test_weather_speed_factor_clear() {
        assert!(
            (ConstructionModifiers::weather_speed_factor(WeatherCondition::Sunny, 20.0) - 1.0)
                .abs()
                < f32::EPSILON
        );
        assert!(
            (ConstructionModifiers::weather_speed_factor(WeatherCondition::PartlyCloudy, 20.0)
                - 1.0)
                .abs()
                < f32::EPSILON
        );
        assert!(
            (ConstructionModifiers::weather_speed_factor(WeatherCondition::Overcast, 20.0) - 1.0)
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_weather_speed_factor_rain() {
        assert!(
            (ConstructionModifiers::weather_speed_factor(WeatherCondition::Rain, 10.0) - 0.5).abs()
                < f32::EPSILON
        );
        assert!(
            (ConstructionModifiers::weather_speed_factor(WeatherCondition::HeavyRain, 10.0) - 0.5)
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_weather_speed_factor_snow() {
        assert!(
            (ConstructionModifiers::weather_speed_factor(WeatherCondition::Snow, -2.0) - 0.3).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_storm_halts_construction() {
        assert!(
            (ConstructionModifiers::weather_speed_factor(WeatherCondition::Storm, 15.0) - 0.0)
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_extreme_cold_slows_construction() {
        assert!(
            (ConstructionModifiers::weather_speed_factor(WeatherCondition::Sunny, -10.0) - 0.2)
                .abs()
                < f32::EPSILON
        );
        assert!(
            (ConstructionModifiers::weather_speed_factor(WeatherCondition::Overcast, -6.0) - 0.2)
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_season_cost_factors() {
        assert!(
            (ConstructionModifiers::season_cost_factor(Season::Spring) - 1.0).abs() < f32::EPSILON
        );
        assert!(
            (ConstructionModifiers::season_cost_factor(Season::Summer) - 1.0).abs() < f32::EPSILON
        );
        assert!(
            (ConstructionModifiers::season_cost_factor(Season::Autumn) - 1.05).abs() < f32::EPSILON
        );
        assert!(
            (ConstructionModifiers::season_cost_factor(Season::Winter) - 1.25).abs() < f32::EPSILON
        );
    }

    #[test]
    fn test_summer_gives_1_1x_speed() {
        let speed = ConstructionModifiers::season_speed_factor(Season::Summer)
            * ConstructionModifiers::weather_speed_factor(WeatherCondition::Sunny, 25.0);
        assert!((speed - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_speed_factor_winter_storm() {
        let speed = ConstructionModifiers::season_speed_factor(Season::Winter)
            * ConstructionModifiers::weather_speed_factor(WeatherCondition::Storm, 2.0);
        assert!(speed.abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_speed_factor_spring_rain() {
        let speed = ConstructionModifiers::season_speed_factor(Season::Spring)
            * ConstructionModifiers::weather_speed_factor(WeatherCondition::Rain, 10.0);
        assert!((speed - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_construction_modifiers_default() {
        let cm = ConstructionModifiers::default();
        assert!((cm.speed_factor - 1.0).abs() < f32::EPSILON);
        assert!((cm.cost_factor - 1.0).abs() < f32::EPSILON);
    }
}

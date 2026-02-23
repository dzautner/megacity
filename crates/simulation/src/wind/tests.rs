#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::time_of_day::GameClock;
    use crate::weather::{ClimateZone, Weather, WeatherCondition};
    use crate::TickCounter;

    use super::super::systems::{
        angle_diff, rand_signed_f32, rand_unsigned_f32, splitmix64, update_wind,
        WIND_UPDATE_INTERVAL,
    };
    use super::super::types::{prevailing_direction_for_zone, WindState};

    #[test]
    fn test_wind_default() {
        let w = WindState::default();
        assert!((w.direction - 0.0).abs() < f32::EPSILON);
        assert!((w.speed - 0.3).abs() < f32::EPSILON);
        assert!((w.prevailing_direction - 0.0).abs() < f32::EPSILON);
        assert_eq!(w.gust_remaining, 0);
    }

    #[test]
    fn test_compass_direction() {
        let mut w = WindState::default();

        w.direction = 0.0;
        assert_eq!(w.compass_direction(), "E");

        w.direction = std::f32::consts::FRAC_PI_2;
        assert_eq!(w.compass_direction(), "N");

        w.direction = std::f32::consts::PI;
        assert_eq!(w.compass_direction(), "W");

        w.direction = 3.0 * std::f32::consts::FRAC_PI_2;
        assert_eq!(w.compass_direction(), "S");

        w.direction = std::f32::consts::FRAC_PI_4;
        assert_eq!(w.compass_direction(), "NE");
    }

    #[test]
    fn test_speed_label() {
        let mut w = WindState::default();

        w.speed = 0.0;
        assert_eq!(w.speed_label(), "Calm");

        w.speed = 0.2;
        assert_eq!(w.speed_label(), "Light");

        w.speed = 0.5;
        assert_eq!(w.speed_label(), "Moderate");

        w.speed = 0.9;
        assert_eq!(w.speed_label(), "Strong");
    }

    #[test]
    fn test_direction_vector() {
        let mut w = WindState::default();

        // East (0 rad): dx=1, dy=0
        w.direction = 0.0;
        let (dx, dy) = w.direction_vector();
        assert!((dx - 1.0).abs() < 0.01);
        assert!(dy.abs() < 0.01);

        // North (PI/2): dx=0, dy=1
        w.direction = std::f32::consts::FRAC_PI_2;
        let (dx, dy) = w.direction_vector();
        assert!(dx.abs() < 0.01);
        assert!((dy - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_splitmix64_deterministic() {
        let a = splitmix64(42);
        let b = splitmix64(42);
        assert_eq!(a, b);
        assert_ne!(splitmix64(42), splitmix64(43));
    }

    #[test]
    fn test_rand_signed_f32_range() {
        for seed in 0..1000u64 {
            let val = rand_signed_f32(seed);
            assert!(
                val >= -1.0 && val < 1.0,
                "rand_signed_f32({}) = {} out of range",
                seed,
                val
            );
        }
    }

    #[test]
    fn test_rand_unsigned_f32_range() {
        for seed in 0..1000u64 {
            let val = rand_unsigned_f32(seed);
            assert!(
                val >= 0.0 && val < 1.0,
                "rand_unsigned_f32({}) = {} out of range",
                seed,
                val
            );
        }
    }

    #[test]
    fn test_angle_diff() {
        // Same angle
        assert!((angle_diff(0.0, 0.0)).abs() < f32::EPSILON);

        // Quarter turn positive
        let diff = angle_diff(0.0, std::f32::consts::FRAC_PI_2);
        assert!((diff - std::f32::consts::FRAC_PI_2).abs() < 0.01);

        // Quarter turn negative (wrapping)
        let diff = angle_diff(std::f32::consts::FRAC_PI_2, 0.0);
        assert!((diff - (-std::f32::consts::FRAC_PI_2)).abs() < 0.01);

        // Shortest path across 0/2PI boundary
        let diff = angle_diff(0.1, std::f32::consts::TAU - 0.1);
        assert!(diff < 0.0, "Should take the short way around");
        assert!((diff - (-0.2)).abs() < 0.01);
    }

    #[test]
    fn test_prevailing_direction_for_zones() {
        // Temperate should be westerly (0.0)
        assert!((prevailing_direction_for_zone(ClimateZone::Temperate) - 0.0).abs() < f32::EPSILON);
        // Tropical should be trade winds (PI)
        assert!(
            (prevailing_direction_for_zone(ClimateZone::Tropical) - std::f32::consts::PI).abs()
                < f32::EPSILON
        );
        // Oceanic should be westerly (0.0)
        assert!((prevailing_direction_for_zone(ClimateZone::Oceanic) - 0.0).abs() < f32::EPSILON);
    }

    // -----------------------------------------------------------------------
    // Integration tests using Bevy App
    // -----------------------------------------------------------------------

    /// Helper: build a minimal Bevy App with wind system and required resources.
    fn wind_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<TickCounter>()
            .init_resource::<WindState>()
            .init_resource::<Weather>()
            .init_resource::<ClimateZone>()
            .init_resource::<GameClock>()
            .add_systems(Update, update_wind);
        app
    }

    /// Advance the app by setting the tick counter and running an update.
    fn advance_wind(app: &mut App, tick_value: u64) {
        app.world_mut().resource_mut::<TickCounter>().0 = tick_value;
        app.update();
    }

    #[test]
    fn test_wind_direction_trends_toward_prevailing() {
        let mut app = wind_test_app();

        // Start wind pointing south (3*PI/2), prevailing is east (0.0 = westerly default)
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.direction = 3.0 * std::f32::consts::FRAC_PI_2; // South
            wind.speed = 0.3;
        }

        // Set weather to mild (no storm/calm override)
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::PartlyCloudy;
            weather.cloud_cover = 0.4;
        }

        let initial_direction = app.world().resource::<WindState>().direction;

        // Run many wind updates
        for i in 1..=50 {
            advance_wind(&mut app, i * WIND_UPDATE_INTERVAL);
        }

        let final_direction = app.world().resource::<WindState>().direction;
        let prevailing = app.world().resource::<WindState>().prevailing_direction;

        // The wind should have moved closer to prevailing (0.0) from initial (3*PI/2)
        let initial_dist = angle_diff(initial_direction, prevailing).abs();
        let final_dist = angle_diff(final_direction, prevailing).abs();

        assert!(
            final_dist < initial_dist,
            "Wind should trend toward prevailing direction. Initial distance: {}, Final distance: {}",
            initial_dist,
            final_dist
        );
    }

    #[test]
    fn test_storm_increases_wind_speed() {
        let mut app = wind_test_app();

        // Set storm weather
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Storm;
            weather.cloud_cover = 0.95;
            weather.precipitation_intensity = 0.85;
        }

        // Start with low speed
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.1;
            // Set prev_condition to Storm so no gust fires from transition
            wind.prev_condition = WeatherCondition::Storm;
        }

        // Run several updates to let speed converge
        for i in 1..=20 {
            advance_wind(&mut app, i * WIND_UPDATE_INTERVAL);
        }

        let final_speed = app.world().resource::<WindState>().speed;
        assert!(
            final_speed >= 0.6,
            "Storm wind speed should reach 0.6+, got {}",
            final_speed
        );
    }

    #[test]
    fn test_calm_clear_weather_low_wind() {
        let mut app = wind_test_app();

        // Set clear sunny weather with very low cloud cover (high pressure proxy)
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.05;
            weather.precipitation_intensity = 0.0;
        }

        // Start with moderate speed
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.5;
            // Set prev_condition to Sunny so no gust fires
            wind.prev_condition = WeatherCondition::Sunny;
        }

        // Run several updates to let speed converge downward
        for i in 1..=30 {
            advance_wind(&mut app, i * WIND_UPDATE_INTERVAL);
        }

        let final_speed = app.world().resource::<WindState>().speed;
        assert!(
            final_speed <= 0.15,
            "Calm clear weather should produce low wind speed (<= 0.15), got {}",
            final_speed
        );
    }

    #[test]
    fn test_diurnal_afternoon_boost() {
        // Test that afternoon hours produce higher wind speed than morning
        let mut app_afternoon = wind_test_app();
        let mut app_morning = wind_test_app();

        // Both start with identical state
        let setup = |app: &mut App| {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::PartlyCloudy;
            weather.cloud_cover = 0.4;

            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.3;
            wind.prev_condition = WeatherCondition::PartlyCloudy;
        };

        setup(&mut app_afternoon);
        setup(&mut app_morning);

        // Set afternoon clock (hour 15)
        app_afternoon.world_mut().resource_mut::<GameClock>().hour = 15.0;
        // Set morning clock (hour 8)
        app_morning.world_mut().resource_mut::<GameClock>().hour = 8.0;

        advance_wind(&mut app_afternoon, WIND_UPDATE_INTERVAL);
        advance_wind(&mut app_morning, WIND_UPDATE_INTERVAL);

        let afternoon_speed = app_afternoon.world().resource::<WindState>().speed;
        let morning_speed = app_morning.world().resource::<WindState>().speed;

        assert!(
            afternoon_speed > morning_speed,
            "Afternoon wind ({}) should be stronger than morning wind ({})",
            afternoon_speed,
            morning_speed
        );
    }

    #[test]
    fn test_gust_on_weather_transition() {
        let mut app = wind_test_app();

        // Start with sunny weather, record prev_condition as Sunny
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
        }
        {
            let mut wind = app.world_mut().resource_mut::<WindState>();
            wind.speed = 0.3;
            wind.prev_condition = WeatherCondition::Sunny;
        }

        // Now switch to storm -- this should trigger a gust
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Storm;
            weather.cloud_cover = 0.95;
            weather.precipitation_intensity = 0.85;
        }

        advance_wind(&mut app, WIND_UPDATE_INTERVAL);

        // After the update, prev_condition should now be Storm
        let wind = app.world().resource::<WindState>();
        assert_eq!(
            wind.prev_condition,
            WeatherCondition::Storm,
            "prev_condition should update to Storm"
        );
        // Speed should be elevated (storm target + gust boost)
        // At minimum the storm target alone is 0.6+, gust adds 0.2+
        assert!(
            wind.speed >= 0.4,
            "Wind speed during gust should be elevated, got {}",
            wind.speed
        );
    }

    #[test]
    fn test_climate_zone_changes_prevailing() {
        let mut app = wind_test_app();

        // Set tropical climate zone
        *app.world_mut().resource_mut::<ClimateZone>() = ClimateZone::Tropical;

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::PartlyCloudy;
            weather.cloud_cover = 0.4;
        }

        advance_wind(&mut app, WIND_UPDATE_INTERVAL);

        let wind = app.world().resource::<WindState>();
        assert!(
            (wind.prevailing_direction - std::f32::consts::PI).abs() < f32::EPSILON,
            "Tropical zone should have prevailing direction PI (trade winds), got {}",
            wind.prevailing_direction
        );
    }
}

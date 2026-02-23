#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::time_of_day::GameClock;
    use crate::weather::*;

    #[test]
    fn test_diurnal_factor_peak_at_15() {
        let peak = diurnal_factor(15);
        assert!(
            (peak - 1.0_f32).abs() < 0.01,
            "Peak at 15:00 should be ~1.0, got {}",
            peak
        );
    }

    #[test]
    fn test_diurnal_factor_minimum_at_06() {
        let minimum = diurnal_factor(6);
        assert!(
            minimum.abs() < 0.01_f32,
            "Minimum at 06:00 should be ~0.0, got {}",
            minimum
        );
    }

    #[test]
    fn test_diurnal_factor_range() {
        for hour in 0..24 {
            let f = diurnal_factor(hour);
            assert!(
                f >= -0.01 && f <= 1.01,
                "diurnal_factor({}) = {} out of range",
                hour,
                f
            );
        }
    }

    #[test]
    fn test_diurnal_factor_monotonic_morning() {
        // Should be monotonically increasing from 6 to 15
        let mut prev = diurnal_factor(6);
        for hour in 7..=15 {
            let current = diurnal_factor(hour);
            assert!(
                current >= prev,
                "diurnal_factor should increase from {} to {}: {} < {}",
                hour - 1,
                hour,
                current,
                prev
            );
            prev = current;
        }
    }

    #[test]
    fn test_diurnal_factor_monotonic_evening() {
        // Should be monotonically decreasing from 15 to 6 (next day)
        let mut prev = diurnal_factor(15);
        for hour_offset in 1..=15 {
            let hour = (15 + hour_offset) % 24;
            let current = diurnal_factor(hour);
            assert!(
                current <= prev + 0.01, // small epsilon for floating point
                "diurnal_factor should decrease from {} to {}: {} > {}",
                (hour + 23) % 24,
                hour,
                current,
                prev
            );
            prev = current;
        }
    }

    #[test]
    fn test_smooth_temperature_transition() {
        // Verify that the smooth transition formula converges
        let target: f32 = 25.0;
        let mut temp: f32 = 10.0;
        for _ in 0..20 {
            temp += (target - temp) * 0.3;
        }
        assert!(
            (temp - target).abs() < 0.1,
            "Temperature should converge to target, got {}",
            temp
        );
    }

    #[test]
    fn test_hourly_temperature_varies() {
        // Check that temperature at 6am differs from temperature at 3pm for summer
        let (t_min, t_max) = Season::Summer.temperature_range_for_zone(ClimateZone::Temperate);
        let factor_6 = diurnal_factor(6);
        let factor_15 = diurnal_factor(15);
        let temp_6 = t_min + (t_max - t_min) * factor_6;
        let temp_15 = t_min + (t_max - t_min) * factor_15;
        assert!(
            temp_15 > temp_6 + 5.0,
            "Afternoon should be significantly warmer: {}C vs {}C",
            temp_15,
            temp_6
        );
    }

    // -----------------------------------------------------------------------
    // WeatherChangeEvent tests
    // -----------------------------------------------------------------------

    /// Helper: build a minimal Bevy App with weather system and resources.
    fn weather_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<GameClock>()
            .init_resource::<Weather>()
            .init_resource::<ClimateZone>()
            .add_event::<WeatherChangeEvent>()
            .add_systems(Update, update_weather);
        app
    }

    #[test]
    fn test_event_fired_on_clear_to_rain_transition() {
        let mut app = weather_test_app();

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.atmo_precipitation = 0.0;
            weather.temperature = 20.0;
            weather.last_update_day = 1;
            weather.last_update_hour = 5;
            weather.season = Season::Spring;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 1;
            clock.hour = 6.0;
        }

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.cloud_cover = 0.8;
            weather.atmo_precipitation = 0.3;
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(
            !fired.is_empty(),
            "WeatherChangeEvent should fire when condition changes"
        );
        let evt = &fired[0];
        assert_eq!(evt.old_condition, WeatherCondition::Sunny);
        assert_eq!(evt.new_condition, WeatherCondition::Rain);
        assert!(!evt.is_extreme, "Rain is not extreme weather");
    }

    #[test]
    fn test_event_is_extreme_for_storm() {
        let mut app = weather_test_app();

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.atmo_precipitation = 0.0;
            weather.temperature = 20.0;
            weather.last_update_day = 1;
            weather.last_update_hour = 5;
            weather.season = Season::Summer;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 1;
            clock.hour = 6.0;
        }

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.cloud_cover = 0.95;
            weather.atmo_precipitation = 0.85;
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(!fired.is_empty(), "Event should fire for Storm");
        let evt = &fired[0];
        assert_eq!(evt.new_condition, WeatherCondition::Storm);
        assert!(evt.is_extreme, "Storm should be flagged as extreme");
    }

    #[test]
    fn test_event_is_extreme_for_heat_wave() {
        let mut app = weather_test_app();

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.05;
            weather.atmo_precipitation = 0.0;
            weather.temperature = 50.0;
            weather.last_update_day = 120;
            weather.last_update_hour = 14;
            weather.season = Season::Summer;
            weather.event_days_remaining = 5;
            weather.prev_extreme = false;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 120;
            clock.hour = 15.0;
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(
            !fired.is_empty(),
            "Event should fire when crossing extreme heat threshold"
        );
        let evt = &fired[fired.len() - 1];
        assert!(evt.is_extreme, "Temperature > 35C should be extreme");
    }

    #[test]
    fn test_event_is_extreme_for_cold_snap() {
        let mut app = weather_test_app();

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.2;
            weather.atmo_precipitation = 0.0;
            weather.temperature = -25.0;
            weather.last_update_day = 300;
            weather.last_update_hour = 5;
            weather.season = Season::Winter;
            weather.event_days_remaining = 5;
            weather.prev_extreme = false;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 300;
            clock.hour = 6.0;
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(
            !fired.is_empty(),
            "Event should fire when crossing extreme cold threshold"
        );
        let evt = &fired[fired.len() - 1];
        assert!(evt.is_extreme, "Temperature < -5C should be extreme");
    }

    #[test]
    fn test_event_fired_on_season_change() {
        let mut app = weather_test_app();

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.atmo_precipitation = 0.0;
            weather.temperature = 20.0;
            weather.last_update_day = 90;
            weather.last_update_hour = 11;
            weather.season = Season::Spring;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 91;
            clock.hour = 12.0;
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(!fired.is_empty(), "Event should fire on season transition");
        let evt = &fired[0];
        assert_eq!(evt.old_season, Season::Spring);
        assert_eq!(evt.new_season, Season::Summer);
    }

    #[test]
    fn test_no_event_when_nothing_changes() {
        let mut app = weather_test_app();

        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.atmo_precipitation = 0.0;
            weather.temperature = 15.0;
            weather.last_update_day = 1;
            weather.last_update_hour = 5;
            weather.season = Season::Spring;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 1;
            clock.hour = 6.0;
        }

        app.update();

        let events = app.world().resource::<Events<WeatherChangeEvent>>();
        let mut reader = events.get_cursor();
        let fired: Vec<_> = reader.read(events).collect();

        assert!(
            fired.is_empty(),
            "No event should fire when weather does not change; got {} events",
            fired.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::weather::*;

    // -----------------------------------------------------------------------
    // Climate Zone tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_climate_zone_default_is_temperate() {
        let zone = ClimateZone::default();
        assert_eq!(zone, ClimateZone::Temperate);
    }

    #[test]
    fn test_tropical_winter_low_about_18c_no_snow() {
        let params = ClimateZone::Tropical.season_params(Season::Winter);
        assert!(
            (params.t_min - 18.0).abs() < 1.0,
            "Tropical winter low should be ~18C (65F), got {}",
            params.t_min
        );
        assert!(
            !params.snow_enabled,
            "Tropical zone should have snow disabled"
        );
        for &season in &[
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ] {
            let p = ClimateZone::Tropical.season_params(season);
            assert!(
                !p.snow_enabled,
                "Tropical {:?} should not have snow",
                season
            );
        }
    }

    #[test]
    fn test_subarctic_winter_low_about_negative_34c_heavy_snow() {
        let params = ClimateZone::Subarctic.season_params(Season::Winter);
        assert!(
            (params.t_min - (-34.0)).abs() < 2.0,
            "Subarctic winter low should be ~-34C (-30F), got {}",
            params.t_min
        );
        assert!(
            params.snow_enabled,
            "Subarctic winter should have snow enabled"
        );
        assert!(
            params.precipitation_chance >= 0.12,
            "Subarctic winter should have high precipitation chance, got {}",
            params.precipitation_chance
        );
    }

    #[test]
    fn test_arid_very_low_precipitation() {
        for &season in &[
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ] {
            let params = ClimateZone::Arid.season_params(season);
            assert!(
                params.precipitation_chance <= 0.05,
                "Arid {:?} precipitation chance should be very low, got {}",
                season,
                params.precipitation_chance
            );
        }
        let summer = ClimateZone::Arid.season_params(Season::Summer);
        assert!(
            summer.precipitation_chance <= 0.02,
            "Arid summer should be extremely dry, got {}",
            summer.precipitation_chance
        );
    }

    #[test]
    fn test_temperate_backward_compatible() {
        let spring = ClimateZone::Temperate.season_params(Season::Spring);
        assert!((spring.t_min - 8.0).abs() < 0.01);
        assert!((spring.t_max - 22.0).abs() < 0.01);

        let summer = ClimateZone::Temperate.season_params(Season::Summer);
        assert!((summer.t_min - 20.0).abs() < 0.01);
        assert!((summer.t_max - 36.0).abs() < 0.01);

        let autumn = ClimateZone::Temperate.season_params(Season::Autumn);
        assert!((autumn.t_min - 5.0).abs() < 0.01);
        assert!((autumn.t_max - 19.0).abs() < 0.01);

        let winter = ClimateZone::Temperate.season_params(Season::Winter);
        assert!((winter.t_min - (-8.0)).abs() < 0.01);
        assert!((winter.t_max - 6.0).abs() < 0.01);
        assert!(winter.snow_enabled);
    }

    #[test]
    fn test_all_zones_have_valid_temperature_ranges() {
        for &zone in ClimateZone::all() {
            for &season in &[
                Season::Spring,
                Season::Summer,
                Season::Autumn,
                Season::Winter,
            ] {
                let params = zone.season_params(season);
                assert!(
                    params.t_max > params.t_min,
                    "{:?} {:?}: t_max ({}) should be > t_min ({})",
                    zone,
                    season,
                    params.t_max,
                    params.t_min
                );
                assert!(
                    (0.0..=1.0).contains(&params.precipitation_chance),
                    "{:?} {:?}: precipitation_chance {} out of range",
                    zone,
                    season,
                    params.precipitation_chance
                );
            }
        }
    }

    #[test]
    fn test_continental_extreme_temperature_swing() {
        let summer = ClimateZone::Continental.season_params(Season::Summer);
        let winter = ClimateZone::Continental.season_params(Season::Winter);
        let swing = summer.t_max - winter.t_min;
        assert!(
            swing > 50.0,
            "Continental temperature swing should be >50C, got {}",
            swing
        );
    }

    #[test]
    fn test_mediterranean_dry_summers_wet_winters() {
        let summer = ClimateZone::Mediterranean.season_params(Season::Summer);
        let winter = ClimateZone::Mediterranean.season_params(Season::Winter);
        assert!(
            winter.precipitation_chance > summer.precipitation_chance * 3.0,
            "Mediterranean winters should be much wetter than summers: winter={}, summer={}",
            winter.precipitation_chance,
            summer.precipitation_chance
        );
    }

    #[test]
    fn test_oceanic_narrow_temperature_range() {
        let summer = ClimateZone::Oceanic.season_params(Season::Summer);
        let winter = ClimateZone::Oceanic.season_params(Season::Winter);
        let annual_range = summer.t_max - winter.t_min;
        assert!(
            annual_range < 25.0,
            "Oceanic annual temperature range should be < 25C, got {}",
            annual_range
        );
    }

    #[test]
    fn test_climate_zone_names() {
        assert_eq!(ClimateZone::Temperate.name(), "Temperate");
        assert_eq!(ClimateZone::Tropical.name(), "Tropical");
        assert_eq!(ClimateZone::Arid.name(), "Arid");
        assert_eq!(ClimateZone::Mediterranean.name(), "Mediterranean");
        assert_eq!(ClimateZone::Continental.name(), "Continental");
        assert_eq!(ClimateZone::Subarctic.name(), "Subarctic");
        assert_eq!(ClimateZone::Oceanic.name(), "Oceanic");
    }

    #[test]
    fn test_climate_zone_all_variants() {
        let all = ClimateZone::all();
        assert_eq!(all.len(), 7);
    }

    // -----------------------------------------------------------------------
    // Precipitation intensity by event tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_clear_conditions_produce_zero_intensity() {
        for condition in [
            WeatherCondition::Sunny,
            WeatherCondition::PartlyCloudy,
            WeatherCondition::Overcast,
        ] {
            let intensity = precipitation_intensity_for_event(condition, Season::Summer, 50);
            assert!(
                intensity.abs() < f32::EPSILON,
                "{:?} should produce 0.0 intensity, got {}",
                condition,
                intensity,
            );
        }
    }

    #[test]
    fn test_rain_intensity_range() {
        for hash in [0, 25, 50, 75, 99] {
            for season in [
                Season::Spring,
                Season::Summer,
                Season::Autumn,
                Season::Winter,
            ] {
                let intensity =
                    precipitation_intensity_for_event(WeatherCondition::Rain, season, hash);
                assert!(
                    intensity >= 0.1 && intensity <= 1.0,
                    "Rain intensity for {:?} hash={} should be in [0.1, 1.0], got {}",
                    season,
                    hash,
                    intensity,
                );
            }
        }
    }

    #[test]
    fn test_heavy_rain_intensity_range() {
        for hash in [0, 25, 50, 75, 99] {
            for season in [
                Season::Spring,
                Season::Summer,
                Season::Autumn,
                Season::Winter,
            ] {
                let intensity =
                    precipitation_intensity_for_event(WeatherCondition::HeavyRain, season, hash);
                assert!(
                    intensity >= 1.0 && intensity <= 2.5,
                    "HeavyRain intensity for {:?} hash={} should be in [1.0, 2.5], got {}",
                    season,
                    hash,
                    intensity,
                );
            }
        }
    }

    #[test]
    fn test_storm_intensity_minimum() {
        for hash in [0, 25, 50, 75, 99] {
            for season in [
                Season::Spring,
                Season::Summer,
                Season::Autumn,
                Season::Winter,
            ] {
                let intensity =
                    precipitation_intensity_for_event(WeatherCondition::Storm, season, hash);
                assert!(
                    intensity >= 2.0,
                    "Storm intensity for {:?} hash={} should be >= 2.0, got {}",
                    season,
                    hash,
                    intensity,
                );
            }
        }
    }

    #[test]
    fn test_storm_produces_higher_intensity_than_rain() {
        let hash = 50;
        for season in [
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ] {
            let rain = precipitation_intensity_for_event(WeatherCondition::Rain, season, hash);
            let storm = precipitation_intensity_for_event(WeatherCondition::Storm, season, hash);
            assert!(
                storm > rain,
                "Storm ({}) should produce higher intensity than Rain ({}) in {:?}",
                storm,
                rain,
                season,
            );
        }
    }

    #[test]
    fn test_summer_rain_heavier_than_winter() {
        let hash = 50;
        let summer =
            precipitation_intensity_for_event(WeatherCondition::Rain, Season::Summer, hash);
        let winter =
            precipitation_intensity_for_event(WeatherCondition::Rain, Season::Winter, hash);
        assert!(
            summer > winter,
            "Summer rain ({}) should be heavier than winter rain ({})",
            summer,
            winter,
        );
    }

    #[test]
    fn test_snow_water_equivalent_range() {
        for hash in [0, 50, 99] {
            let intensity =
                precipitation_intensity_for_event(WeatherCondition::Snow, Season::Winter, hash);
            assert!(
                intensity >= 0.05 && intensity <= 0.30,
                "Snow water equivalent should be in [0.05, 0.30], got {}",
                intensity,
            );
        }
    }

    // -----------------------------------------------------------------------
    // Daily accumulation and rolling window tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_daily_rainfall_accumulation() {
        let mut w = Weather::default();
        assert_eq!(w.daily_rainfall, 0.0);

        w.daily_rainfall += 1.0;
        w.daily_rainfall += 1.0;
        w.daily_rainfall += 1.0;
        assert!((w.daily_rainfall - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_roll_daily_rainfall() {
        use crate::weather::systems::roll_daily_rainfall;

        let mut w = Weather::default();
        w.daily_rainfall = 2.5;

        roll_daily_rainfall(&mut w);

        assert_eq!(w.daily_rainfall, 0.0);
        assert!((w.rainfall_history[29] - 2.5).abs() < f32::EPSILON);
        assert_eq!(w.rainfall_history.len(), 30);
    }

    #[test]
    fn test_rolling_30day_total() {
        use crate::weather::systems::roll_daily_rainfall;

        let mut w = Weather::default();

        for _ in 0..5 {
            w.daily_rainfall = 1.0;
            roll_daily_rainfall(&mut w);
        }

        let total: f32 = w.rainfall_history.iter().sum();
        assert!(
            (total - 5.0).abs() < f32::EPSILON,
            "Rolling total should be 5.0 after 5 days of 1.0 in, got {}",
            total,
        );
    }

    #[test]
    fn test_rolling_window_drops_oldest() {
        use crate::weather::systems::roll_daily_rainfall;

        let mut w = Weather::default();

        for _ in 0..30 {
            w.daily_rainfall = 1.0;
            roll_daily_rainfall(&mut w);
        }

        let total: f32 = w.rainfall_history.iter().sum();
        assert!(
            (total - 30.0).abs() < f32::EPSILON,
            "All 30 days filled with 1.0 should total 30.0, got {}",
            total,
        );

        w.daily_rainfall = 0.0;
        roll_daily_rainfall(&mut w);

        let total: f32 = w.rainfall_history.iter().sum();
        assert!(
            (total - 29.0).abs() < f32::EPSILON,
            "After adding 0.0 day, total should be 29.0, got {}",
            total,
        );
    }

    #[test]
    fn test_precipitation_category_from_weather() {
        let mut w = Weather::default();
        w.precipitation_intensity = 0.0;
        assert_eq!(w.precipitation_category(), PrecipitationCategory::None);

        w.precipitation_intensity = 0.5;
        assert_eq!(w.precipitation_category(), PrecipitationCategory::Moderate);

        w.precipitation_intensity = 3.0;
        assert_eq!(
            w.precipitation_category(),
            PrecipitationCategory::Torrential
        );
    }

    #[test]
    fn test_default_weather_has_precipitation_fields() {
        let w = Weather::default();
        assert_eq!(w.daily_rainfall, 0.0);
        assert_eq!(w.rolling_30day_rainfall, 0.0);
        assert_eq!(w.rainfall_history.len(), 30);
        assert!(w.rainfall_history.iter().all(|&v| v == 0.0));
        assert_eq!(w.atmo_precipitation, 0.0);
    }

    #[test]
    fn test_rainfall_history_repairs_wrong_size() {
        use crate::weather::systems::roll_daily_rainfall;

        let mut w = Weather::default();
        w.rainfall_history = vec![1.0; 5];
        w.daily_rainfall = 2.0;

        roll_daily_rainfall(&mut w);

        assert_eq!(w.rainfall_history.len(), 30);
    }
}

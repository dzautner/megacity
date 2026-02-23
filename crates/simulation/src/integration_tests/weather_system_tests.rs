//! Integration tests for the weather system (Issue #825 / TEST-046).
//!
//! Covers:
//! - Season transitions at correct day boundaries
//! - Temperature ranges appropriate for each season
//! - Precipitation variation by season
//! - Weather state serialization round-trip

use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::weather::climate::ClimateZone;
use crate::weather::state::Weather;
use crate::weather::types::Season;

// ---------------------------------------------------------------------------
// Helper: advance the city clock to a specific day + hour, then tick once
// so the weather system processes the new time.
// ---------------------------------------------------------------------------

/// Set the game clock to the given day/hour and tick once to trigger
/// the weather update system.
fn advance_to(city: &mut TestCity, day: u32, hour: f32) {
    {
        let world = city.world_mut();
        let mut clock = world.resource_mut::<GameClock>();
        clock.day = day;
        clock.hour = hour;
    }
    city.tick(1);
}

// ---------------------------------------------------------------------------
// 1. Season transitions at correct day boundaries
// ---------------------------------------------------------------------------

#[test]
fn test_weather_season_transition_spring_to_summer() {
    let mut city = TestCity::new();

    // Day 90 is the last day of Spring
    advance_to(&mut city, 90, 12.0);
    let weather = city.resource::<Weather>();
    assert_eq!(
        weather.season,
        Season::Spring,
        "Day 90 should be Spring, got {:?}",
        weather.season
    );

    // Day 91 is the first day of Summer
    advance_to(&mut city, 91, 12.0);
    let weather = city.resource::<Weather>();
    assert_eq!(
        weather.season,
        Season::Summer,
        "Day 91 should be Summer, got {:?}",
        weather.season
    );
}

#[test]
fn test_weather_season_transition_summer_to_autumn() {
    let mut city = TestCity::new();

    advance_to(&mut city, 180, 12.0);
    let weather = city.resource::<Weather>();
    assert_eq!(
        weather.season,
        Season::Summer,
        "Day 180 should be Summer, got {:?}",
        weather.season
    );

    advance_to(&mut city, 181, 12.0);
    let weather = city.resource::<Weather>();
    assert_eq!(
        weather.season,
        Season::Autumn,
        "Day 181 should be Autumn, got {:?}",
        weather.season
    );
}

#[test]
fn test_weather_season_transition_autumn_to_winter() {
    let mut city = TestCity::new();

    advance_to(&mut city, 270, 12.0);
    let weather = city.resource::<Weather>();
    assert_eq!(
        weather.season,
        Season::Autumn,
        "Day 270 should be Autumn, got {:?}",
        weather.season
    );

    advance_to(&mut city, 271, 12.0);
    let weather = city.resource::<Weather>();
    assert_eq!(
        weather.season,
        Season::Winter,
        "Day 271 should be Winter, got {:?}",
        weather.season
    );
}

#[test]
fn test_weather_season_transition_winter_to_spring_year_wrap() {
    let mut city = TestCity::new();

    advance_to(&mut city, 360, 12.0);
    let weather = city.resource::<Weather>();
    assert_eq!(
        weather.season,
        Season::Winter,
        "Day 360 should be Winter, got {:?}",
        weather.season
    );

    // Day 361 wraps the 360-day year back to Spring
    advance_to(&mut city, 361, 12.0);
    let weather = city.resource::<Weather>();
    assert_eq!(
        weather.season,
        Season::Spring,
        "Day 361 should wrap to Spring, got {:?}",
        weather.season
    );
}

// ---------------------------------------------------------------------------
// 2. Temperature ranges appropriate for each season (Temperate zone)
// ---------------------------------------------------------------------------

#[test]
fn test_weather_temperature_within_seasonal_range_after_convergence() {
    let mut city = TestCity::new();

    // For each season, advance to the middle of that season and let
    // temperature converge over several hour ticks. The smooth transition
    // (0.3 factor) needs multiple ticks to converge from the default 15C.
    let season_days: [(u32, Season); 4] = [
        (45, Season::Spring),  // mid-Spring
        (135, Season::Summer), // mid-Summer
        (225, Season::Autumn), // mid-Autumn
        (315, Season::Winter), // mid-Winter
    ];

    let zone = ClimateZone::Temperate;

    for (day, expected_season) in &season_days {
        // Reset weather temperature to a neutral value so convergence
        // works the same for each test case.
        {
            let world = city.world_mut();
            let mut weather = world.resource_mut::<Weather>();
            weather.temperature = 10.0;
            weather.last_update_day = 0;
            weather.last_update_hour = 0;
        }

        // Tick through 24+ hours at the target day to let temperature converge
        for hour in 0..48 {
            advance_to(&mut city, *day, (hour % 24) as f32);
        }

        let weather = city.resource::<Weather>();
        assert_eq!(
            weather.season, *expected_season,
            "Day {} should be {:?}",
            day, expected_season
        );

        let params = zone.season_params(*expected_season);
        // Allow +/- 6C beyond the climate range to account for daily variation
        // (deterministic hash adds up to +/- 3C) and smooth transition lag.
        let margin = 6.0;
        assert!(
            weather.temperature >= params.t_min - margin
                && weather.temperature <= params.t_max + margin,
            "Temperature {} should be near [{}, {}] for {:?} (with {}C margin)",
            weather.temperature,
            params.t_min,
            params.t_max,
            expected_season,
            margin
        );
    }
}

#[test]
fn test_weather_summer_warmer_than_winter() {
    let mut city = TestCity::new();

    // Converge at mid-Summer afternoon (day 135, 15:00 = peak diurnal)
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.temperature = 20.0;
        weather.last_update_day = 0;
        weather.last_update_hour = 0;
    }
    for hour in 0..48 {
        advance_to(&mut city, 135, (hour % 24) as f32);
    }
    let summer_temp = city.resource::<Weather>().temperature;

    // Converge at mid-Winter morning (day 315, 6:00 = diurnal minimum)
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.temperature = 0.0;
        weather.last_update_day = 0;
        weather.last_update_hour = 0;
    }
    for hour in 0..48 {
        advance_to(&mut city, 315, (hour % 24) as f32);
    }
    let winter_temp = city.resource::<Weather>().temperature;

    assert!(
        summer_temp > winter_temp + 10.0,
        "Summer temp ({:.1}C) should be significantly warmer than winter ({:.1}C)",
        summer_temp,
        winter_temp
    );
}

// ---------------------------------------------------------------------------
// 3. Precipitation variation by season
// ---------------------------------------------------------------------------

#[test]
fn test_weather_precipitation_intensity_varies_by_condition() {
    // This test verifies that precipitation intensity is correctly set based
    // on the weather condition (via the update_weather system), without
    // relying on randomized weather events. We set atmospheric values directly.
    let mut city = TestCity::new();

    // Set up a clear day — precipitation should be 0
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.cloud_cover = 0.1;
        weather.atmo_precipitation = 0.0;
        weather.temperature = 20.0;
        weather.last_update_day = 0;
        weather.last_update_hour = 0;
        weather.event_days_remaining = 0;
    }
    advance_to(&mut city, 50, 12.0);
    let clear_intensity = city.resource::<Weather>().precipitation_intensity;

    // Set up a rainy day — atmo_precipitation triggers Rain condition
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.cloud_cover = 0.8;
        weather.atmo_precipitation = 0.3;
        weather.temperature = 15.0;
        weather.last_update_hour = 11; // force hour boundary crossing
        weather.event_days_remaining = 3; // prevent the daily event logic from overwriting
    }
    advance_to(&mut city, 50, 12.0);
    let rain_intensity = city.resource::<Weather>().precipitation_intensity;

    // Set up a storm — high atmo_precipitation and cloud cover
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<Weather>();
        weather.cloud_cover = 0.95;
        weather.atmo_precipitation = 0.85;
        weather.temperature = 20.0;
        weather.last_update_hour = 11;
        weather.event_days_remaining = 3;
    }
    advance_to(&mut city, 50, 12.0);
    let storm_intensity = city.resource::<Weather>().precipitation_intensity;

    assert!(
        clear_intensity < 0.01,
        "Clear sky should have ~0 precipitation intensity, got {}",
        clear_intensity
    );
    assert!(
        rain_intensity > 0.05,
        "Rain should have positive precipitation intensity, got {}",
        rain_intensity
    );
    assert!(
        storm_intensity > rain_intensity,
        "Storm intensity ({}) should exceed rain intensity ({})",
        storm_intensity,
        rain_intensity
    );
}

#[test]
fn test_weather_precipitation_summer_storm_heavier_than_winter_rain() {
    use crate::weather::systems::precipitation_intensity_for_event;
    use crate::weather::types::WeatherCondition;

    // Using the same day_hash for fair comparison
    let hash = 50;
    let summer_storm =
        precipitation_intensity_for_event(WeatherCondition::Storm, Season::Summer, hash);
    let winter_rain =
        precipitation_intensity_for_event(WeatherCondition::Rain, Season::Winter, hash);

    assert!(
        summer_storm > winter_rain,
        "Summer storm ({}) should produce more precipitation than winter rain ({})",
        summer_storm,
        winter_rain
    );
}

// ---------------------------------------------------------------------------
// 4. Weather state serialization round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_weather_state_serialization_roundtrip() {
    // Create a Weather resource with non-default values and verify
    // it survives a JSON serialization round-trip.
    use crate::weather::types::WeatherCondition;

    let original = Weather {
        season: Season::Winter,
        temperature: -5.5,
        current_event: WeatherCondition::Snow,
        event_days_remaining: 3,
        last_update_day: 300,
        disasters_enabled: false,
        humidity: 0.85,
        cloud_cover: 0.72,
        precipitation_intensity: 0.15,
        atmo_precipitation: 0.45,
        last_update_hour: 14,
        prev_extreme: true,
        daily_rainfall: 1.2,
        rolling_30day_rainfall: 8.5,
        rainfall_history: {
            let mut h = vec![0.0; 30];
            h[29] = 1.5;
            h[28] = 0.8;
            h[0] = 0.3;
            h
        },
    };

    let json = serde_json::to_string(&original).expect("Weather should serialize to JSON");
    let deserialized: Weather =
        serde_json::from_str(&json).expect("Weather should deserialize from JSON");

    assert_eq!(deserialized.season, original.season);
    assert!(
        (deserialized.temperature - original.temperature).abs() < f32::EPSILON,
        "temperature mismatch"
    );
    assert_eq!(deserialized.current_event, original.current_event);
    assert_eq!(
        deserialized.event_days_remaining,
        original.event_days_remaining
    );
    assert_eq!(deserialized.last_update_day, original.last_update_day);
    assert_eq!(deserialized.disasters_enabled, original.disasters_enabled);
    assert!(
        (deserialized.humidity - original.humidity).abs() < f32::EPSILON,
        "humidity mismatch"
    );
    assert!(
        (deserialized.cloud_cover - original.cloud_cover).abs() < f32::EPSILON,
        "cloud_cover mismatch"
    );
    assert!(
        (deserialized.precipitation_intensity - original.precipitation_intensity).abs()
            < f32::EPSILON,
        "precipitation_intensity mismatch"
    );
    assert!(
        (deserialized.atmo_precipitation - original.atmo_precipitation).abs() < f32::EPSILON,
        "atmo_precipitation mismatch"
    );
    assert_eq!(deserialized.last_update_hour, original.last_update_hour);
    assert_eq!(deserialized.prev_extreme, original.prev_extreme);
    assert!(
        (deserialized.daily_rainfall - original.daily_rainfall).abs() < f32::EPSILON,
        "daily_rainfall mismatch"
    );
    assert!(
        (deserialized.rolling_30day_rainfall - original.rolling_30day_rainfall).abs()
            < f32::EPSILON,
        "rolling_30day_rainfall mismatch"
    );
    assert_eq!(
        deserialized.rainfall_history.len(),
        original.rainfall_history.len(),
        "rainfall_history length mismatch"
    );
    for (i, (d, o)) in deserialized
        .rainfall_history
        .iter()
        .zip(original.rainfall_history.iter())
        .enumerate()
    {
        assert!(
            (d - o).abs() < f32::EPSILON,
            "rainfall_history[{}] mismatch: {} vs {}",
            i,
            d,
            o
        );
    }
}

#[test]
fn test_weather_deserialization_with_missing_optional_fields() {
    // Simulate loading a save file from before the precipitation fields were added.
    // The #[serde(default)] annotations should fill in sensible defaults.
    let legacy_json = r#"{
        "season": "Summer",
        "temperature": 28.0,
        "current_event": "Sunny",
        "event_days_remaining": 0,
        "last_update_day": 100,
        "disasters_enabled": true
    }"#;

    let weather: Weather =
        serde_json::from_str(legacy_json).expect("Should deserialize legacy save data");

    assert_eq!(weather.season, Season::Summer);
    assert!((weather.temperature - 28.0).abs() < f32::EPSILON);
    // Default values for fields missing from old save
    assert_eq!(weather.cloud_cover, 0.0);
    assert_eq!(weather.precipitation_intensity, 0.0);
    assert_eq!(weather.atmo_precipitation, 0.0);
    assert_eq!(weather.last_update_hour, 0);
    assert!(!weather.prev_extreme);
    assert_eq!(weather.daily_rainfall, 0.0);
    assert_eq!(weather.rolling_30day_rainfall, 0.0);
    assert_eq!(weather.rainfall_history.len(), 30);
    // humidity has a custom default of 0.5
    assert!((weather.humidity - 0.5).abs() < f32::EPSILON);
}

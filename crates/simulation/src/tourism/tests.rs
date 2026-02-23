use crate::services::ServiceType;
use crate::weather::{Season, Weather, WeatherCondition};

use super::attraction_formula::*;
use super::*;

/// Helper to build a Weather with a given season, condition, and temperature.
fn make_weather(season: Season, condition: WeatherCondition, temperature: f32) -> Weather {
    Weather {
        season,
        current_event: condition,
        temperature,
        ..Default::default()
    }
}

// -------------------------------------------------------------------
// Seasonal multiplier tests
// -------------------------------------------------------------------

#[test]
fn test_seasonal_multipliers() {
    assert!((seasonal_tourism_multiplier(Season::Spring) - 1.2).abs() < f32::EPSILON);
    assert!((seasonal_tourism_multiplier(Season::Summer) - 1.5).abs() < f32::EPSILON);
    assert!((seasonal_tourism_multiplier(Season::Autumn) - 1.1).abs() < f32::EPSILON);
    assert!((seasonal_tourism_multiplier(Season::Winter) - 0.6).abs() < f32::EPSILON);
}

// -------------------------------------------------------------------
// Weather condition multiplier tests
// -------------------------------------------------------------------

#[test]
fn test_weather_multipliers() {
    assert!((weather_tourism_multiplier(WeatherCondition::Sunny) - 1.2).abs() < f32::EPSILON);
    assert!(
        (weather_tourism_multiplier(WeatherCondition::PartlyCloudy) - 1.0).abs() < f32::EPSILON
    );
    assert!((weather_tourism_multiplier(WeatherCondition::Overcast) - 0.8).abs() < f32::EPSILON);
    assert!((weather_tourism_multiplier(WeatherCondition::Rain) - 0.5).abs() < f32::EPSILON);
    assert!((weather_tourism_multiplier(WeatherCondition::HeavyRain) - 0.5).abs() < f32::EPSILON);
    assert!((weather_tourism_multiplier(WeatherCondition::Snow) - 0.7).abs() < f32::EPSILON);
    assert!((weather_tourism_multiplier(WeatherCondition::Storm) - 0.2).abs() < f32::EPSILON);
}

// -------------------------------------------------------------------
// Combined modifier tests
// -------------------------------------------------------------------

#[test]
fn test_summer_sunny_combined_modifier() {
    let w = make_weather(Season::Summer, WeatherCondition::Sunny, 25.0);
    let modifier = tourism_seasonal_modifier(Season::Summer, &w);
    assert!((modifier - 1.8).abs() < 0.001);
}

#[test]
fn test_winter_storm_combined_modifier() {
    let w = make_weather(Season::Winter, WeatherCondition::Storm, 2.0);
    let modifier = tourism_seasonal_modifier(Season::Winter, &w);
    assert!((modifier - 0.12).abs() < 0.001);
}

#[test]
fn test_spring_rain_combined_modifier() {
    let w = make_weather(Season::Spring, WeatherCondition::Rain, 10.0);
    let modifier = tourism_seasonal_modifier(Season::Spring, &w);
    assert!((modifier - 0.6).abs() < 0.001);
}

#[test]
fn test_autumn_overcast_combined_modifier() {
    let w = make_weather(Season::Autumn, WeatherCondition::Overcast, 12.0);
    let modifier = tourism_seasonal_modifier(Season::Autumn, &w);
    assert!((modifier - 0.88).abs() < 0.001);
}

#[test]
fn test_extreme_heat_applies_01_weather_multiplier() {
    let w = make_weather(Season::Summer, WeatherCondition::Sunny, 40.0);
    let modifier = tourism_seasonal_modifier(Season::Summer, &w);
    assert!((modifier - 0.15).abs() < 0.001);
}

#[test]
fn test_extreme_cold_applies_01_weather_multiplier() {
    let w = make_weather(Season::Winter, WeatherCondition::Snow, -10.0);
    let modifier = tourism_seasonal_modifier(Season::Winter, &w);
    assert!((modifier - 0.06).abs() < 0.001);
}

#[test]
fn test_summer_tourism_higher_than_winter() {
    let summer_w = make_weather(Season::Summer, WeatherCondition::PartlyCloudy, 25.0);
    let winter_w = make_weather(Season::Winter, WeatherCondition::PartlyCloudy, 2.0);
    let summer_mod = tourism_seasonal_modifier(Season::Summer, &summer_w);
    let winter_mod = tourism_seasonal_modifier(Season::Winter, &winter_w);
    assert!(summer_mod > winter_mod);
}

// -------------------------------------------------------------------
// Weather event tests
// -------------------------------------------------------------------

#[test]
fn test_festival_on_sunny_summer() {
    let w = make_weather(Season::Summer, WeatherCondition::Sunny, 25.0);
    assert_eq!(
        tourism_weather_event(Season::Summer, &w),
        Some(TourismWeatherEvent::Festival)
    );
}

#[test]
fn test_festival_on_sunny_spring() {
    let w = make_weather(Season::Spring, WeatherCondition::Sunny, 18.0);
    assert_eq!(
        tourism_weather_event(Season::Spring, &w),
        Some(TourismWeatherEvent::Festival)
    );
}

#[test]
fn test_no_festival_on_sunny_autumn() {
    let w = make_weather(Season::Autumn, WeatherCondition::Sunny, 15.0);
    assert_eq!(tourism_weather_event(Season::Autumn, &w), None);
}

#[test]
fn test_closure_on_storm() {
    let w = make_weather(Season::Summer, WeatherCondition::Storm, 20.0);
    assert_eq!(
        tourism_weather_event(Season::Summer, &w),
        Some(TourismWeatherEvent::Closure)
    );
}

#[test]
fn test_closure_on_extreme_heat() {
    let w = make_weather(Season::Summer, WeatherCondition::Sunny, 40.0);
    assert_eq!(
        tourism_weather_event(Season::Summer, &w),
        Some(TourismWeatherEvent::Closure)
    );
}

#[test]
fn test_closure_on_extreme_cold() {
    let w = make_weather(Season::Winter, WeatherCondition::Snow, -10.0);
    assert_eq!(
        tourism_weather_event(Season::Winter, &w),
        Some(TourismWeatherEvent::Closure)
    );
}

#[test]
fn test_no_event_on_mild_overcast() {
    let w = make_weather(Season::Autumn, WeatherCondition::Overcast, 12.0);
    assert_eq!(tourism_weather_event(Season::Autumn, &w), None);
}

// -------------------------------------------------------------------
// Tourism default state tests
// -------------------------------------------------------------------

#[test]
fn test_tourism_default_attractiveness_is_zero() {
    let t = Tourism::default();
    assert!((t.attractiveness - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_tourism_default_monthly_visitors_is_zero() {
    assert_eq!(Tourism::default().monthly_visitors, 0);
}

#[test]
fn test_tourism_default_monthly_income_is_zero() {
    assert!((Tourism::default().monthly_tourism_income - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_tourism_default_airport_multiplier_is_one() {
    assert!((Tourism::default().airport_multiplier - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_tourism_default_stay_days() {
    assert!((Tourism::default().average_stay_days - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_tourism_default_commercial_spending() {
    assert!((Tourism::default().commercial_spending - 0.0).abs() < f64::EPSILON);
}

// -------------------------------------------------------------------
// Tourism draw per service type tests
// -------------------------------------------------------------------

#[test]
fn test_tourism_draw_values() {
    assert_eq!(Tourism::tourism_draw(ServiceType::Stadium), 500);
    assert_eq!(Tourism::tourism_draw(ServiceType::Museum), 300);
    assert_eq!(Tourism::tourism_draw(ServiceType::Cathedral), 200);
    assert_eq!(Tourism::tourism_draw(ServiceType::CityHall), 100);
    assert_eq!(Tourism::tourism_draw(ServiceType::TVStation), 150);
    assert_eq!(Tourism::tourism_draw(ServiceType::LargePark), 100);
    assert_eq!(Tourism::tourism_draw(ServiceType::SportsField), 50);
    assert_eq!(Tourism::tourism_draw(ServiceType::Plaza), 80);
}

#[test]
fn test_tourism_draw_non_tourist_services_return_zero() {
    assert_eq!(Tourism::tourism_draw(ServiceType::FireStation), 0);
    assert_eq!(Tourism::tourism_draw(ServiceType::PoliceStation), 0);
    assert_eq!(Tourism::tourism_draw(ServiceType::Hospital), 0);
    assert_eq!(Tourism::tourism_draw(ServiceType::ElementarySchool), 0);
    assert_eq!(Tourism::tourism_draw(ServiceType::Landfill), 0);
    assert_eq!(Tourism::tourism_draw(ServiceType::Cemetery), 0);
    assert_eq!(Tourism::tourism_draw(ServiceType::Library), 0);
}

#[test]
fn test_tourism_draw_ranking() {
    assert!(
        Tourism::tourism_draw(ServiceType::Stadium) > Tourism::tourism_draw(ServiceType::Museum)
    );
    assert!(
        Tourism::tourism_draw(ServiceType::Museum) > Tourism::tourism_draw(ServiceType::Cathedral)
    );
    assert!(
        Tourism::tourism_draw(ServiceType::Cathedral)
            > Tourism::tourism_draw(ServiceType::TVStation)
    );
}

// -------------------------------------------------------------------
// Attraction formula component tests
// -------------------------------------------------------------------

#[test]
fn test_attraction_breakdown_all_hundred() {
    let b = AttractionBreakdown {
        cultural_facilities: 100.0,
        natural_beauty: 100.0,
        hotel_capacity: 100.0,
        transport_access: 100.0,
        safety: 100.0,
        entertainment: 100.0,
    };
    assert!((b.total() - 100.0).abs() < 0.01);
}

#[test]
fn test_cultural_only_gives_thirty_percent() {
    let b = AttractionBreakdown {
        cultural_facilities: 100.0,
        ..Default::default()
    };
    assert!((b.total() - 30.0).abs() < 0.01);
}

#[test]
fn test_safety_only_gives_ten_percent() {
    let b = AttractionBreakdown {
        safety: 100.0,
        ..Default::default()
    };
    assert!((b.total() - 10.0).abs() < 0.01);
}

#[test]
fn test_breakdown_from_tourism_resource() {
    let mut t = Tourism::default();
    t.cultural_facilities_score = 80.0;
    t.natural_beauty_score = 60.0;
    let b = t.breakdown();
    assert!((b.cultural_facilities - 80.0).abs() < f32::EPSILON);
    assert!(b.total() > 0.0);
}

// -------------------------------------------------------------------
// Extreme weather boundary tests
// -------------------------------------------------------------------

#[test]
fn test_extreme_heat_boundary_exactly_35_not_extreme() {
    let w = make_weather(Season::Summer, WeatherCondition::Sunny, 35.0);
    let modifier = tourism_seasonal_modifier(Season::Summer, &w);
    assert!((modifier - 1.8).abs() < 0.001);
}

#[test]
fn test_extreme_heat_boundary_just_above_35() {
    let w = make_weather(Season::Summer, WeatherCondition::Sunny, 35.1);
    let modifier = tourism_seasonal_modifier(Season::Summer, &w);
    assert!((modifier - 0.15).abs() < 0.001);
}

#[test]
fn test_extreme_cold_boundary_exactly_minus_5_not_extreme() {
    let w = make_weather(Season::Winter, WeatherCondition::Snow, -5.0);
    let modifier = tourism_seasonal_modifier(Season::Winter, &w);
    assert!((modifier - 0.42).abs() < 0.001);
}

#[test]
fn test_extreme_cold_boundary_just_below_minus_5() {
    let w = make_weather(Season::Winter, WeatherCondition::Snow, -5.1);
    let modifier = tourism_seasonal_modifier(Season::Winter, &w);
    assert!((modifier - 0.06).abs() < 0.001);
}

// -------------------------------------------------------------------
// Weather event boundary tests
// -------------------------------------------------------------------

#[test]
fn test_event_extreme_heat_35_no_closure() {
    let w = make_weather(Season::Summer, WeatherCondition::Sunny, 35.0);
    assert_eq!(
        tourism_weather_event(Season::Summer, &w),
        Some(TourismWeatherEvent::Festival)
    );
}

#[test]
fn test_event_extreme_heat_above_35_closure() {
    let w = make_weather(Season::Summer, WeatherCondition::Sunny, 35.1);
    assert_eq!(
        tourism_weather_event(Season::Summer, &w),
        Some(TourismWeatherEvent::Closure)
    );
}

#[test]
fn test_event_extreme_cold_minus_5_no_closure() {
    let w = make_weather(Season::Winter, WeatherCondition::Snow, -5.0);
    assert_eq!(tourism_weather_event(Season::Winter, &w), None);
}

#[test]
fn test_event_extreme_cold_below_minus_5_closure() {
    let w = make_weather(Season::Winter, WeatherCondition::Snow, -5.1);
    assert_eq!(
        tourism_weather_event(Season::Winter, &w),
        Some(TourismWeatherEvent::Closure)
    );
}

#[test]
fn test_no_festival_on_sunny_winter() {
    let w = make_weather(Season::Winter, WeatherCondition::Sunny, 5.0);
    assert_eq!(tourism_weather_event(Season::Winter, &w), None);
}

#[test]
fn test_no_event_on_rain() {
    let w = make_weather(Season::Summer, WeatherCondition::Rain, 20.0);
    assert_eq!(tourism_weather_event(Season::Summer, &w), None);
}

// -------------------------------------------------------------------
// Saveable tests
// -------------------------------------------------------------------

#[test]
fn test_tourism_saveable_key() {
    use crate::Saveable;
    assert_eq!(Tourism::SAVE_KEY, "tourism");
}

#[test]
fn test_tourism_saveable_roundtrip() {
    use crate::Saveable;
    let mut t = Tourism::default();
    t.attractiveness = 75.0;
    t.monthly_visitors = 1234;
    t.cultural_facilities_score = 60.0;
    t.safety_score = 85.0;
    let bytes = t.save_to_bytes().unwrap();
    let restored = Tourism::load_from_bytes(&bytes);
    assert!((restored.attractiveness - 75.0).abs() < 0.01);
    assert_eq!(restored.monthly_visitors, 1234);
    assert!((restored.cultural_facilities_score - 60.0).abs() < 0.01);
    assert!((restored.safety_score - 85.0).abs() < 0.01);
}

#[test]
fn test_tourism_saveable_skip_empty() {
    use crate::Saveable;
    let t = Tourism::default();
    assert!(t.save_to_bytes().is_none());
}

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::services::{ServiceBuilding, ServiceType};
use crate::stats::CityStats;
use crate::weather::{Season, Weather, WeatherCondition};

/// Tourism tracking
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Tourism {
    pub attractiveness: f32, // 0-100 score
    pub monthly_visitors: u32,
    pub monthly_tourism_income: f64,
    pub last_update_day: u32,
    /// Multiplier from airport system (1.0 = no airports, >1.0 = airports boost tourism).
    pub airport_multiplier: f32,
}

impl Default for Tourism {
    fn default() -> Self {
        Self {
            attractiveness: 0.0,
            monthly_visitors: 0,
            monthly_tourism_income: 0.0,
            last_update_day: 0,
            airport_multiplier: 1.0,
        }
    }
}

impl Tourism {
    /// How many tourists a service type attracts per month
    fn tourism_draw(service_type: ServiceType) -> u32 {
        match service_type {
            ServiceType::Stadium => 500,
            ServiceType::Museum => 300,
            ServiceType::Cathedral => 200,
            ServiceType::CityHall => 100,
            ServiceType::TVStation => 150,
            ServiceType::LargePark => 100,
            ServiceType::SportsField => 50,
            ServiceType::Plaza => 80,
            _ => 0,
        }
    }
}

/// Seasonal base multiplier for tourism arrivals.
///
/// Summer is peak tourism season; winter is the low season.
pub fn seasonal_tourism_multiplier(season: Season) -> f32 {
    match season {
        Season::Spring => 1.2,
        Season::Summer => 1.5,
        Season::Autumn => 1.1,
        Season::Winter => 0.6,
    }
}

/// Weather condition multiplier for tourism arrivals.
///
/// Good weather encourages tourism; storms and extreme conditions suppress it.
pub fn weather_tourism_multiplier(condition: WeatherCondition) -> f32 {
    match condition {
        WeatherCondition::Sunny => 1.2,
        WeatherCondition::PartlyCloudy => 1.0,
        WeatherCondition::Overcast => 0.8,
        WeatherCondition::Rain => 0.5,
        WeatherCondition::HeavyRain => 0.5,
        WeatherCondition::Snow => 0.7,
        WeatherCondition::Storm => 0.2,
    }
}

/// Combined seasonal and weather tourism modifier.
///
/// Returns `seasonal_tourism_multiplier(season) * weather_tourism_multiplier(condition)`.
/// An extreme temperature (heat wave > 35C or cold snap < -5C) applies an additional 0.1x
/// penalty, matching the `Extreme=0.1` requirement from the spec.
pub fn tourism_seasonal_modifier(season: Season, weather: &Weather) -> f32 {
    let season_mult = seasonal_tourism_multiplier(season);
    let weather_mult = if weather.temperature > 35.0 || weather.temperature < -5.0 {
        // Extreme weather overrides the condition-based multiplier
        0.1
    } else {
        weather_tourism_multiplier(weather.current_event)
    };
    season_mult * weather_mult
}

/// Tourism events that can occur based on weather conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TourismWeatherEvent {
    /// Good-weather festival: occurs on Sunny days in Spring/Summer.
    Festival,
    /// Weather closure: occurs during Storm or extreme conditions.
    Closure,
}

/// Determine if a weather-related tourism event should occur.
///
/// Returns `Some(Festival)` for sunny Spring/Summer days and `Some(Closure)` for storms
/// or extreme temperature conditions.
pub fn tourism_weather_event(season: Season, weather: &Weather) -> Option<TourismWeatherEvent> {
    // Closures: storm or extreme temperatures
    if weather.current_event == WeatherCondition::Storm
        || weather.temperature > 35.0
        || weather.temperature < -5.0
    {
        return Some(TourismWeatherEvent::Closure);
    }
    // Festivals: sunny days in peak seasons
    if weather.current_event == WeatherCondition::Sunny
        && (season == Season::Spring || season == Season::Summer)
    {
        return Some(TourismWeatherEvent::Festival);
    }
    None
}

pub fn update_tourism(
    clock: Res<crate::time_of_day::GameClock>,
    mut tourism: ResMut<Tourism>,
    services: Query<&ServiceBuilding>,
    stats: Res<CityStats>,
    weather: Res<Weather>,
) {
    // Update monthly
    if clock.day <= tourism.last_update_day + 30 {
        return;
    }
    tourism.last_update_day = clock.day;

    // Calculate attractiveness from landmarks and entertainment
    let mut total_draw = 0u32;
    for service in &services {
        total_draw += Tourism::tourism_draw(service.service_type);
    }

    // Attractiveness scales with city size and landmarks
    let pop_factor = (stats.population as f32 / 10000.0).min(5.0);
    tourism.attractiveness = (total_draw as f32 * 0.1 + pop_factor * 10.0).min(100.0);

    // Visitors based on attractiveness, boosted by airport and weather/season modifiers
    let base_visitors = (tourism.attractiveness * 50.0) as u32;
    let season_weather_modifier = tourism_seasonal_modifier(weather.season, &weather);
    tourism.monthly_visitors =
        (base_visitors as f32 * tourism.airport_multiplier * season_weather_modifier) as u32;

    // Tourism income: visitors spend money at commercial buildings
    // Airport multiplier also boosts per-visitor spending (international travelers spend more)
    let spending_per_visitor = 2.0 * tourism.airport_multiplier as f64;
    tourism.monthly_tourism_income = tourism.monthly_visitors as f64 * spending_per_visitor;
}

#[cfg(test)]
mod tests {
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
        assert!(
            (weather_tourism_multiplier(WeatherCondition::Overcast) - 0.8).abs() < f32::EPSILON
        );
        assert!((weather_tourism_multiplier(WeatherCondition::Rain) - 0.5).abs() < f32::EPSILON);
        assert!(
            (weather_tourism_multiplier(WeatherCondition::HeavyRain) - 0.5).abs() < f32::EPSILON
        );
        assert!((weather_tourism_multiplier(WeatherCondition::Snow) - 0.7).abs() < f32::EPSILON);
        assert!((weather_tourism_multiplier(WeatherCondition::Storm) - 0.2).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------
    // Combined modifier tests (from issue spec)
    // -------------------------------------------------------------------

    #[test]
    fn test_summer_sunny_combined_modifier() {
        // Summer (1.5) * Sunny (1.2) = 1.8
        let w = make_weather(Season::Summer, WeatherCondition::Sunny, 25.0);
        let modifier = tourism_seasonal_modifier(Season::Summer, &w);
        assert!(
            (modifier - 1.8).abs() < 0.001,
            "Summer + Sunny should be 1.8, got {}",
            modifier
        );
    }

    #[test]
    fn test_winter_storm_combined_modifier() {
        // Winter (0.6) * Storm (0.2) = 0.12
        let w = make_weather(Season::Winter, WeatherCondition::Storm, 2.0);
        let modifier = tourism_seasonal_modifier(Season::Winter, &w);
        assert!(
            (modifier - 0.12).abs() < 0.001,
            "Winter + Storm should be 0.12, got {}",
            modifier
        );
    }

    #[test]
    fn test_spring_rain_combined_modifier() {
        // Spring (1.2) * Rain (0.5) = 0.6
        let w = make_weather(Season::Spring, WeatherCondition::Rain, 10.0);
        let modifier = tourism_seasonal_modifier(Season::Spring, &w);
        assert!(
            (modifier - 0.6).abs() < 0.001,
            "Spring + Rain should be 0.6, got {}",
            modifier
        );
    }

    #[test]
    fn test_autumn_overcast_combined_modifier() {
        // Autumn (1.1) * Overcast (0.8) = 0.88
        let w = make_weather(Season::Autumn, WeatherCondition::Overcast, 12.0);
        let modifier = tourism_seasonal_modifier(Season::Autumn, &w);
        assert!(
            (modifier - 0.88).abs() < 0.001,
            "Autumn + Overcast should be 0.88, got {}",
            modifier
        );
    }

    #[test]
    fn test_extreme_heat_applies_01_weather_multiplier() {
        // Extreme heat (>35C): weather multiplier becomes 0.1
        // Summer (1.5) * Extreme (0.1) = 0.15
        let w = make_weather(Season::Summer, WeatherCondition::Sunny, 40.0);
        let modifier = tourism_seasonal_modifier(Season::Summer, &w);
        assert!(
            (modifier - 0.15).abs() < 0.001,
            "Summer + extreme heat should be 0.15, got {}",
            modifier
        );
    }

    #[test]
    fn test_extreme_cold_applies_01_weather_multiplier() {
        // Extreme cold (<-5C): weather multiplier becomes 0.1
        // Winter (0.6) * Extreme (0.1) = 0.06
        let w = make_weather(Season::Winter, WeatherCondition::Snow, -10.0);
        let modifier = tourism_seasonal_modifier(Season::Winter, &w);
        assert!(
            (modifier - 0.06).abs() < 0.001,
            "Winter + extreme cold should be 0.06, got {}",
            modifier
        );
    }

    // -------------------------------------------------------------------
    // Tourism revenue seasonal integration test
    // -------------------------------------------------------------------

    #[test]
    fn test_summer_tourism_higher_than_winter() {
        // With same conditions (PartlyCloudy, mild temp), summer should beat winter
        let summer_w = make_weather(Season::Summer, WeatherCondition::PartlyCloudy, 25.0);
        let winter_w = make_weather(Season::Winter, WeatherCondition::PartlyCloudy, 2.0);
        let summer_mod = tourism_seasonal_modifier(Season::Summer, &summer_w);
        let winter_mod = tourism_seasonal_modifier(Season::Winter, &winter_w);
        assert!(
            summer_mod > winter_mod,
            "Summer modifier ({}) should exceed winter ({})",
            summer_mod,
            winter_mod
        );
    }

    // -------------------------------------------------------------------
    // Tourism weather event tests
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
        let tourism = Tourism::default();
        assert!(
            (tourism.attractiveness - 0.0).abs() < f32::EPSILON,
            "Default attractiveness should be 0.0, got {}",
            tourism.attractiveness
        );
    }

    #[test]
    fn test_tourism_default_monthly_visitors_is_zero() {
        let tourism = Tourism::default();
        assert_eq!(
            tourism.monthly_visitors, 0,
            "Default monthly visitors should be 0"
        );
    }

    #[test]
    fn test_tourism_default_monthly_income_is_zero() {
        let tourism = Tourism::default();
        assert!(
            (tourism.monthly_tourism_income - 0.0).abs() < f64::EPSILON,
            "Default monthly tourism income should be 0.0, got {}",
            tourism.monthly_tourism_income
        );
    }

    #[test]
    fn test_tourism_default_airport_multiplier_is_one() {
        let tourism = Tourism::default();
        assert!(
            (tourism.airport_multiplier - 1.0).abs() < f32::EPSILON,
            "Default airport multiplier should be 1.0, got {}",
            tourism.airport_multiplier
        );
    }

    #[test]
    fn test_tourism_default_last_update_day_is_zero() {
        let tourism = Tourism::default();
        assert_eq!(
            tourism.last_update_day, 0,
            "Default last_update_day should be 0"
        );
    }

    // -------------------------------------------------------------------
    // Tourism draw per service type tests
    // -------------------------------------------------------------------

    #[test]
    fn test_tourism_draw_stadium() {
        assert_eq!(Tourism::tourism_draw(ServiceType::Stadium), 500);
    }

    #[test]
    fn test_tourism_draw_museum() {
        assert_eq!(Tourism::tourism_draw(ServiceType::Museum), 300);
    }

    #[test]
    fn test_tourism_draw_cathedral() {
        assert_eq!(Tourism::tourism_draw(ServiceType::Cathedral), 200);
    }

    #[test]
    fn test_tourism_draw_city_hall() {
        assert_eq!(Tourism::tourism_draw(ServiceType::CityHall), 100);
    }

    #[test]
    fn test_tourism_draw_tv_station() {
        assert_eq!(Tourism::tourism_draw(ServiceType::TVStation), 150);
    }

    #[test]
    fn test_tourism_draw_large_park() {
        assert_eq!(Tourism::tourism_draw(ServiceType::LargePark), 100);
    }

    #[test]
    fn test_tourism_draw_sports_field() {
        assert_eq!(Tourism::tourism_draw(ServiceType::SportsField), 50);
    }

    #[test]
    fn test_tourism_draw_plaza() {
        assert_eq!(Tourism::tourism_draw(ServiceType::Plaza), 80);
    }

    #[test]
    fn test_tourism_draw_non_tourist_services_return_zero() {
        // Services that are NOT tourist attractions should return 0
        assert_eq!(Tourism::tourism_draw(ServiceType::FireStation), 0);
        assert_eq!(Tourism::tourism_draw(ServiceType::PoliceStation), 0);
        assert_eq!(Tourism::tourism_draw(ServiceType::Hospital), 0);
        assert_eq!(Tourism::tourism_draw(ServiceType::ElementarySchool), 0);
        assert_eq!(Tourism::tourism_draw(ServiceType::Landfill), 0);
        assert_eq!(Tourism::tourism_draw(ServiceType::Cemetery), 0);
        assert_eq!(Tourism::tourism_draw(ServiceType::Library), 0);
    }

    // -------------------------------------------------------------------
    // Attractiveness calculation logic tests
    // -------------------------------------------------------------------

    #[test]
    fn test_attractiveness_formula_no_services_no_population() {
        // With 0 total_draw and 0 population:
        // pop_factor = (0 / 10000.0).min(5.0) = 0.0
        // attractiveness = (0 * 0.1 + 0.0 * 10.0).min(100.0) = 0.0
        let total_draw: u32 = 0;
        let population: u32 = 0;
        let pop_factor = (population as f32 / 10000.0).min(5.0);
        let attractiveness = (total_draw as f32 * 0.1 + pop_factor * 10.0).min(100.0);
        assert!(
            (attractiveness - 0.0).abs() < f32::EPSILON,
            "No services and no population should yield 0 attractiveness"
        );
    }

    #[test]
    fn test_attractiveness_formula_with_stadium() {
        // Stadium draws 500, no population:
        // attractiveness = (500 * 0.1 + 0.0 * 10.0).min(100.0) = 50.0
        let total_draw: u32 = 500;
        let population: u32 = 0;
        let pop_factor = (population as f32 / 10000.0).min(5.0);
        let attractiveness = (total_draw as f32 * 0.1 + pop_factor * 10.0).min(100.0);
        assert!(
            (attractiveness - 50.0).abs() < 0.01,
            "Stadium alone (no pop) should yield 50.0, got {}",
            attractiveness
        );
    }

    #[test]
    fn test_attractiveness_formula_with_population() {
        // No services, 10000 population:
        // pop_factor = (10000 / 10000.0).min(5.0) = 1.0
        // attractiveness = (0 * 0.1 + 1.0 * 10.0).min(100.0) = 10.0
        let total_draw: u32 = 0;
        let population: u32 = 10_000;
        let pop_factor = (population as f32 / 10000.0).min(5.0);
        let attractiveness = (total_draw as f32 * 0.1 + pop_factor * 10.0).min(100.0);
        assert!(
            (attractiveness - 10.0).abs() < 0.01,
            "10K pop with no services should yield 10.0, got {}",
            attractiveness
        );
    }

    #[test]
    fn test_attractiveness_capped_at_100() {
        // Very high draw + max population:
        // pop_factor = (50000 / 10000.0).min(5.0) = 5.0
        // attractiveness = (2000 * 0.1 + 5.0 * 10.0).min(100.0) = (200 + 50).min(100) = 100.0
        let total_draw: u32 = 2000;
        let population: u32 = 50_000;
        let pop_factor = (population as f32 / 10000.0).min(5.0);
        let attractiveness = (total_draw as f32 * 0.1 + pop_factor * 10.0).min(100.0);
        assert!(
            (attractiveness - 100.0).abs() < 0.01,
            "Attractiveness should cap at 100.0, got {}",
            attractiveness
        );
    }

    #[test]
    fn test_population_factor_capped_at_5() {
        // Ensure populations above 50K don't push pop_factor beyond 5.0
        let pop_factor_50k = (50_000f32 / 10000.0).min(5.0);
        let pop_factor_100k = (100_000f32 / 10000.0).min(5.0);
        assert!(
            (pop_factor_50k - pop_factor_100k).abs() < f32::EPSILON,
            "Pop factor should cap at 5.0 for both 50K and 100K"
        );
    }

    // -------------------------------------------------------------------
    // Visitor count proportional to attractiveness tests
    // -------------------------------------------------------------------

    #[test]
    fn test_visitors_proportional_to_attractiveness() {
        // base_visitors = (attractiveness * 50.0) as u32
        // With attractiveness=50 => base_visitors=2500
        let attractiveness: f32 = 50.0;
        let base_visitors = (attractiveness * 50.0) as u32;
        assert_eq!(
            base_visitors, 2500,
            "50 attractiveness should yield 2500 base visitors"
        );
    }

    #[test]
    fn test_visitors_zero_attractiveness() {
        let attractiveness: f32 = 0.0;
        let base_visitors = (attractiveness * 50.0) as u32;
        assert_eq!(
            base_visitors, 0,
            "0 attractiveness should yield 0 base visitors"
        );
    }

    #[test]
    fn test_visitors_max_attractiveness() {
        let attractiveness: f32 = 100.0;
        let base_visitors = (attractiveness * 50.0) as u32;
        assert_eq!(
            base_visitors, 5000,
            "100 attractiveness should yield 5000 base visitors"
        );
    }

    #[test]
    fn test_visitors_scaled_by_airport_multiplier() {
        let base_visitors: u32 = 2500;
        let airport_multiplier: f32 = 2.0;
        let w = make_weather(Season::Spring, WeatherCondition::PartlyCloudy, 15.0);
        let season_weather_mod = tourism_seasonal_modifier(Season::Spring, &w);
        // Spring (1.2) * PartlyCloudy (1.0) = 1.2
        let monthly_visitors =
            (base_visitors as f32 * airport_multiplier * season_weather_mod) as u32;
        // 2500 * 2.0 * 1.2 = 6000
        assert_eq!(
            monthly_visitors, 6000,
            "Visitors should scale with airport multiplier, got {}",
            monthly_visitors
        );
    }

    #[test]
    fn test_visitors_modulated_by_season_and_weather() {
        let base_visitors: u32 = 1000;
        let airport_multiplier: f32 = 1.0;

        // Summer + Sunny = 1.5 * 1.2 = 1.8
        let summer_w = make_weather(Season::Summer, WeatherCondition::Sunny, 25.0);
        let summer_visitors = (base_visitors as f32
            * airport_multiplier
            * tourism_seasonal_modifier(Season::Summer, &summer_w))
            as u32;

        // Winter + Storm = 0.6 * 0.2 = 0.12
        let winter_w = make_weather(Season::Winter, WeatherCondition::Storm, 2.0);
        let winter_visitors = (base_visitors as f32
            * airport_multiplier
            * tourism_seasonal_modifier(Season::Winter, &winter_w))
            as u32;

        assert!(
            summer_visitors > winter_visitors,
            "Summer sunny visitors ({}) should exceed winter storm visitors ({})",
            summer_visitors,
            winter_visitors
        );
        assert_eq!(summer_visitors, 1800);
        assert_eq!(winter_visitors, 120);
    }

    // -------------------------------------------------------------------
    // Tourism revenue tests
    // -------------------------------------------------------------------

    #[test]
    fn test_tourism_revenue_formula() {
        // spending_per_visitor = 2.0 * airport_multiplier
        // monthly_tourism_income = monthly_visitors * spending_per_visitor
        let monthly_visitors: u32 = 1000;
        let airport_multiplier: f32 = 1.0;
        let spending_per_visitor = 2.0 * airport_multiplier as f64;
        let income = monthly_visitors as f64 * spending_per_visitor;
        assert!(
            (income - 2000.0).abs() < f64::EPSILON,
            "1000 visitors at 2.0 per visitor should yield 2000.0 income, got {}",
            income
        );
    }

    #[test]
    fn test_tourism_revenue_with_airport_boost() {
        // airport_multiplier of 2.0 doubles per-visitor spending
        let monthly_visitors: u32 = 1000;
        let airport_multiplier: f32 = 2.0;
        let spending_per_visitor = 2.0 * airport_multiplier as f64;
        let income = monthly_visitors as f64 * spending_per_visitor;
        assert!(
            (income - 4000.0).abs() < f64::EPSILON,
            "Airport multiplier 2.0 should double revenue, got {}",
            income
        );
    }

    #[test]
    fn test_tourism_revenue_zero_visitors() {
        let monthly_visitors: u32 = 0;
        let airport_multiplier: f32 = 1.0;
        let spending_per_visitor = 2.0 * airport_multiplier as f64;
        let income = monthly_visitors as f64 * spending_per_visitor;
        assert!(
            (income - 0.0).abs() < f64::EPSILON,
            "Zero visitors should yield zero revenue"
        );
    }

    // -------------------------------------------------------------------
    // Extreme weather boundary tests
    // -------------------------------------------------------------------

    #[test]
    fn test_extreme_heat_boundary_exactly_35_not_extreme() {
        // Temperature exactly 35.0 should NOT trigger extreme (condition is > 35.0)
        let w = make_weather(Season::Summer, WeatherCondition::Sunny, 35.0);
        let modifier = tourism_seasonal_modifier(Season::Summer, &w);
        // Should use normal: Summer (1.5) * Sunny (1.2) = 1.8
        assert!(
            (modifier - 1.8).abs() < 0.001,
            "35.0 is not extreme heat, modifier should be 1.8, got {}",
            modifier
        );
    }

    #[test]
    fn test_extreme_heat_boundary_just_above_35() {
        // Temperature 35.1 should trigger extreme
        let w = make_weather(Season::Summer, WeatherCondition::Sunny, 35.1);
        let modifier = tourism_seasonal_modifier(Season::Summer, &w);
        // Extreme: Summer (1.5) * 0.1 = 0.15
        assert!(
            (modifier - 0.15).abs() < 0.001,
            "35.1 should trigger extreme, modifier should be 0.15, got {}",
            modifier
        );
    }

    #[test]
    fn test_extreme_cold_boundary_exactly_minus_5_not_extreme() {
        // Temperature exactly -5.0 should NOT trigger extreme (condition is < -5.0)
        let w = make_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        let modifier = tourism_seasonal_modifier(Season::Winter, &w);
        // Should use normal: Winter (0.6) * Snow (0.7) = 0.42
        assert!(
            (modifier - 0.42).abs() < 0.001,
            "-5.0 is not extreme cold, modifier should be 0.42, got {}",
            modifier
        );
    }

    #[test]
    fn test_extreme_cold_boundary_just_below_minus_5() {
        // Temperature -5.1 should trigger extreme
        let w = make_weather(Season::Winter, WeatherCondition::Snow, -5.1);
        let modifier = tourism_seasonal_modifier(Season::Winter, &w);
        // Extreme: Winter (0.6) * 0.1 = 0.06
        assert!(
            (modifier - 0.06).abs() < 0.001,
            "-5.1 should trigger extreme, modifier should be 0.06, got {}",
            modifier
        );
    }

    // -------------------------------------------------------------------
    // Weather event boundary tests
    // -------------------------------------------------------------------

    #[test]
    fn test_event_extreme_heat_boundary_35_no_closure() {
        // Temperature exactly 35.0 should NOT trigger closure
        let w = make_weather(Season::Summer, WeatherCondition::Sunny, 35.0);
        // It's sunny summer, so it should be a Festival
        assert_eq!(
            tourism_weather_event(Season::Summer, &w),
            Some(TourismWeatherEvent::Festival)
        );
    }

    #[test]
    fn test_event_extreme_heat_boundary_above_35_closure() {
        let w = make_weather(Season::Summer, WeatherCondition::Sunny, 35.1);
        assert_eq!(
            tourism_weather_event(Season::Summer, &w),
            Some(TourismWeatherEvent::Closure)
        );
    }

    #[test]
    fn test_event_extreme_cold_boundary_minus_5_no_closure() {
        let w = make_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        // Not extreme, not sunny in spring/summer, so None
        assert_eq!(tourism_weather_event(Season::Winter, &w), None);
    }

    #[test]
    fn test_event_extreme_cold_boundary_below_minus_5_closure() {
        let w = make_weather(Season::Winter, WeatherCondition::Snow, -5.1);
        assert_eq!(
            tourism_weather_event(Season::Winter, &w),
            Some(TourismWeatherEvent::Closure)
        );
    }

    #[test]
    fn test_no_festival_on_sunny_winter() {
        let w = make_weather(Season::Winter, WeatherCondition::Sunny, 5.0);
        assert_eq!(
            tourism_weather_event(Season::Winter, &w),
            None,
            "Festivals only occur in Spring and Summer"
        );
    }

    #[test]
    fn test_no_event_on_rain() {
        let w = make_weather(Season::Summer, WeatherCondition::Rain, 20.0);
        assert_eq!(
            tourism_weather_event(Season::Summer, &w),
            None,
            "Rain alone should not trigger any event"
        );
    }

    // -------------------------------------------------------------------
    // Tourism draw ordering / ranking tests
    // -------------------------------------------------------------------

    #[test]
    fn test_tourism_draw_ranking() {
        // Stadium > Museum > Cathedral > TVStation > CityHall >= LargePark > Plaza > SportsField
        assert!(
            Tourism::tourism_draw(ServiceType::Stadium)
                > Tourism::tourism_draw(ServiceType::Museum)
        );
        assert!(
            Tourism::tourism_draw(ServiceType::Museum)
                > Tourism::tourism_draw(ServiceType::Cathedral)
        );
        assert!(
            Tourism::tourism_draw(ServiceType::Cathedral)
                > Tourism::tourism_draw(ServiceType::TVStation)
        );
        assert!(
            Tourism::tourism_draw(ServiceType::TVStation)
                > Tourism::tourism_draw(ServiceType::CityHall)
        );
        assert!(
            Tourism::tourism_draw(ServiceType::CityHall)
                >= Tourism::tourism_draw(ServiceType::LargePark)
        );
        assert!(
            Tourism::tourism_draw(ServiceType::LargePark)
                > Tourism::tourism_draw(ServiceType::Plaza)
        );
        assert!(
            Tourism::tourism_draw(ServiceType::Plaza)
                > Tourism::tourism_draw(ServiceType::SportsField)
        );
    }

    // -------------------------------------------------------------------
    // Combined attractiveness + visitors + revenue end-to-end test
    // -------------------------------------------------------------------

    #[test]
    fn test_end_to_end_tourism_calculation() {
        // Simulate what update_tourism does with known inputs:
        // - 1 Stadium (500 draw) + 1 Museum (300 draw) = 800 total draw
        // - Population = 20_000
        // - Airport multiplier = 1.5
        // - Season: Summer, Weather: Sunny, Temp: 25.0
        let total_draw: u32 = 800; // Stadium + Museum
        let population: u32 = 20_000;
        let airport_multiplier: f32 = 1.5;

        // Attractiveness
        let pop_factor = (population as f32 / 10000.0).min(5.0); // 2.0
        let attractiveness = (total_draw as f32 * 0.1 + pop_factor * 10.0).min(100.0);
        // = (80.0 + 20.0).min(100.0) = 100.0
        assert!(
            (attractiveness - 100.0).abs() < 0.01,
            "Expected attractiveness 100.0, got {}",
            attractiveness
        );

        // Visitors
        let base_visitors = (attractiveness * 50.0) as u32; // 5000
        let w = make_weather(Season::Summer, WeatherCondition::Sunny, 25.0);
        let season_weather_mod = tourism_seasonal_modifier(Season::Summer, &w); // 1.8
        let monthly_visitors =
            (base_visitors as f32 * airport_multiplier * season_weather_mod) as u32;
        // 5000 * 1.5 * 1.8 = 13500
        assert_eq!(
            monthly_visitors, 13500,
            "Expected 13500 visitors, got {}",
            monthly_visitors
        );

        // Revenue
        let spending_per_visitor = 2.0 * airport_multiplier as f64; // 3.0
        let income = monthly_visitors as f64 * spending_per_visitor;
        // 13500 * 3.0 = 40500.0
        assert!(
            (income - 40500.0).abs() < 0.01,
            "Expected 40500.0 income, got {}",
            income
        );
    }
}

pub struct TourismPlugin;

impl Plugin for TourismPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tourism>().add_systems(
            FixedUpdate,
            update_tourism
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

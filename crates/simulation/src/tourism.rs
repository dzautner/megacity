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
}

pub struct TourismPlugin;

impl Plugin for TourismPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tourism>()
            .add_systems(
                FixedUpdate,
                update_tourism.after(crate::imports_exports::process_trade),
            );
    }
}

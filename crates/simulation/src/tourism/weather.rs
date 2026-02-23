use crate::weather::{Season, Weather, WeatherCondition};

use super::TourismWeatherEvent;

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

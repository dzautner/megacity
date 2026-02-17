use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::time_of_day::GameClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn from_day(day: u32) -> Season {
        let day_of_year = day % 365;
        match day_of_year {
            0..=90 => Season::Spring,
            91..=181 => Season::Summer,
            182..=272 => Season::Autumn,
            _ => Season::Winter,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherEvent {
    Clear,
    Rain,
    HeatWave,
    ColdSnap,
    Storm,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Weather {
    pub season: Season,
    pub temperature: f32,         // -10 to 40 Celsius
    pub current_event: WeatherEvent,
    pub event_days_remaining: u32,
    pub last_update_day: u32,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            season: Season::Spring,
            temperature: 15.0,
            current_event: WeatherEvent::Clear,
            event_days_remaining: 0,
            last_update_day: 0,
        }
    }
}

impl Weather {
    /// Base temperature for each season
    fn base_temperature(season: Season) -> f32 {
        match season {
            Season::Spring => 15.0,
            Season::Summer => 28.0,
            Season::Autumn => 12.0,
            Season::Winter => -2.0,
        }
    }

    /// Power consumption multiplier (heating in winter, cooling in summer)
    pub fn power_multiplier(&self) -> f32 {
        match self.season {
            Season::Winter => 1.4,
            Season::Summer => 1.2,
            _ => 1.0,
        }
    }

    /// Water consumption multiplier
    pub fn water_multiplier(&self) -> f32 {
        match self.season {
            Season::Summer => 1.3,
            Season::Winter => 0.9,
            _ => 1.0,
        }
    }

    /// Agricultural output multiplier (farms produce less in winter)
    pub fn agriculture_multiplier(&self) -> f32 {
        match self.season {
            Season::Spring => 1.2,
            Season::Summer => 1.0,
            Season::Autumn => 0.8,
            Season::Winter => 0.3,
        }
    }

    /// Park effectiveness multiplier (people visit parks more in good weather)
    pub fn park_multiplier(&self) -> f32 {
        match (self.season, self.current_event) {
            (_, WeatherEvent::Rain) | (_, WeatherEvent::Storm) => 0.3,
            (_, WeatherEvent::ColdSnap) => 0.2,
            (Season::Summer, WeatherEvent::Clear) => 1.5,
            (Season::Spring, _) => 1.3,
            (Season::Autumn, _) => 0.8,
            (Season::Winter, _) => 0.4,
            _ => 1.0,
        }
    }

    /// Happiness modifier from weather
    pub fn happiness_modifier(&self) -> f32 {
        let mut modifier = 0.0;
        match self.current_event {
            WeatherEvent::HeatWave => modifier -= 5.0,
            WeatherEvent::ColdSnap => modifier -= 8.0,
            WeatherEvent::Storm => modifier -= 3.0,
            WeatherEvent::Rain => modifier -= 1.0,
            WeatherEvent::Clear => {
                if self.season == Season::Spring || self.season == Season::Summer {
                    modifier += 2.0;
                }
            }
        }
        modifier
    }

    /// Travel speed multiplier (snow/rain slows traffic)
    pub fn travel_speed_multiplier(&self) -> f32 {
        match self.current_event {
            WeatherEvent::Storm => 0.5,
            WeatherEvent::Rain => 0.8,
            WeatherEvent::ColdSnap => 0.7,
            _ => {
                if self.season == Season::Winter { 0.85 } else { 1.0 }
            }
        }
    }
}

pub fn update_weather(
    clock: Res<GameClock>,
    mut weather: ResMut<Weather>,
) {
    if clock.day == weather.last_update_day {
        return;
    }
    weather.last_update_day = clock.day;

    // Update season
    weather.season = Season::from_day(clock.day);

    // Update temperature (base + small daily variation)
    let base = Weather::base_temperature(weather.season);
    let variation = ((clock.day as f32 * 0.1).sin()) * 5.0; // ±5 degree swing
    weather.temperature = base + variation;

    // Apply weather event modifiers
    match weather.current_event {
        WeatherEvent::HeatWave => weather.temperature += 10.0,
        WeatherEvent::ColdSnap => weather.temperature -= 15.0,
        _ => {}
    }

    // Count down event duration
    if weather.event_days_remaining > 0 {
        weather.event_days_remaining -= 1;
        if weather.event_days_remaining == 0 {
            weather.current_event = WeatherEvent::Clear;
        }
    }

    // Random weather events (simple deterministic based on day hash)
    if weather.current_event == WeatherEvent::Clear {
        let hash = (clock.day.wrapping_mul(2654435761)) % 100;
        match (weather.season, hash) {
            (Season::Spring, 0..=8) => {
                weather.current_event = WeatherEvent::Rain;
                weather.event_days_remaining = 2 + (hash % 3);
            }
            (Season::Summer, 0..=3) => {
                weather.current_event = WeatherEvent::HeatWave;
                weather.event_days_remaining = 3 + (hash % 4);
            }
            (Season::Summer, 4..=7) => {
                weather.current_event = WeatherEvent::Storm;
                weather.event_days_remaining = 1 + (hash % 2);
            }
            (Season::Autumn, 0..=10) => {
                weather.current_event = WeatherEvent::Rain;
                weather.event_days_remaining = 2 + (hash % 4);
            }
            (Season::Winter, 0..=5) => {
                weather.current_event = WeatherEvent::ColdSnap;
                weather.event_days_remaining = 3 + (hash % 5);
            }
            (Season::Winter, 6..=8) => {
                weather.current_event = WeatherEvent::Storm;
                weather.event_days_remaining = 1 + (hash % 3);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_season_from_day() {
        assert_eq!(Season::from_day(1), Season::Spring);
        assert_eq!(Season::from_day(91), Season::Summer);
        assert_eq!(Season::from_day(182), Season::Autumn);
        assert_eq!(Season::from_day(273), Season::Winter);
        assert_eq!(Season::from_day(365), Season::Spring); // wraps
    }

    #[test]
    fn test_multipliers_in_range() {
        let weather = Weather::default();
        assert!(weather.power_multiplier() >= 0.5 && weather.power_multiplier() <= 2.0);
        assert!(weather.water_multiplier() >= 0.5 && weather.water_multiplier() <= 2.0);
        assert!(weather.park_multiplier() >= 0.0 && weather.park_multiplier() <= 2.0);
        assert!(weather.travel_speed_multiplier() >= 0.3 && weather.travel_speed_multiplier() <= 1.5);
    }

    #[test]
    fn test_weather_event_modifiers() {
        let mut w = Weather::default();
        w.current_event = WeatherEvent::HeatWave;
        assert!(w.happiness_modifier() < 0.0);

        w.current_event = WeatherEvent::Clear;
        w.season = Season::Summer;
        assert!(w.happiness_modifier() > 0.0);
    }
}

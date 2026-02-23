use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Precipitation intensity categories based on inches per hour.
///
/// Maps the continuous `Weather.precipitation_intensity` (in/hr) to discrete
/// categories for gameplay logic (UI display, threshold checks, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrecipitationCategory {
    /// No precipitation (0.0 in/hr).
    None,
    /// Very light rain (0.01 - 0.1 in/hr).
    Drizzle,
    /// Light rain (0.1 - 0.25 in/hr).
    Light,
    /// Moderate rain (0.25 - 1.0 in/hr).
    Moderate,
    /// Heavy rain (1.0 - 2.0 in/hr).
    Heavy,
    /// Torrential rain (2.0 - 4.0 in/hr).
    Torrential,
    /// Extreme rainfall (4.0+ in/hr).
    Extreme,
}

impl PrecipitationCategory {
    /// Classify a precipitation intensity (inches per hour) into a category.
    pub fn from_intensity(intensity: f32) -> Self {
        if intensity < 0.01 {
            PrecipitationCategory::None
        } else if intensity < 0.1 {
            PrecipitationCategory::Drizzle
        } else if intensity < 0.25 {
            PrecipitationCategory::Light
        } else if intensity < 1.0 {
            PrecipitationCategory::Moderate
        } else if intensity < 2.0 {
            PrecipitationCategory::Heavy
        } else if intensity < 4.0 {
            PrecipitationCategory::Torrential
        } else {
            PrecipitationCategory::Extreme
        }
    }

    /// Human-readable name for display in the UI.
    pub fn name(self) -> &'static str {
        match self {
            PrecipitationCategory::None => "None",
            PrecipitationCategory::Drizzle => "Drizzle",
            PrecipitationCategory::Light => "Light",
            PrecipitationCategory::Moderate => "Moderate",
            PrecipitationCategory::Heavy => "Heavy",
            PrecipitationCategory::Torrential => "Torrential",
            PrecipitationCategory::Extreme => "Extreme",
        }
    }
}

/// Lightweight event fired whenever weather conditions change.
///
/// Consumers can listen for this with `EventReader<WeatherChangeEvent>` instead of
/// polling the `Weather` resource every tick.
#[derive(Event, Debug, Clone)]
pub struct WeatherChangeEvent {
    /// The weather condition before the change.
    pub old_condition: WeatherCondition,
    /// The weather condition after the change.
    pub new_condition: WeatherCondition,
    /// The season before the change (differs from `new_season` on season transitions).
    pub old_season: Season,
    /// The season after the change.
    pub new_season: Season,
    /// `true` when the new condition is Storm, or temperature crosses extreme
    /// thresholds (heat-wave >35 C, cold-snap < -5 C).
    pub is_extreme: bool,
}

/// Temperature thresholds for extreme weather classification.
pub const EXTREME_HEAT_THRESHOLD: f32 = 35.0;
pub const EXTREME_COLD_THRESHOLD: f32 = -5.0;

/// Returns `true` if the weather condition or temperature qualifies as extreme.
pub fn is_extreme_weather(condition: WeatherCondition, temperature: f32) -> bool {
    matches!(condition, WeatherCondition::Storm)
        || !(EXTREME_COLD_THRESHOLD..=EXTREME_HEAT_THRESHOLD).contains(&temperature)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn from_day(day: u32) -> Season {
        // 360-day year: 90 days per season (30 days/month, 3 months/season)
        let day_of_year = ((day.saturating_sub(1)) % 360) + 1;
        match day_of_year {
            1..=90 => Season::Spring,
            91..=180 => Season::Summer,
            181..=270 => Season::Autumn,
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

    /// Seasonal happiness modifier: Summer +2, Spring +1, Autumn 0, Winter -2.
    pub fn happiness_modifier(self) -> f32 {
        match self {
            Season::Spring => 1.0,
            Season::Summer => 2.0,
            Season::Autumn => 0.0,
            Season::Winter => -2.0,
        }
    }

    /// Base grass color tint for terrain rendering, varying by season.
    pub fn grass_color(self) -> [f32; 3] {
        match self {
            Season::Spring => [0.35, 0.65, 0.15], // Bright green with slight yellow tint
            Season::Summer => [0.25, 0.55, 0.12], // Lush deep green
            Season::Autumn => [0.55, 0.40, 0.15], // Orange/brown
            Season::Winter => [0.75, 0.78, 0.82], // Grey/white with slight blue tint
        }
    }

    /// Seasonal min/max temperature range for Temperate (legacy default).
    #[allow(dead_code)]
    fn temperature_range(self) -> (f32, f32) {
        let params = super::climate::ClimateZone::Temperate.season_params(self);
        (params.t_min, params.t_max)
    }

    /// Seasonal min/max temperature range for a given climate zone.
    pub fn temperature_range_for_zone(self, zone: super::climate::ClimateZone) -> (f32, f32) {
        let params = zone.season_params(self);
        (params.t_min, params.t_max)
    }
}

/// Weather conditions derived from atmospheric state (cloud cover, precipitation, temperature).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    Sunny,
    PartlyCloudy,
    Overcast,
    Rain,
    HeavyRain,
    Snow,
    Storm,
}

/// Legacy alias kept for backward compatibility with save system and downstream consumers.
pub type WeatherEvent = WeatherCondition;

impl WeatherCondition {
    /// Derive condition from atmospheric state.
    pub fn from_atmosphere(
        cloud_cover: f32,
        precipitation_intensity: f32,
        temperature: f32,
    ) -> Self {
        if precipitation_intensity > 0.7 && cloud_cover > 0.8 {
            if temperature < 0.0 {
                WeatherCondition::Snow
            } else {
                WeatherCondition::Storm
            }
        } else if precipitation_intensity > 0.4 {
            if temperature < 0.0 {
                WeatherCondition::Snow
            } else {
                WeatherCondition::HeavyRain
            }
        } else if precipitation_intensity > 0.1 {
            if temperature < 0.0 {
                WeatherCondition::Snow
            } else {
                WeatherCondition::Rain
            }
        } else if cloud_cover > 0.7 {
            WeatherCondition::Overcast
        } else if cloud_cover > 0.3 {
            WeatherCondition::PartlyCloudy
        } else {
            WeatherCondition::Sunny
        }
    }

    /// Whether this condition counts as precipitation for gameplay purposes.
    pub fn is_precipitation(self) -> bool {
        matches!(
            self,
            WeatherCondition::Rain
                | WeatherCondition::HeavyRain
                | WeatherCondition::Snow
                | WeatherCondition::Storm
        )
    }
}

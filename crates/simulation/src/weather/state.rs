use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::climate::ClimateZone;
use super::types::{PrecipitationCategory, Season, WeatherCondition};

/// Number of days in the rolling rainfall window for drought calculation.
pub(crate) const ROLLING_RAINFALL_DAYS: usize = 30;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Weather {
    pub season: Season,
    pub temperature: f32, // -10 to 40 Celsius
    pub current_event: WeatherCondition,
    pub event_days_remaining: u32,
    pub last_update_day: u32,
    /// Whether natural disasters (tornado, earthquake, flood) can occur.
    pub disasters_enabled: bool,
    /// Relative humidity (0.0 to 1.0).
    #[serde(default = "default_humidity")]
    pub humidity: f32,
    /// Cloud cover fraction (0.0 = clear sky, 1.0 = fully overcast).
    #[serde(default)]
    pub cloud_cover: f32,
    /// Precipitation intensity in inches per hour (0.0 = none, 4.0+ = extreme).
    ///
    /// Set each hour based on the derived weather condition and season.
    /// Use `precipitation_category()` to classify into discrete buckets.
    #[serde(default)]
    pub precipitation_intensity: f32,
    /// Internal atmospheric precipitation signal (0.0 - 1.0) used by the weather
    /// model to derive weather conditions. Not intended for external consumption;
    /// use `precipitation_intensity` (in/hr) instead.
    #[serde(default)]
    pub atmo_precipitation: f32,
    /// Last hour that triggered a weather update (used for hourly boundary detection).
    #[serde(default)]
    pub last_update_hour: u32,
    /// Whether the previous tick ended in an extreme weather state (for change detection).
    #[serde(default)]
    pub prev_extreme: bool,
    /// Accumulated rainfall for the current day (inches).
    #[serde(default)]
    pub daily_rainfall: f32,
    /// Rolling 30-day rainfall total (inches) for drought calculation.
    #[serde(default)]
    pub rolling_30day_rainfall: f32,
    /// Ring buffer of daily rainfall totals for the last 30 days.
    /// Index 0 corresponds to the oldest day in the window.
    #[serde(default = "default_rainfall_history")]
    pub rainfall_history: Vec<f32>,
}

fn default_humidity() -> f32 {
    0.5
}

fn default_rainfall_history() -> Vec<f32> {
    vec![0.0; ROLLING_RAINFALL_DAYS]
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            season: Season::Spring,
            temperature: 15.0,
            current_event: WeatherCondition::Sunny,
            event_days_remaining: 0,
            last_update_day: 0,
            disasters_enabled: true,
            humidity: 0.5,
            cloud_cover: 0.1,
            precipitation_intensity: 0.0,
            atmo_precipitation: 0.0,
            last_update_hour: 0,
            prev_extreme: false,
            daily_rainfall: 0.0,
            rolling_30day_rainfall: 0.0,
            rainfall_history: vec![0.0; ROLLING_RAINFALL_DAYS],
        }
    }
}

impl Weather {
    /// Seasonal base temperature range for Temperate (legacy default).
    #[allow(dead_code)]
    fn seasonal_range(season: Season) -> (f32, f32) {
        season.temperature_range_for_zone(ClimateZone::Temperate)
    }

    /// Seasonal base temperature range for a given climate zone.
    #[allow(dead_code)]
    fn seasonal_range_for_zone(season: Season, zone: ClimateZone) -> (f32, f32) {
        season.temperature_range_for_zone(zone)
    }

    /// Derive the current weather condition from atmospheric state.
    pub fn condition(&self) -> WeatherCondition {
        WeatherCondition::from_atmosphere(
            self.cloud_cover,
            self.atmo_precipitation,
            self.temperature,
        )
    }

    /// Return the current precipitation intensity category.
    pub fn precipitation_category(&self) -> PrecipitationCategory {
        PrecipitationCategory::from_intensity(self.precipitation_intensity)
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
        let cond = self.current_event;
        match (self.season, cond) {
            (_, WeatherCondition::Rain)
            | (_, WeatherCondition::HeavyRain)
            | (_, WeatherCondition::Storm) => 0.3,
            (_, WeatherCondition::Snow) => 0.2,
            (_, WeatherCondition::Overcast) => 0.6,
            (Season::Summer, WeatherCondition::Sunny) => 1.5,
            (Season::Spring, _) => 1.3,
            (Season::Autumn, _) => 0.8,
            (Season::Winter, _) => 0.4,
            _ => 1.0,
        }
    }

    /// Happiness modifier from weather (events + seasonal baseline)
    pub fn happiness_modifier(&self) -> f32 {
        let mut modifier = self.season.happiness_modifier();
        // Extreme temperature penalties (replaces old HeatWave/ColdSnap event modifiers)
        if self.temperature > 35.0 {
            modifier -= 5.0; // equivalent to old HeatWave
        } else if self.temperature < -5.0 {
            modifier -= 8.0; // equivalent to old ColdSnap
        }
        match self.current_event {
            WeatherCondition::Storm => modifier -= 3.0,
            WeatherCondition::HeavyRain => modifier -= 2.0,
            WeatherCondition::Rain | WeatherCondition::Snow => modifier -= 1.0,
            WeatherCondition::Overcast => modifier -= 0.5,
            WeatherCondition::Sunny | WeatherCondition::PartlyCloudy => {
                if self.season == Season::Spring || self.season == Season::Summer {
                    modifier += 2.0;
                }
            }
        }
        modifier
    }

    /// Travel speed multiplier (snow/rain/fog slows traffic).
    ///
    /// When a `FogState` is available, use `travel_speed_multiplier_with_fog` instead
    /// to incorporate the fog traffic penalty.
    pub fn travel_speed_multiplier(&self) -> f32 {
        match self.current_event {
            WeatherCondition::Storm => 0.5,
            WeatherCondition::HeavyRain => 0.65,
            WeatherCondition::Snow => 0.6,
            WeatherCondition::Rain => 0.8,
            WeatherCondition::Overcast | WeatherCondition::PartlyCloudy => {
                if self.season == Season::Winter {
                    0.85
                } else {
                    1.0
                }
            }
            WeatherCondition::Sunny => {
                if self.season == Season::Winter {
                    0.85
                } else {
                    1.0
                }
            }
        }
    }

    /// Travel speed multiplier incorporating both weather and fog effects.
    ///
    /// Returns the minimum of the weather-based multiplier and the fog traffic modifier,
    /// ensuring the worst condition dominates.
    pub fn travel_speed_multiplier_with_fog(&self, fog_traffic_modifier: f32) -> f32 {
        self.travel_speed_multiplier().min(fog_traffic_modifier)
    }
}

/// Resource holding current construction speed and cost modifiers derived from weather/season.
///
/// Updated each tick by `update_construction_modifiers`. Other systems can query this
/// resource when construction times are applied.
///
/// Speed factor: `construction_progress_per_tick = base_rate * speed_factor`
/// Cost factor: multiplied against base construction cost.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionModifiers {
    /// Combined speed factor (season_factor * weather_factor). Range: 0.0 to ~1.1.
    pub speed_factor: f32,
    /// Cost multiplier based on season. Range: 1.0 to 1.25.
    pub cost_factor: f32,
}

impl Default for ConstructionModifiers {
    fn default() -> Self {
        Self {
            speed_factor: 1.0,
            cost_factor: 1.0,
        }
    }
}

impl ConstructionModifiers {
    /// Season-based construction speed factor.
    pub fn season_speed_factor(season: Season) -> f32 {
        match season {
            Season::Spring => 1.0,
            Season::Summer => 1.1,
            Season::Autumn => 0.9,
            Season::Winter => 0.6,
        }
    }

    /// Weather condition-based construction speed factor.
    pub fn weather_speed_factor(condition: WeatherCondition, temperature: f32) -> f32 {
        // Extreme cold (below -5C) halts construction almost entirely
        if temperature < -5.0 {
            return 0.2;
        }

        match condition {
            WeatherCondition::Sunny | WeatherCondition::PartlyCloudy => 1.0,
            WeatherCondition::Overcast => 1.0,
            WeatherCondition::Rain => 0.5,
            WeatherCondition::HeavyRain => 0.5,
            WeatherCondition::Snow => 0.3,
            WeatherCondition::Storm => 0.0,
        }
    }

    /// Season-based construction cost factor.
    pub fn season_cost_factor(season: Season) -> f32 {
        match season {
            Season::Spring | Season::Summer => 1.0,
            Season::Autumn => 1.05,
            Season::Winter => 1.25,
        }
    }
}

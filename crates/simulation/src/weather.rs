use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::time_of_day::GameClock;

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
const EXTREME_HEAT_THRESHOLD: f32 = 35.0;
const EXTREME_COLD_THRESHOLD: f32 = -5.0;

/// Returns `true` if the weather condition or temperature qualifies as extreme.
fn is_extreme_weather(condition: WeatherCondition, temperature: f32) -> bool {
    matches!(condition, WeatherCondition::Storm)
        || !(EXTREME_COLD_THRESHOLD..=EXTREME_HEAT_THRESHOLD).contains(&temperature)
}

/// Climate zone presets that shift all seasonal parameters for different map types.
///
/// Each zone defines temperature ranges, precipitation patterns, and snow behavior
/// that replace the hardcoded Temperate defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Resource, Default)]
pub enum ClimateZone {
    /// Moderate temperatures, four distinct seasons. Backward-compatible default.
    #[default]
    Temperate,
    /// Hot year-round, heavy rainfall, no snow.
    Tropical,
    /// Very hot summers, mild winters, minimal precipitation.
    Arid,
    /// Dry hot summers, mild wet winters, no snow.
    Mediterranean,
    /// Extreme temperature swings between seasons, cold winters, warm summers.
    Continental,
    /// Very cold winters, cool summers, heavy snow.
    Subarctic,
    /// Mild, wet year-round with narrow temperature range.
    Oceanic,
}

/// Per-season climate parameters driven by the active `ClimateZone`.
#[derive(Debug, Clone, Copy)]
pub struct SeasonClimateParams {
    /// Minimum temperature for the season (Celsius).
    pub t_min: f32,
    /// Maximum temperature for the season (Celsius).
    pub t_max: f32,
    /// Base chance of a precipitation event starting on any given day (0.0 to 1.0).
    pub precipitation_chance: f32,
    /// Whether snow is possible in this season/zone combination.
    pub snow_enabled: bool,
}

impl ClimateZone {
    /// Return all climate zone variants (useful for UI iteration).
    pub fn all() -> &'static [ClimateZone] {
        &[
            ClimateZone::Temperate,
            ClimateZone::Tropical,
            ClimateZone::Arid,
            ClimateZone::Mediterranean,
            ClimateZone::Continental,
            ClimateZone::Subarctic,
            ClimateZone::Oceanic,
        ]
    }

    /// Human-readable name for display in the UI.
    pub fn name(self) -> &'static str {
        match self {
            ClimateZone::Temperate => "Temperate",
            ClimateZone::Tropical => "Tropical",
            ClimateZone::Arid => "Arid",
            ClimateZone::Mediterranean => "Mediterranean",
            ClimateZone::Continental => "Continental",
            ClimateZone::Subarctic => "Subarctic",
            ClimateZone::Oceanic => "Oceanic",
        }
    }

    /// Get the climate parameters for a given season in this zone.
    ///
    /// Temperature values are in Celsius. Precipitation chance is a base probability
    /// (0.0 to 1.0) that a precipitation event begins on any given day.
    pub fn season_params(self, season: Season) -> SeasonClimateParams {
        match self {
            ClimateZone::Temperate => match season {
                // Original hardcoded values preserved for backward compatibility.
                Season::Spring => SeasonClimateParams {
                    t_min: 8.0,
                    t_max: 22.0,
                    precipitation_chance: 0.09,
                    snow_enabled: false,
                },
                Season::Summer => SeasonClimateParams {
                    t_min: 20.0,
                    t_max: 36.0,
                    precipitation_chance: 0.08,
                    snow_enabled: false,
                },
                Season::Autumn => SeasonClimateParams {
                    t_min: 5.0,
                    t_max: 19.0,
                    precipitation_chance: 0.11,
                    snow_enabled: false,
                },
                Season::Winter => SeasonClimateParams {
                    t_min: -8.0,
                    t_max: 6.0,
                    precipitation_chance: 0.09,
                    snow_enabled: true,
                },
            },
            ClimateZone::Tropical => match season {
                // Hot year-round, heavy rainfall, no snow.
                // Winter low ~18C (65F), Summer high ~38C
                Season::Spring => SeasonClimateParams {
                    t_min: 22.0,
                    t_max: 34.0,
                    precipitation_chance: 0.20,
                    snow_enabled: false,
                },
                Season::Summer => SeasonClimateParams {
                    t_min: 24.0,
                    t_max: 38.0,
                    precipitation_chance: 0.30,
                    snow_enabled: false,
                },
                Season::Autumn => SeasonClimateParams {
                    t_min: 22.0,
                    t_max: 34.0,
                    precipitation_chance: 0.25,
                    snow_enabled: false,
                },
                Season::Winter => SeasonClimateParams {
                    t_min: 18.0,
                    t_max: 30.0,
                    precipitation_chance: 0.15,
                    snow_enabled: false,
                },
            },
            ClimateZone::Arid => match season {
                // Very hot, minimal precipitation.
                Season::Spring => SeasonClimateParams {
                    t_min: 15.0,
                    t_max: 35.0,
                    precipitation_chance: 0.02,
                    snow_enabled: false,
                },
                Season::Summer => SeasonClimateParams {
                    t_min: 25.0,
                    t_max: 48.0,
                    precipitation_chance: 0.01,
                    snow_enabled: false,
                },
                Season::Autumn => SeasonClimateParams {
                    t_min: 15.0,
                    t_max: 33.0,
                    precipitation_chance: 0.02,
                    snow_enabled: false,
                },
                Season::Winter => SeasonClimateParams {
                    t_min: 5.0,
                    t_max: 22.0,
                    precipitation_chance: 0.03,
                    snow_enabled: false,
                },
            },
            ClimateZone::Mediterranean => match season {
                // Dry hot summers, mild wet winters.
                Season::Spring => SeasonClimateParams {
                    t_min: 12.0,
                    t_max: 24.0,
                    precipitation_chance: 0.10,
                    snow_enabled: false,
                },
                Season::Summer => SeasonClimateParams {
                    t_min: 20.0,
                    t_max: 35.0,
                    precipitation_chance: 0.02,
                    snow_enabled: false,
                },
                Season::Autumn => SeasonClimateParams {
                    t_min: 12.0,
                    t_max: 25.0,
                    precipitation_chance: 0.12,
                    snow_enabled: false,
                },
                Season::Winter => SeasonClimateParams {
                    t_min: 5.0,
                    t_max: 15.0,
                    precipitation_chance: 0.18,
                    snow_enabled: false,
                },
            },
            ClimateZone::Continental => match season {
                // Extreme temperature swings.
                Season::Spring => SeasonClimateParams {
                    t_min: 0.0,
                    t_max: 18.0,
                    precipitation_chance: 0.10,
                    snow_enabled: true,
                },
                Season::Summer => SeasonClimateParams {
                    t_min: 18.0,
                    t_max: 38.0,
                    precipitation_chance: 0.10,
                    snow_enabled: false,
                },
                Season::Autumn => SeasonClimateParams {
                    t_min: -2.0,
                    t_max: 15.0,
                    precipitation_chance: 0.10,
                    snow_enabled: true,
                },
                Season::Winter => SeasonClimateParams {
                    t_min: -25.0,
                    t_max: -5.0,
                    precipitation_chance: 0.12,
                    snow_enabled: true,
                },
            },
            ClimateZone::Subarctic => match season {
                // Very cold, heavy snow. Winter low ~-34C (-30F).
                Season::Spring => SeasonClimateParams {
                    t_min: -10.0,
                    t_max: 8.0,
                    precipitation_chance: 0.10,
                    snow_enabled: true,
                },
                Season::Summer => SeasonClimateParams {
                    t_min: 8.0,
                    t_max: 22.0,
                    precipitation_chance: 0.12,
                    snow_enabled: false,
                },
                Season::Autumn => SeasonClimateParams {
                    t_min: -12.0,
                    t_max: 5.0,
                    precipitation_chance: 0.12,
                    snow_enabled: true,
                },
                Season::Winter => SeasonClimateParams {
                    t_min: -34.0,
                    t_max: -12.0,
                    precipitation_chance: 0.15,
                    snow_enabled: true,
                },
            },
            ClimateZone::Oceanic => match season {
                // Mild, wet year-round, narrow temperature range.
                Season::Spring => SeasonClimateParams {
                    t_min: 8.0,
                    t_max: 16.0,
                    precipitation_chance: 0.18,
                    snow_enabled: false,
                },
                Season::Summer => SeasonClimateParams {
                    t_min: 14.0,
                    t_max: 24.0,
                    precipitation_chance: 0.14,
                    snow_enabled: false,
                },
                Season::Autumn => SeasonClimateParams {
                    t_min: 7.0,
                    t_max: 16.0,
                    precipitation_chance: 0.20,
                    snow_enabled: false,
                },
                Season::Winter => SeasonClimateParams {
                    t_min: 2.0,
                    t_max: 10.0,
                    precipitation_chance: 0.22,
                    snow_enabled: true,
                },
            },
        }
    }

    /// Seasonal baseline cloud cover for this climate zone and season.
    pub fn baseline_cloud_cover(self, season: Season) -> f32 {
        match self {
            ClimateZone::Temperate => match season {
                Season::Spring => 0.3,
                Season::Summer => 0.15,
                Season::Autumn => 0.4,
                Season::Winter => 0.5,
            },
            ClimateZone::Tropical => match season {
                Season::Spring => 0.4,
                Season::Summer => 0.5,
                Season::Autumn => 0.45,
                Season::Winter => 0.3,
            },
            ClimateZone::Arid => match season {
                Season::Spring => 0.1,
                Season::Summer => 0.05,
                Season::Autumn => 0.1,
                Season::Winter => 0.15,
            },
            ClimateZone::Mediterranean => match season {
                Season::Spring => 0.25,
                Season::Summer => 0.1,
                Season::Autumn => 0.3,
                Season::Winter => 0.45,
            },
            ClimateZone::Continental => match season {
                Season::Spring => 0.35,
                Season::Summer => 0.2,
                Season::Autumn => 0.4,
                Season::Winter => 0.5,
            },
            ClimateZone::Subarctic => match season {
                Season::Spring => 0.4,
                Season::Summer => 0.3,
                Season::Autumn => 0.5,
                Season::Winter => 0.55,
            },
            ClimateZone::Oceanic => match season {
                Season::Spring => 0.45,
                Season::Summer => 0.35,
                Season::Autumn => 0.5,
                Season::Winter => 0.55,
            },
        }
    }

    /// Seasonal baseline humidity for this climate zone and season.
    pub fn baseline_humidity(self, season: Season) -> f32 {
        match self {
            ClimateZone::Temperate => match season {
                Season::Spring => 0.55,
                Season::Summer => 0.4,
                Season::Autumn => 0.6,
                Season::Winter => 0.65,
            },
            ClimateZone::Tropical => match season {
                Season::Spring => 0.75,
                Season::Summer => 0.85,
                Season::Autumn => 0.8,
                Season::Winter => 0.65,
            },
            ClimateZone::Arid => match season {
                Season::Spring => 0.2,
                Season::Summer => 0.1,
                Season::Autumn => 0.2,
                Season::Winter => 0.25,
            },
            ClimateZone::Mediterranean => match season {
                Season::Spring => 0.5,
                Season::Summer => 0.3,
                Season::Autumn => 0.55,
                Season::Winter => 0.65,
            },
            ClimateZone::Continental => match season {
                Season::Spring => 0.5,
                Season::Summer => 0.45,
                Season::Autumn => 0.55,
                Season::Winter => 0.6,
            },
            ClimateZone::Subarctic => match season {
                Season::Spring => 0.55,
                Season::Summer => 0.5,
                Season::Autumn => 0.6,
                Season::Winter => 0.65,
            },
            ClimateZone::Oceanic => match season {
                Season::Spring => 0.65,
                Season::Summer => 0.6,
                Season::Autumn => 0.7,
                Season::Winter => 0.75,
            },
        }
    }
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
    fn temperature_range(self) -> (f32, f32) {
        let params = ClimateZone::Temperate.season_params(self);
        (params.t_min, params.t_max)
    }

    /// Seasonal min/max temperature range for a given climate zone.
    pub fn temperature_range_for_zone(self, zone: ClimateZone) -> (f32, f32) {
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

/// Diurnal temperature factor: models realistic day/night temperature cycle.
///
/// Returns a value in `[0.0, 1.0]` where 0.0 is the daily minimum (around 06:00)
/// and 1.0 is the daily maximum (around 15:00).
///
/// Uses a cosine curve shifted so that:
/// - Minimum temperature occurs at hour 6 (just after sunrise)
/// - Maximum temperature occurs at hour 15 (mid-afternoon solar lag)
pub fn diurnal_factor(hour: u32) -> f32 {
    // Center of the cosine at hour 15 (peak), period 24 hours
    // cos((hour - 15) * 2*PI / 24) maps:
    //   hour=15 -> cos(0) = 1.0
    //   hour=3  -> cos(PI) = -1.0
    //   hour=6  -> cos(-9 * PI/12) = cos(-3*PI/4) ~ -0.707
    //
    // We want minimum at 6, maximum at 15. A shifted cosine:
    // factor = 0.5 + 0.5 * cos((hour - 15) * 2*PI / 24)
    // At hour 15: 0.5 + 0.5*1.0 = 1.0
    // At hour 3: 0.5 + 0.5*(-1.0) = 0.0
    // At hour 6: 0.5 + 0.5*cos(-3*PI/4) ~ 0.146
    //
    // To get exact 0.0 at hour 6, use a piecewise or adjusted formula.
    // Simple approach: remap so hour 6->0.0, hour 15->1.0 using cosine.
    // Phase: peak at 15, trough at 15-12=3. We want trough at 6.
    // Shift: use (hour - 10.5) to center between 6 and 15 (midpoint = 10.5)
    // cos((hour - 10.5) * PI / 9) gives:
    //   hour=10.5 -> cos(0) = 1.0 (wrong, we want peak at 15)
    //
    // Cleanest: use a sine with the right phase.
    // sin((hour - 6) * PI / 18) * sin(...)  -- no, keep it simple:
    //
    // factor = 0.5 - 0.5 * cos((hour - 6) * PI / 9)  for hour in [6..15] rising
    // For a full 24-hour smooth cycle, use:
    // factor = 0.5 + 0.5 * cos((hour - 15) * 2*PI / 24)
    // Then clamp and renormalize so min->0, max->1.

    let h = (hour % 24) as f32;

    // Piecewise smooth: nighttime cooling from 15:00 to 06:00 (15 hours),
    // daytime warming from 06:00 to 15:00 (9 hours).
    if (6.0..=15.0).contains(&h) {
        // Warming phase: 06:00 to 15:00 (9 hours)
        let t = (h - 6.0) / 9.0; // 0..1
        0.5 - 0.5 * (t * std::f32::consts::PI).cos()
    } else {
        // Cooling phase: 15:00 to 06:00 next day (15 hours)
        let hours_since_15 = if h >= 15.0 { h - 15.0 } else { h + 9.0 };
        let t = hours_since_15 / 15.0; // 0..1
        0.5 + 0.5 * (t * std::f32::consts::PI).cos()
    }
}

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
    /// Precipitation intensity (0.0 = none, 1.0 = torrential).
    #[serde(default)]
    pub precipitation_intensity: f32,
    /// Last hour that triggered a weather update (used for hourly boundary detection).
    #[serde(default)]
    pub last_update_hour: u32,
    /// Whether the previous tick ended in an extreme weather state (for change detection).
    #[serde(default)]
    pub prev_extreme: bool,
}

fn default_humidity() -> f32 {
    0.5
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
            last_update_hour: 0,
            prev_extreme: false,
        }
    }
}

impl Weather {
    /// Seasonal base temperature range for Temperate (legacy default).
    fn seasonal_range(season: Season) -> (f32, f32) {
        season.temperature_range()
    }

    /// Seasonal base temperature range for a given climate zone.
    fn seasonal_range_for_zone(season: Season, zone: ClimateZone) -> (f32, f32) {
        season.temperature_range_for_zone(zone)
    }

    /// Derive the current weather condition from atmospheric state.
    pub fn condition(&self) -> WeatherCondition {
        WeatherCondition::from_atmosphere(
            self.cloud_cover,
            self.precipitation_intensity,
            self.temperature,
        )
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

    /// Travel speed multiplier (snow/rain slows traffic)
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
}

/// Hourly weather update system. Runs every time the game clock crosses an hour boundary.
///
/// Implements:
/// - Diurnal temperature curve: `T(hour) = T_min + (T_max - T_min) * diurnal_factor(hour)`
/// - Smooth transitions: `temperature += (target - temperature) * 0.3`
/// - Daily variation via deterministic hash on day
/// - Atmospheric state updates (cloud_cover, humidity, precipitation)
/// - Weather condition derived from atmospheric state
/// - All parameters driven by the active `ClimateZone`.
pub fn update_weather(
    clock: Res<GameClock>,
    mut weather: ResMut<Weather>,
    mut change_events: EventWriter<WeatherChangeEvent>,
    climate: Res<ClimateZone>,
) {
    let current_hour = clock.hour_of_day();

    // Only update on hour boundaries (when the integer hour changes)
    if current_hour == weather.last_update_hour && clock.day == weather.last_update_day {
        return;
    }

    // Snapshot pre-update state for change detection
    let old_condition = weather.current_event;
    let old_season = weather.season;
    let old_was_extreme = weather.prev_extreme;

    let day_changed = clock.day != weather.last_update_day;
    weather.last_update_hour = current_hour;
    weather.last_update_day = clock.day;

    // Update season
    weather.season = Season::from_day(clock.day);

    // Get climate parameters for the current season and zone
    let zone = *climate;
    let climate_params = zone.season_params(weather.season);

    // --- Diurnal temperature ---
    let (t_min, t_max) = (climate_params.t_min, climate_params.t_max);
    // Add daily variation (deterministic based on day) of +/- 3 degrees
    let day_variation = ((clock.day as f32 * 0.1).sin()) * 3.0;
    let effective_min = t_min + day_variation;
    let effective_max = t_max + day_variation;

    let factor = diurnal_factor(current_hour);
    let target_temp = effective_min + (effective_max - effective_min) * factor;

    // Smooth transition toward target
    weather.temperature += (target_temp - weather.temperature) * 0.3;

    // --- Atmospheric state updates (daily events + hourly cloud drift) ---
    if day_changed {
        // Count down event duration
        if weather.event_days_remaining > 0 {
            weather.event_days_remaining -= 1;
            if weather.event_days_remaining == 0 {
                // Event ended: reset atmospheric state toward clear
                weather.cloud_cover *= 0.5;
                weather.precipitation_intensity = 0.0;
                weather.humidity *= 0.7;
            }
        }

        // Random weather events (deterministic based on day hash)
        if weather.event_days_remaining == 0 {
            let hash = (clock.day.wrapping_mul(2654435761)) % 100;

            // Compute the precipitation threshold for the current season/zone.
            // The base precipitation_chance (0.0..1.0) is scaled to a 0..99 hash range.
            let precip_threshold = (climate_params.precipitation_chance * 100.0) as u32;

            // Check if a precipitation event should occur
            let is_precip_day = hash < precip_threshold;

            // Check for extreme weather events (heat wave in summer, cold snap in winter)
            let is_extreme_day = hash < 4; // ~4% chance for extreme events

            match (weather.season, is_extreme_day, is_precip_day) {
                // Summer heat wave (only if extreme day, any climate)
                (Season::Summer, true, _) => {
                    weather.cloud_cover = 0.05;
                    weather.precipitation_intensity = 0.0;
                    weather.humidity = 0.3;
                    weather.event_days_remaining = 3 + (hash % 4);
                    weather.temperature = t_max + 8.0;
                }
                // Winter cold snap (only if extreme day and snow is enabled)
                (Season::Winter, true, _) if climate_params.snow_enabled => {
                    weather.cloud_cover = 0.2;
                    weather.precipitation_intensity = 0.0;
                    weather.humidity = 0.4;
                    weather.event_days_remaining = 3 + (hash % 5);
                    weather.temperature = t_min - 10.0;
                }
                // Precipitation event
                (_, _, true) => {
                    let is_storm = hash < (precip_threshold / 3).max(1);
                    if is_storm {
                        // Storm / heavy precipitation
                        weather.cloud_cover = 0.9;
                        weather.precipitation_intensity = 0.7 + (hash % 20) as f32 * 0.01;
                        weather.humidity = 0.9 + (hash % 10) as f32 * 0.005;
                        weather.event_days_remaining = 1 + (hash % 3);
                    } else {
                        // Normal rain/snow
                        weather.cloud_cover = 0.7 + (hash % 20) as f32 * 0.01;
                        weather.precipitation_intensity = 0.2 + (hash % 15) as f32 * 0.02;
                        weather.humidity = 0.8;
                        weather.event_days_remaining = 2 + (hash % 4);
                    }
                }
                // No event: drift toward seasonal baseline
                _ => {
                    let seasonal_baseline_cloud = zone.baseline_cloud_cover(weather.season);
                    weather.cloud_cover +=
                        (seasonal_baseline_cloud - weather.cloud_cover) * 0.2;
                    weather.precipitation_intensity *= 0.5; // decay precipitation
                    let seasonal_humidity = zone.baseline_humidity(weather.season);
                    weather.humidity += (seasonal_humidity - weather.humidity) * 0.2;
                }
            }
        }
    }

    // Hourly cloud drift: small random-ish perturbation based on hour + day
    let hour_hash =
        ((clock.day.wrapping_mul(7919)).wrapping_add(current_hour.wrapping_mul(6271))) % 1000;
    let drift = (hour_hash as f32 / 1000.0 - 0.5) * 0.06; // +/- 0.03
    weather.cloud_cover = (weather.cloud_cover + drift).clamp(0.0, 1.0);

    // Clamp all atmospheric values
    weather.humidity = weather.humidity.clamp(0.0, 1.0);
    weather.precipitation_intensity = weather.precipitation_intensity.clamp(0.0, 1.0);

    // If snow is disabled for this zone/season, convert snow to rain
    let snow_enabled = zone.season_params(weather.season).snow_enabled;
    let effective_temp = if !snow_enabled && weather.temperature < 0.0 {
        // Force positive temperature so WeatherCondition::from_atmosphere won't produce Snow
        0.1
    } else {
        weather.temperature
    };

    // Derive weather condition from atmospheric state
    weather.current_event = WeatherCondition::from_atmosphere(
        weather.cloud_cover,
        weather.precipitation_intensity,
        effective_temp,
    );

    // --- Fire WeatherChangeEvent if anything meaningful changed ---
    let new_condition = weather.current_event;
    let new_season = weather.season;
    let new_is_extreme = is_extreme_weather(new_condition, weather.temperature);

    let condition_changed = old_condition != new_condition;
    let season_changed = old_season != new_season;
    let extreme_crossed = old_was_extreme != new_is_extreme;

    // Store current extreme state for next tick's comparison
    weather.prev_extreme = new_is_extreme;

    if condition_changed || season_changed || extreme_crossed {
        change_events.send(WeatherChangeEvent {
            old_condition,
            new_condition,
            old_season,
            new_season,
            is_extreme: new_is_extreme,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_season_from_day() {
        assert_eq!(Season::from_day(1), Season::Spring);
        assert_eq!(Season::from_day(90), Season::Spring);
        assert_eq!(Season::from_day(91), Season::Summer);
        assert_eq!(Season::from_day(180), Season::Summer);
        assert_eq!(Season::from_day(181), Season::Autumn);
        assert_eq!(Season::from_day(270), Season::Autumn);
        assert_eq!(Season::from_day(271), Season::Winter);
        assert_eq!(Season::from_day(360), Season::Winter);
        assert_eq!(Season::from_day(361), Season::Spring); // wraps
    }

    #[test]
    fn test_season_happiness_modifiers() {
        assert_eq!(Season::Spring.happiness_modifier(), 1.0);
        assert_eq!(Season::Summer.happiness_modifier(), 2.0);
        assert_eq!(Season::Autumn.happiness_modifier(), 0.0);
        assert_eq!(Season::Winter.happiness_modifier(), -2.0);
    }

    #[test]
    fn test_multipliers_in_range() {
        let weather = Weather::default();
        assert!((0.5..=2.0).contains(&weather.power_multiplier()));
        assert!((0.5..=2.0).contains(&weather.water_multiplier()));
        assert!((0.0..=2.0).contains(&weather.park_multiplier()));
        assert!((0.3..=1.5).contains(&weather.travel_speed_multiplier()));
    }

    #[test]
    fn test_weather_condition_modifiers() {
        let mut w = Weather::default();
        // Simulate heat wave: extreme temperature
        w.temperature = 38.0;
        w.current_event = WeatherCondition::Sunny;
        // HeatWave equivalent: seasonal(Spring=+1) + extreme_heat(-5) + sunny_spring(+2) = -2
        assert!(w.happiness_modifier() < 0.0);

        w.current_event = WeatherCondition::Sunny;
        w.temperature = 25.0;
        w.season = Season::Summer;
        // Clear+Summer: seasonal(+2) + sunny_bonus(+2) = +4
        assert!(w.happiness_modifier() > 0.0);

        w.season = Season::Winter;
        w.temperature = -10.0;
        w.current_event = WeatherCondition::Snow;
        // ColdSnap equivalent: seasonal(-2) + extreme_cold(-8) + snow(-1) = -11
        assert!(w.happiness_modifier() < -5.0);
    }

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
    fn test_condition_from_atmosphere_sunny() {
        let cond = WeatherCondition::from_atmosphere(0.1, 0.0, 20.0);
        assert_eq!(cond, WeatherCondition::Sunny);
    }

    #[test]
    fn test_condition_from_atmosphere_partly_cloudy() {
        let cond = WeatherCondition::from_atmosphere(0.5, 0.0, 20.0);
        assert_eq!(cond, WeatherCondition::PartlyCloudy);
    }

    #[test]
    fn test_condition_from_atmosphere_overcast() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.05, 20.0);
        assert_eq!(cond, WeatherCondition::Overcast);
    }

    #[test]
    fn test_condition_from_atmosphere_rain() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.2, 10.0);
        assert_eq!(cond, WeatherCondition::Rain);
    }

    #[test]
    fn test_condition_from_atmosphere_heavy_rain() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.5, 10.0);
        assert_eq!(cond, WeatherCondition::HeavyRain);
    }

    #[test]
    fn test_condition_from_atmosphere_snow() {
        let cond = WeatherCondition::from_atmosphere(0.8, 0.3, -5.0);
        assert_eq!(cond, WeatherCondition::Snow);
    }

    #[test]
    fn test_condition_from_atmosphere_storm() {
        let cond = WeatherCondition::from_atmosphere(0.9, 0.8, 15.0);
        assert_eq!(cond, WeatherCondition::Storm);
    }

    #[test]
    fn test_condition_is_precipitation() {
        assert!(!WeatherCondition::Sunny.is_precipitation());
        assert!(!WeatherCondition::PartlyCloudy.is_precipitation());
        assert!(!WeatherCondition::Overcast.is_precipitation());
        assert!(WeatherCondition::Rain.is_precipitation());
        assert!(WeatherCondition::HeavyRain.is_precipitation());
        assert!(WeatherCondition::Snow.is_precipitation());
        assert!(WeatherCondition::Storm.is_precipitation());
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
        let (t_min, t_max) = Season::Summer.temperature_range();
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

    #[test]
    fn test_default_weather_has_new_fields() {
        let w = Weather::default();
        assert!((w.humidity - 0.5_f32).abs() < 0.01);
        assert!(w.cloud_cover < 0.2_f32);
        assert!(w.precipitation_intensity < 0.01_f32);
        assert_eq!(w.last_update_hour, 0);
    }

    #[test]
    fn test_weather_condition_method() {
        let mut w = Weather::default();
        w.cloud_cover = 0.1;
        w.precipitation_intensity = 0.0;
        w.temperature = 20.0;
        assert_eq!(w.condition(), WeatherCondition::Sunny);

        w.cloud_cover = 0.9;
        w.precipitation_intensity = 0.8;
        w.temperature = 20.0;
        assert_eq!(w.condition(), WeatherCondition::Storm);
    }

    #[test]
    fn test_travel_speed_new_conditions() {
        let mut w = Weather::default();
        w.current_event = WeatherCondition::HeavyRain;
        assert!(w.travel_speed_multiplier() < 0.7);

        w.current_event = WeatherCondition::Snow;
        assert!(w.travel_speed_multiplier() < 0.7);
    }

    #[test]
    fn test_park_multiplier_new_conditions() {
        let mut w = Weather::default();
        w.current_event = WeatherCondition::HeavyRain;
        assert!(w.park_multiplier() < 0.5);

        w.current_event = WeatherCondition::Overcast;
        assert!(w.park_multiplier() < 0.8);

        w.current_event = WeatherCondition::Snow;
        assert!(w.park_multiplier() < 0.3);
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

        // Start: Sunny, day 1, hour 0
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.precipitation_intensity = 0.0;
            weather.temperature = 20.0;
            weather.last_update_day = 1;
            weather.last_update_hour = 5;
            weather.season = Season::Spring;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 1;
            clock.hour = 6.0; // different hour to trigger update
        }

        // Force rainy atmospheric state by setting cloud_cover and precipitation
        // before the system runs. The system will derive Rain from atmosphere.
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.cloud_cover = 0.8;
            weather.precipitation_intensity = 0.3;
        }

        app.update();

        // Read events
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
            weather.precipitation_intensity = 0.0;
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

        // Set storm-level atmospheric state
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.cloud_cover = 0.95;
            weather.precipitation_intensity = 0.85;
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

        // Set up non-extreme state, then push temp to extreme
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.05;
            weather.precipitation_intensity = 0.0;
            weather.temperature = 50.0; // will smooth but stay > 35C
            weather.last_update_day = 120;
            weather.last_update_hour = 14;
            weather.season = Season::Summer;
            weather.event_days_remaining = 5;
            weather.prev_extreme = false; // previous tick was NOT extreme
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 120; // Summer day
            clock.hour = 15.0; // peak heat hour
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

        // Set up non-extreme state, then push temp to extreme cold
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.2;
            weather.precipitation_intensity = 0.0;
            weather.temperature = -25.0; // will smooth but stay < -5C
            weather.last_update_day = 300;
            weather.last_update_hour = 5;
            weather.season = Season::Winter;
            weather.event_days_remaining = 5;
            weather.prev_extreme = false; // previous tick was NOT extreme
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 300; // Winter day
            clock.hour = 6.0; // trough temp hour
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

        // Set up: end of Spring (day 90)
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.precipitation_intensity = 0.0;
            weather.temperature = 20.0;
            weather.last_update_day = 90;
            weather.last_update_hour = 11;
            weather.season = Season::Spring;
        }
        {
            let mut clock = app.world_mut().resource_mut::<GameClock>();
            clock.day = 91; // Summer starts
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

        // Set up: Sunny, clear, mild temperature, Spring
        {
            let mut weather = app.world_mut().resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Sunny;
            weather.cloud_cover = 0.1;
            weather.precipitation_intensity = 0.0;
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

        // The condition should remain Sunny (low cloud cover, no precipitation),
        // season stays Spring (day 1), and temperature is mild.
        // No event should fire.
        assert!(
            fired.is_empty(),
            "No event should fire when weather does not change; got {} events",
            fired.len()
        );
    }

    #[test]
    fn test_is_extreme_weather_helper() {
        // Storm is always extreme
        assert!(is_extreme_weather(WeatherCondition::Storm, 20.0));
        // Heat wave
        assert!(is_extreme_weather(WeatherCondition::Sunny, 36.0));
        // Cold snap
        assert!(is_extreme_weather(WeatherCondition::Sunny, -6.0));
        // Normal conditions
        assert!(!is_extreme_weather(WeatherCondition::Sunny, 20.0));
        assert!(!is_extreme_weather(WeatherCondition::Rain, 10.0));
        assert!(!is_extreme_weather(WeatherCondition::Snow, -3.0));
    }

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
        // Issue requirement: Tropical zone has winter_low=65F (~18C), no snow
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
        // Verify no snow in any season for tropical
        for &season in &[Season::Spring, Season::Summer, Season::Autumn, Season::Winter] {
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
        // Issue requirement: Subarctic zone has winter_low=-30F (~-34C), heavy snow
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
        // Subarctic should have heavy precipitation chance in winter
        assert!(
            params.precipitation_chance >= 0.12,
            "Subarctic winter should have high precipitation chance, got {}",
            params.precipitation_chance
        );
    }

    #[test]
    fn test_arid_very_low_precipitation() {
        // Issue requirement: Arid zone has very low precipitation chance
        for &season in &[Season::Spring, Season::Summer, Season::Autumn, Season::Winter] {
            let params = ClimateZone::Arid.season_params(season);
            assert!(
                params.precipitation_chance <= 0.05,
                "Arid {:?} precipitation chance should be very low, got {}",
                season,
                params.precipitation_chance
            );
        }
        // Summer should be the driest
        let summer = ClimateZone::Arid.season_params(Season::Summer);
        assert!(
            summer.precipitation_chance <= 0.02,
            "Arid summer should be extremely dry, got {}",
            summer.precipitation_chance
        );
    }

    #[test]
    fn test_temperate_backward_compatible() {
        // Temperate zone parameters should match the original hardcoded values
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
            for &season in &[Season::Spring, Season::Summer, Season::Autumn, Season::Winter] {
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
        // Continental should have large temperature difference between winter and summer
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
        // Annual temperature range should be narrow (< 25C from coldest to hottest)
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
}

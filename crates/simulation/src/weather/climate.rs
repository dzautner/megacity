use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::types::Season;

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

//! Configuration and state types for seasonal rendering effects.

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use crate::weather::{Season, WeatherCondition};
use crate::Saveable;

// =============================================================================
// Configuration
// =============================================================================

/// Player-toggleable configuration for each seasonal effect category.
/// Allows disabling expensive effects for performance.
#[derive(
    Resource, Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode, PartialEq,
)]
pub struct SeasonalEffectsConfig {
    /// Enable falling leaf particles in autumn.
    pub leaves_enabled: bool,
    /// Enable snow accumulation tint on building roofs in winter.
    pub snow_roofs_enabled: bool,
    /// Enable flower particles near parks in spring.
    pub flowers_enabled: bool,
    /// Enable heat shimmer effect on hot summer days.
    pub heat_shimmer_enabled: bool,
    /// Enable rain streak particles during rain events.
    pub rain_streaks_enabled: bool,
    /// Enable storm darkening and lightning flash effects.
    pub storm_effects_enabled: bool,
    /// Enable snowflake particles during winter precipitation.
    pub snowflakes_enabled: bool,
    /// Enable extended shadow length in summer.
    pub summer_shadows_enabled: bool,
    /// Enable brighter lighting in spring.
    pub spring_brightness_enabled: bool,
}

impl Default for SeasonalEffectsConfig {
    fn default() -> Self {
        Self {
            leaves_enabled: true,
            snow_roofs_enabled: true,
            flowers_enabled: true,
            heat_shimmer_enabled: true,
            rain_streaks_enabled: true,
            storm_effects_enabled: true,
            snowflakes_enabled: true,
            summer_shadows_enabled: true,
            spring_brightness_enabled: true,
        }
    }
}

// =============================================================================
// Rendering State
// =============================================================================

/// Seasonal rendering effect state consumed by the rendering layer.
///
/// Each field represents the current intensity or activation state of a
/// visual effect. The simulation system updates these values each slow tick.
#[derive(
    Resource, Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode, PartialEq,
)]
pub struct SeasonalRenderingState {
    // --- Autumn ---
    /// Falling leaf particle intensity (0.0 - 1.0). Controls particle spawn rate.
    pub leaf_intensity: f32,
    /// Number of grid cells with trees available for leaf spawning.
    pub leaf_source_cells: u32,

    // --- Winter ---
    /// Snow-on-roof tint intensity (0.0 - 1.0). 0 = no tint, 1 = full white.
    pub snow_roof_intensity: f32,
    /// Number of building cells eligible for snow roof rendering.
    pub snow_roof_cells: u32,
    /// Snowflake particle intensity (0.0 - 1.0). Controls particle spawn rate.
    pub snowflake_intensity: f32,

    // --- Spring ---
    /// Flower particle intensity (0.0 - 1.0). Controls particle spawn rate near parks.
    pub flower_intensity: f32,
    /// Number of park/grass cells available for flower spawning.
    pub flower_source_cells: u32,
    /// Spring ambient brightness boost (0.0 - 0.3).
    pub spring_brightness: f32,

    // --- Summer ---
    /// Heat shimmer effect intensity (0.0 - 1.0). Only active on hot days.
    pub heat_shimmer_intensity: f32,
    /// Shadow length multiplier (1.0 = normal, > 1.0 = longer summer shadows).
    pub shadow_multiplier: f32,

    // --- Rain ---
    /// Rain streak particle intensity (0.0 - 1.0). Scales with precipitation.
    pub rain_streak_intensity: f32,

    // --- Storm ---
    /// Sky darkening intensity (0.0 = clear, 1.0 = fully dark storm sky).
    pub storm_darkening: f32,
    /// Whether a lightning flash is active this tick.
    pub lightning_active: bool,
    /// Remaining ticks for the current lightning flash (0 = no flash).
    pub lightning_timer: u32,

    // --- Metadata ---
    /// The current active season as a u8 (0=Spring, 1=Summer, 2=Autumn, 3=Winter).
    /// Use `active_season()` to convert to `Season`.
    pub current_season_id: u8,
    /// The current weather condition as a u8 (0=Sunny, 1=PartlyCloudy, 2=Overcast,
    /// 3=Rain, 4=HeavyRain, 5=Snow, 6=Storm).
    /// Use `active_condition()` to convert to `WeatherCondition`.
    pub current_condition_id: u8,
}

impl Default for SeasonalRenderingState {
    fn default() -> Self {
        Self {
            leaf_intensity: 0.0,
            leaf_source_cells: 0,
            snow_roof_intensity: 0.0,
            snow_roof_cells: 0,
            snowflake_intensity: 0.0,
            flower_intensity: 0.0,
            flower_source_cells: 0,
            spring_brightness: 0.0,
            heat_shimmer_intensity: 0.0,
            shadow_multiplier: 1.0,
            rain_streak_intensity: 0.0,
            storm_darkening: 0.0,
            lightning_active: false,
            lightning_timer: 0,
            current_season_id: 0,    // Spring
            current_condition_id: 0, // Sunny
        }
    }
}

impl SeasonalRenderingState {
    /// Convert the stored season id to a `Season` enum.
    pub fn active_season(&self) -> Season {
        season_from_id(self.current_season_id)
    }

    /// Convert the stored condition id to a `WeatherCondition` enum.
    pub fn active_condition(&self) -> WeatherCondition {
        condition_from_id(self.current_condition_id)
    }
}

/// Convert a `Season` to its u8 id.
pub(crate) fn season_to_id(season: Season) -> u8 {
    match season {
        Season::Spring => 0,
        Season::Summer => 1,
        Season::Autumn => 2,
        Season::Winter => 3,
    }
}

/// Convert a u8 id to a `Season`.
fn season_from_id(id: u8) -> Season {
    match id {
        0 => Season::Spring,
        1 => Season::Summer,
        2 => Season::Autumn,
        _ => Season::Winter,
    }
}

/// Convert a `WeatherCondition` to its u8 id.
pub(crate) fn condition_to_id(condition: WeatherCondition) -> u8 {
    match condition {
        WeatherCondition::Sunny => 0,
        WeatherCondition::PartlyCloudy => 1,
        WeatherCondition::Overcast => 2,
        WeatherCondition::Rain => 3,
        WeatherCondition::HeavyRain => 4,
        WeatherCondition::Snow => 5,
        WeatherCondition::Storm => 6,
    }
}

/// Convert a u8 id to a `WeatherCondition`.
fn condition_from_id(id: u8) -> WeatherCondition {
    match id {
        0 => WeatherCondition::Sunny,
        1 => WeatherCondition::PartlyCloudy,
        2 => WeatherCondition::Overcast,
        3 => WeatherCondition::Rain,
        4 => WeatherCondition::HeavyRain,
        5 => WeatherCondition::Snow,
        _ => WeatherCondition::Storm,
    }
}

impl Saveable for SeasonalRenderingState {
    const SAVE_KEY: &'static str = "seasonal_rendering";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if everything is at default (no active effects).
        if self.leaf_intensity == 0.0
            && self.snow_roof_intensity == 0.0
            && self.snowflake_intensity == 0.0
            && self.flower_intensity == 0.0
            && self.heat_shimmer_intensity == 0.0
            && self.rain_streak_intensity == 0.0
            && self.storm_darkening == 0.0
        {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

impl Saveable for SeasonalEffectsConfig {
    const SAVE_KEY: &'static str = "seasonal_effects_config";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if all defaults (all enabled).
        if *self == Self::default() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

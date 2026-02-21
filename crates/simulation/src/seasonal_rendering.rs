//! Enhanced seasonal rendering effects (WEATHER-018).
//!
//! Tracks visual effect state for each season: falling leaves in autumn,
//! snow accumulation on building roofs in winter, flower particles near parks
//! in spring, heat shimmer in summer, rain streaks during rain events,
//! storm darkening and lightning flashes during storms, and snowflake
//! particles during winter precipitation.
//!
//! The `SeasonalRenderingState` resource holds intensity values and active
//! effect flags that the rendering layer reads each frame. The
//! `update_seasonal_rendering` system runs every slow tick, reading the
//! current `Weather` resource to derive which effects should be active and
//! at what intensity.
//!
//! All effects are toggleable via `SeasonalEffectsConfig` for performance
//! tuning.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::WorldGrid;
use crate::snow::SnowGrid;
use crate::trees::TreeGrid;
use crate::weather::{Season, Weather, WeatherCondition};
use crate::Saveable;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Maximum leaf particle intensity (arbitrary units, 0.0 - 1.0).
const MAX_LEAF_INTENSITY: f32 = 1.0;

/// Leaf intensity ramp-up per slow tick during autumn.
const LEAF_RAMP_RATE: f32 = 0.05;

/// Leaf intensity decay rate per slow tick outside autumn.
const LEAF_DECAY_RATE: f32 = 0.1;

/// Maximum flower particle intensity (0.0 - 1.0).
const MAX_FLOWER_INTENSITY: f32 = 1.0;

/// Flower intensity ramp-up per slow tick during spring.
const FLOWER_RAMP_RATE: f32 = 0.05;

/// Flower intensity decay rate per slow tick outside spring.
const FLOWER_DECAY_RATE: f32 = 0.1;

/// Maximum snow roof tint intensity (0.0 - 1.0), representing full white overlay.
const MAX_SNOW_ROOF_INTENSITY: f32 = 1.0;

/// Snow roof tint ramp-up per slow tick when snowing.
const SNOW_ROOF_RAMP_RATE: f32 = 0.04;

/// Snow roof tint decay rate per slow tick when not snowing and above freezing.
const SNOW_ROOF_DECAY_RATE: f32 = 0.02;

/// Temperature threshold (Celsius) above which heat shimmer can appear.
const HEAT_SHIMMER_THRESHOLD: f32 = 30.0;

/// Maximum heat shimmer intensity (0.0 - 1.0).
const MAX_HEAT_SHIMMER_INTENSITY: f32 = 1.0;

/// Rain streak intensity per inch/hr of precipitation (clamped to 1.0).
const RAIN_INTENSITY_SCALE: f32 = 0.5;

/// Maximum rain streak intensity (0.0 - 1.0).
const MAX_RAIN_INTENSITY: f32 = 1.0;

/// Maximum snowflake particle intensity (0.0 - 1.0).
const MAX_SNOWFLAKE_INTENSITY: f32 = 1.0;

/// Snowflake intensity per inch/hr of precipitation (clamped).
const SNOWFLAKE_INTENSITY_SCALE: f32 = 1.0;

/// Storm sky darkening intensity (0.0 = clear, 1.0 = fully dark).
const STORM_DARKENING_INTENSITY: f32 = 0.7;

/// Storm darkening ramp-up rate per slow tick.
const STORM_DARKEN_RAMP_RATE: f32 = 0.15;

/// Storm darkening decay rate per slow tick.
const STORM_DARKEN_DECAY_RATE: f32 = 0.1;

/// Lightning flash duration in slow ticks (each flash lasts ~1 tick).
const LIGHTNING_FLASH_DURATION: u32 = 1;

/// Probability per slow tick of a lightning flash during a storm (0.0 - 1.0).
const LIGHTNING_FLASH_PROBABILITY: f32 = 0.3;

/// Summer shadow length multiplier (longer shadows = more dramatic).
const SUMMER_SHADOW_MULTIPLIER: f32 = 1.5;

/// Spring brightness boost (fraction added to ambient light, 0.0 - 0.3).
const SPRING_BRIGHTNESS_BOOST: f32 = 0.15;

/// Freezing point for snow roof logic (Celsius).
const FREEZING_POINT_C: f32 = 0.0;

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
fn season_to_id(season: Season) -> u8 {
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
fn condition_to_id(condition: WeatherCondition) -> u8 {
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

// =============================================================================
// Pure helper functions (testable without ECS)
// =============================================================================

/// Calculate falling leaf intensity for this tick.
/// Leaves only appear in autumn. Intensity ramps up over time and decays outside autumn.
pub fn compute_leaf_intensity(current: f32, season: Season, enabled: bool) -> f32 {
    if !enabled {
        return 0.0;
    }
    match season {
        Season::Autumn => (current + LEAF_RAMP_RATE).min(MAX_LEAF_INTENSITY),
        _ => (current - LEAF_DECAY_RATE).max(0.0),
    }
}

/// Calculate flower particle intensity for this tick.
/// Flowers only appear in spring.
pub fn compute_flower_intensity(current: f32, season: Season, enabled: bool) -> f32 {
    if !enabled {
        return 0.0;
    }
    match season {
        Season::Spring => (current + FLOWER_RAMP_RATE).min(MAX_FLOWER_INTENSITY),
        _ => (current - FLOWER_DECAY_RATE).max(0.0),
    }
}

/// Calculate snow roof tint intensity based on weather and snow depth.
/// Ramps up when snowing and below freezing, decays when above freezing.
pub fn compute_snow_roof_intensity(
    current: f32,
    weather: &Weather,
    avg_snow_depth: f32,
    enabled: bool,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    let is_snowing = matches!(
        weather.current_event,
        WeatherCondition::Snow | WeatherCondition::Storm
    ) && weather.temperature < FREEZING_POINT_C;

    if is_snowing || avg_snow_depth > 1.0 {
        // Ramp up based on snow depth presence
        let target = (avg_snow_depth / 6.0).min(MAX_SNOW_ROOF_INTENSITY);
        let ramped = current + SNOW_ROOF_RAMP_RATE;
        ramped.min(target.max(current))
    } else if weather.temperature > FREEZING_POINT_C {
        // Decay when not snowing and above freezing
        (current - SNOW_ROOF_DECAY_RATE).max(0.0)
    } else {
        // Below freezing but not snowing: hold current intensity
        current
    }
}

/// Calculate heat shimmer intensity based on temperature.
/// Only appears in summer when temperature exceeds the threshold.
pub fn compute_heat_shimmer_intensity(temperature: f32, season: Season, enabled: bool) -> f32 {
    if !enabled || season != Season::Summer {
        return 0.0;
    }
    if temperature < HEAT_SHIMMER_THRESHOLD {
        return 0.0;
    }
    let excess = temperature - HEAT_SHIMMER_THRESHOLD;
    // Scale: 0 at threshold, 1.0 at threshold + 10C
    (excess / 10.0).min(MAX_HEAT_SHIMMER_INTENSITY)
}

/// Calculate shadow multiplier for summer.
pub fn compute_shadow_multiplier(season: Season, enabled: bool) -> f32 {
    if !enabled {
        return 1.0;
    }
    match season {
        Season::Summer => SUMMER_SHADOW_MULTIPLIER,
        _ => 1.0,
    }
}

/// Calculate spring brightness boost.
pub fn compute_spring_brightness(season: Season, enabled: bool) -> f32 {
    if !enabled {
        return 0.0;
    }
    match season {
        Season::Spring => SPRING_BRIGHTNESS_BOOST,
        _ => 0.0,
    }
}

/// Calculate rain streak intensity from precipitation.
/// Active during Rain, HeavyRain, or Storm conditions.
pub fn compute_rain_intensity(weather: &Weather, enabled: bool) -> f32 {
    if !enabled {
        return 0.0;
    }
    match weather.current_event {
        WeatherCondition::Rain | WeatherCondition::HeavyRain | WeatherCondition::Storm => {
            // Only rain, not snow
            if weather.temperature >= FREEZING_POINT_C {
                (weather.precipitation_intensity * RAIN_INTENSITY_SCALE).min(MAX_RAIN_INTENSITY)
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

/// Calculate snowflake particle intensity from precipitation during winter.
pub fn compute_snowflake_intensity(weather: &Weather, enabled: bool) -> f32 {
    if !enabled {
        return 0.0;
    }
    if weather.temperature >= FREEZING_POINT_C {
        return 0.0;
    }
    match weather.current_event {
        WeatherCondition::Snow | WeatherCondition::Storm => (weather.precipitation_intensity
            * SNOWFLAKE_INTENSITY_SCALE)
            .min(MAX_SNOWFLAKE_INTENSITY),
        _ => 0.0,
    }
}

/// Calculate storm sky darkening.
/// Ramps up during storms, decays otherwise.
pub fn compute_storm_darkening(current: f32, weather: &Weather, enabled: bool) -> f32 {
    if !enabled {
        return 0.0;
    }
    match weather.current_event {
        WeatherCondition::Storm => {
            (current + STORM_DARKEN_RAMP_RATE).min(STORM_DARKENING_INTENSITY)
        }
        _ => (current - STORM_DARKEN_DECAY_RATE).max(0.0),
    }
}

/// Determine if a lightning flash should trigger this tick.
/// Uses a deterministic hash for reproducibility.
pub fn should_trigger_lightning(weather: &Weather, tick_hash: u32, enabled: bool) -> bool {
    if !enabled {
        return false;
    }
    if weather.current_event != WeatherCondition::Storm {
        return false;
    }
    let threshold = (LIGHTNING_FLASH_PROBABILITY * 10000.0) as u32;
    (tick_hash % 10000) < threshold
}

/// Count tree cells in the grid (for leaf particle source locations).
pub fn count_tree_cells(tree_grid: &TreeGrid) -> u32 {
    tree_grid.cells.iter().filter(|&&has_tree| has_tree).count() as u32
}

/// Count building cells in the grid (for snow roof rendering).
pub fn count_building_cells(world_grid: &WorldGrid) -> u32 {
    let mut count = 0u32;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if world_grid.get(x, y).building_id.is_some() {
                count += 1;
            }
        }
    }
    count
}

/// Count grass cells without buildings or roads (for flower spawning in spring).
/// These represent open green spaces / park-like areas.
pub fn count_flower_cells(world_grid: &WorldGrid, tree_grid: &TreeGrid) -> u32 {
    let mut count = 0u32;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = world_grid.get(x, y);
            // Flowers spawn on grass cells that have trees (park-like areas)
            // or on unbuilt grass cells in residential zones (gardens)
            if cell.cell_type == crate::grid::CellType::Grass
                && cell.building_id.is_none()
                && (tree_grid.has_tree(x, y)
                    || matches!(
                        cell.zone,
                        crate::grid::ZoneType::ResidentialLow
                            | crate::grid::ZoneType::ResidentialMedium
                    ))
            {
                count += 1;
            }
        }
    }
    count
}

// =============================================================================
// System
// =============================================================================

/// Main seasonal rendering update system. Runs every slow tick.
///
/// Reads current weather, season, snow depth, and grid state to compute
/// rendering effect intensities for the rendering layer.
#[allow(clippy::too_many_arguments)]
pub fn update_seasonal_rendering(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    world_grid: Res<WorldGrid>,
    tree_grid: Res<TreeGrid>,
    snow_grid: Res<SnowGrid>,
    config: Res<SeasonalEffectsConfig>,
    mut state: ResMut<SeasonalRenderingState>,
    tick_counter: Res<crate::TickCounter>,
) {
    if !timer.should_run() {
        return;
    }

    let season = weather.season;

    // --- Autumn: falling leaves ---
    state.leaf_intensity =
        compute_leaf_intensity(state.leaf_intensity, season, config.leaves_enabled);
    state.leaf_source_cells = if config.leaves_enabled && season == Season::Autumn {
        count_tree_cells(&tree_grid)
    } else if state.leaf_intensity > 0.0 {
        // Still decaying, keep last count
        state.leaf_source_cells
    } else {
        0
    };

    // --- Winter: snow on roofs ---
    let avg_snow = snow_grid.average_depth();
    state.snow_roof_intensity = compute_snow_roof_intensity(
        state.snow_roof_intensity,
        &weather,
        avg_snow,
        config.snow_roofs_enabled,
    );
    state.snow_roof_cells = if config.snow_roofs_enabled
        && (season == Season::Winter || state.snow_roof_intensity > 0.0)
    {
        count_building_cells(&world_grid)
    } else {
        0
    };

    // --- Winter: snowflake particles ---
    state.snowflake_intensity = compute_snowflake_intensity(&weather, config.snowflakes_enabled);

    // --- Spring: flower particles ---
    state.flower_intensity =
        compute_flower_intensity(state.flower_intensity, season, config.flowers_enabled);
    state.flower_source_cells = if config.flowers_enabled && season == Season::Spring {
        count_flower_cells(&world_grid, &tree_grid)
    } else if state.flower_intensity > 0.0 {
        state.flower_source_cells
    } else {
        0
    };

    // --- Spring: brightness boost ---
    state.spring_brightness = compute_spring_brightness(season, config.spring_brightness_enabled);

    // --- Summer: heat shimmer ---
    state.heat_shimmer_intensity =
        compute_heat_shimmer_intensity(weather.temperature, season, config.heat_shimmer_enabled);

    // --- Summer: longer shadows ---
    state.shadow_multiplier = compute_shadow_multiplier(season, config.summer_shadows_enabled);

    // --- Rain: rain streaks ---
    state.rain_streak_intensity = compute_rain_intensity(&weather, config.rain_streaks_enabled);

    // --- Storm: sky darkening ---
    state.storm_darkening = compute_storm_darkening(
        state.storm_darkening,
        &weather,
        config.storm_effects_enabled,
    );

    // --- Storm: lightning flashes ---
    if state.lightning_timer > 0 {
        state.lightning_timer -= 1;
        state.lightning_active = state.lightning_timer > 0;
    } else {
        let tick_hash = tick_counter.0 as u32;
        if should_trigger_lightning(&weather, tick_hash, config.storm_effects_enabled) {
            state.lightning_active = true;
            state.lightning_timer = LIGHTNING_FLASH_DURATION;
        } else {
            state.lightning_active = false;
        }
    }

    // --- Metadata ---
    state.current_season_id = season_to_id(season);
    state.current_condition_id = condition_to_id(weather.current_event);
}

// =============================================================================
// Plugin
// =============================================================================

pub struct SeasonalRenderingPlugin;

impl Plugin for SeasonalRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SeasonalRenderingState>()
            .init_resource::<SeasonalEffectsConfig>()
            .add_systems(
                FixedUpdate,
                update_seasonal_rendering
                    .after(crate::weather::update_weather)
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        let world = app.world_mut();
        let mut registry = world.resource_mut::<crate::SaveableRegistry>();
        registry.register::<SeasonalRenderingState>();
        registry.register::<SeasonalEffectsConfig>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Helper: create a Weather with sensible defaults for testing
    // -------------------------------------------------------------------------

    fn test_weather(season: Season, condition: WeatherCondition, temp: f32) -> Weather {
        Weather {
            season,
            current_event: condition,
            temperature: temp,
            precipitation_intensity: match condition {
                WeatherCondition::Rain => 0.3,
                WeatherCondition::HeavyRain => 1.5,
                WeatherCondition::Storm => 2.0,
                WeatherCondition::Snow => 0.5,
                _ => 0.0,
            },
            ..Default::default()
        }
    }

    // -------------------------------------------------------------------------
    // Leaf intensity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_leaf_intensity_ramps_in_autumn() {
        let intensity = compute_leaf_intensity(0.0, Season::Autumn, true);
        assert!(
            intensity > 0.0,
            "leaf intensity should increase in autumn, got {}",
            intensity
        );
    }

    #[test]
    fn test_leaf_intensity_capped() {
        let intensity = compute_leaf_intensity(0.98, Season::Autumn, true);
        assert!(
            intensity <= MAX_LEAF_INTENSITY,
            "leaf intensity should not exceed {}, got {}",
            MAX_LEAF_INTENSITY,
            intensity
        );
    }

    #[test]
    fn test_leaf_intensity_decays_outside_autumn() {
        let intensity = compute_leaf_intensity(0.5, Season::Summer, true);
        assert!(
            intensity < 0.5,
            "leaf intensity should decay outside autumn, got {}",
            intensity
        );
    }

    #[test]
    fn test_leaf_intensity_zero_when_disabled() {
        let intensity = compute_leaf_intensity(0.5, Season::Autumn, false);
        assert_eq!(intensity, 0.0, "disabled leaves should have 0 intensity");
    }

    #[test]
    fn test_leaf_intensity_decay_floors_at_zero() {
        let intensity = compute_leaf_intensity(0.01, Season::Winter, true);
        assert!(
            intensity >= 0.0,
            "leaf intensity should not go below 0, got {}",
            intensity
        );
    }

    // -------------------------------------------------------------------------
    // Flower intensity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_flower_intensity_ramps_in_spring() {
        let intensity = compute_flower_intensity(0.0, Season::Spring, true);
        assert!(
            intensity > 0.0,
            "flower intensity should increase in spring, got {}",
            intensity
        );
    }

    #[test]
    fn test_flower_intensity_capped() {
        let intensity = compute_flower_intensity(0.98, Season::Spring, true);
        assert!(
            intensity <= MAX_FLOWER_INTENSITY,
            "flower intensity should not exceed {}, got {}",
            MAX_FLOWER_INTENSITY,
            intensity
        );
    }

    #[test]
    fn test_flower_intensity_decays_outside_spring() {
        let intensity = compute_flower_intensity(0.5, Season::Winter, true);
        assert!(
            intensity < 0.5,
            "flower intensity should decay outside spring, got {}",
            intensity
        );
    }

    #[test]
    fn test_flower_intensity_zero_when_disabled() {
        let intensity = compute_flower_intensity(0.5, Season::Spring, false);
        assert_eq!(intensity, 0.0, "disabled flowers should have 0 intensity");
    }

    // -------------------------------------------------------------------------
    // Snow roof intensity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_snow_roof_ramps_when_snowing() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        let intensity = compute_snow_roof_intensity(0.0, &weather, 3.0, true);
        assert!(
            intensity > 0.0,
            "snow roof should ramp up when snowing, got {}",
            intensity
        );
    }

    #[test]
    fn test_snow_roof_decays_above_freezing() {
        let weather = test_weather(Season::Spring, WeatherCondition::Sunny, 10.0);
        let intensity = compute_snow_roof_intensity(0.5, &weather, 0.0, true);
        assert!(
            intensity < 0.5,
            "snow roof should decay above freezing with no snow, got {}",
            intensity
        );
    }

    #[test]
    fn test_snow_roof_holds_below_freezing_no_snow() {
        let weather = test_weather(Season::Winter, WeatherCondition::Sunny, -2.0);
        let intensity = compute_snow_roof_intensity(0.3, &weather, 0.5, true);
        // Below freezing, not snowing, snow depth < 1.0 => should hold
        assert!(
            (intensity - 0.3).abs() < f32::EPSILON,
            "snow roof should hold below freezing with minimal snow, got {}",
            intensity
        );
    }

    #[test]
    fn test_snow_roof_zero_when_disabled() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        let intensity = compute_snow_roof_intensity(0.5, &weather, 6.0, false);
        assert_eq!(intensity, 0.0, "disabled snow roof should have 0 intensity");
    }

    #[test]
    fn test_snow_roof_capped() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, -10.0);
        // With 12 inches, target = 12/6 = 2.0 but capped at MAX_SNOW_ROOF_INTENSITY (1.0)
        let intensity = compute_snow_roof_intensity(0.95, &weather, 12.0, true);
        assert!(
            intensity <= MAX_SNOW_ROOF_INTENSITY,
            "snow roof intensity should not exceed {}, got {}",
            MAX_SNOW_ROOF_INTENSITY,
            intensity
        );
    }

    // -------------------------------------------------------------------------
    // Heat shimmer tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_heat_shimmer_active_in_summer_heat() {
        let intensity = compute_heat_shimmer_intensity(35.0, Season::Summer, true);
        assert!(
            intensity > 0.0,
            "heat shimmer should be active at 35C in summer, got {}",
            intensity
        );
    }

    #[test]
    fn test_heat_shimmer_zero_below_threshold() {
        let intensity = compute_heat_shimmer_intensity(25.0, Season::Summer, true);
        assert_eq!(
            intensity, 0.0,
            "heat shimmer should be zero below threshold"
        );
    }

    #[test]
    fn test_heat_shimmer_zero_outside_summer() {
        let intensity = compute_heat_shimmer_intensity(35.0, Season::Spring, true);
        assert_eq!(intensity, 0.0, "heat shimmer should be zero outside summer");
    }

    #[test]
    fn test_heat_shimmer_zero_when_disabled() {
        let intensity = compute_heat_shimmer_intensity(40.0, Season::Summer, false);
        assert_eq!(intensity, 0.0, "disabled heat shimmer should be zero");
    }

    #[test]
    fn test_heat_shimmer_scales_with_temperature() {
        let low = compute_heat_shimmer_intensity(32.0, Season::Summer, true);
        let high = compute_heat_shimmer_intensity(38.0, Season::Summer, true);
        assert!(
            high > low,
            "higher temperature should produce more shimmer: {} vs {}",
            high,
            low
        );
    }

    #[test]
    fn test_heat_shimmer_capped() {
        let intensity = compute_heat_shimmer_intensity(50.0, Season::Summer, true);
        assert!(
            intensity <= MAX_HEAT_SHIMMER_INTENSITY,
            "heat shimmer should be capped at {}, got {}",
            MAX_HEAT_SHIMMER_INTENSITY,
            intensity
        );
    }

    // -------------------------------------------------------------------------
    // Shadow multiplier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_shadow_multiplier_summer() {
        let mult = compute_shadow_multiplier(Season::Summer, true);
        assert!(
            (mult - SUMMER_SHADOW_MULTIPLIER).abs() < f32::EPSILON,
            "summer should have shadow multiplier {}, got {}",
            SUMMER_SHADOW_MULTIPLIER,
            mult
        );
    }

    #[test]
    fn test_shadow_multiplier_other_seasons() {
        for season in [Season::Spring, Season::Autumn, Season::Winter] {
            let mult = compute_shadow_multiplier(season, true);
            assert!(
                (mult - 1.0).abs() < f32::EPSILON,
                "{:?} should have shadow multiplier 1.0, got {}",
                season,
                mult
            );
        }
    }

    #[test]
    fn test_shadow_multiplier_disabled() {
        let mult = compute_shadow_multiplier(Season::Summer, false);
        assert!(
            (mult - 1.0).abs() < f32::EPSILON,
            "disabled should have shadow multiplier 1.0, got {}",
            mult
        );
    }

    // -------------------------------------------------------------------------
    // Spring brightness tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_spring_brightness_active() {
        let brightness = compute_spring_brightness(Season::Spring, true);
        assert!(
            (brightness - SPRING_BRIGHTNESS_BOOST).abs() < f32::EPSILON,
            "spring should have brightness boost {}, got {}",
            SPRING_BRIGHTNESS_BOOST,
            brightness
        );
    }

    #[test]
    fn test_spring_brightness_other_seasons() {
        for season in [Season::Summer, Season::Autumn, Season::Winter] {
            let brightness = compute_spring_brightness(season, true);
            assert!(
                brightness.abs() < f32::EPSILON,
                "{:?} should have 0 brightness boost, got {}",
                season,
                brightness
            );
        }
    }

    #[test]
    fn test_spring_brightness_disabled() {
        let brightness = compute_spring_brightness(Season::Spring, false);
        assert!(
            brightness.abs() < f32::EPSILON,
            "disabled should have 0 brightness boost, got {}",
            brightness
        );
    }

    // -------------------------------------------------------------------------
    // Rain intensity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_rain_intensity_during_rain() {
        let weather = test_weather(Season::Summer, WeatherCondition::Rain, 20.0);
        let intensity = compute_rain_intensity(&weather, true);
        assert!(
            intensity > 0.0,
            "rain streaks should be active during rain, got {}",
            intensity
        );
    }

    #[test]
    fn test_rain_intensity_during_heavy_rain() {
        let weather = test_weather(Season::Summer, WeatherCondition::HeavyRain, 20.0);
        let intensity = compute_rain_intensity(&weather, true);
        assert!(
            intensity > 0.0,
            "rain streaks should be active during heavy rain, got {}",
            intensity
        );
    }

    #[test]
    fn test_rain_intensity_during_storm() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let intensity = compute_rain_intensity(&weather, true);
        assert!(
            intensity > 0.0,
            "rain streaks should be active during storm, got {}",
            intensity
        );
    }

    #[test]
    fn test_rain_intensity_zero_when_sunny() {
        let weather = test_weather(Season::Summer, WeatherCondition::Sunny, 20.0);
        let intensity = compute_rain_intensity(&weather, true);
        assert_eq!(intensity, 0.0, "rain streaks should be zero when sunny");
    }

    #[test]
    fn test_rain_intensity_zero_when_below_freezing() {
        let weather = test_weather(Season::Winter, WeatherCondition::Storm, -5.0);
        let intensity = compute_rain_intensity(&weather, true);
        assert_eq!(
            intensity, 0.0,
            "rain streaks should be zero below freezing (snow instead)"
        );
    }

    #[test]
    fn test_rain_intensity_disabled() {
        let weather = test_weather(Season::Summer, WeatherCondition::Rain, 20.0);
        let intensity = compute_rain_intensity(&weather, false);
        assert_eq!(intensity, 0.0, "disabled rain should be zero");
    }

    #[test]
    fn test_rain_intensity_capped() {
        let mut weather = test_weather(Season::Summer, WeatherCondition::HeavyRain, 20.0);
        weather.precipitation_intensity = 10.0;
        let intensity = compute_rain_intensity(&weather, true);
        assert!(
            intensity <= MAX_RAIN_INTENSITY,
            "rain intensity should be capped at {}, got {}",
            MAX_RAIN_INTENSITY,
            intensity
        );
    }

    // -------------------------------------------------------------------------
    // Snowflake intensity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_snowflake_during_snow() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        let intensity = compute_snowflake_intensity(&weather, true);
        assert!(
            intensity > 0.0,
            "snowflakes should be active during snow, got {}",
            intensity
        );
    }

    #[test]
    fn test_snowflake_during_winter_storm() {
        let weather = test_weather(Season::Winter, WeatherCondition::Storm, -5.0);
        let intensity = compute_snowflake_intensity(&weather, true);
        assert!(
            intensity > 0.0,
            "snowflakes should be active during winter storm, got {}",
            intensity
        );
    }

    #[test]
    fn test_snowflake_zero_above_freezing() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, 5.0);
        let intensity = compute_snowflake_intensity(&weather, true);
        assert_eq!(intensity, 0.0, "snowflakes should be zero above freezing");
    }

    #[test]
    fn test_snowflake_zero_when_sunny() {
        let weather = test_weather(Season::Winter, WeatherCondition::Sunny, -5.0);
        let intensity = compute_snowflake_intensity(&weather, true);
        assert_eq!(intensity, 0.0, "snowflakes should be zero when sunny");
    }

    #[test]
    fn test_snowflake_disabled() {
        let weather = test_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        let intensity = compute_snowflake_intensity(&weather, false);
        assert_eq!(intensity, 0.0, "disabled snowflakes should be zero");
    }

    #[test]
    fn test_snowflake_capped() {
        let mut weather = test_weather(Season::Winter, WeatherCondition::Snow, -5.0);
        weather.precipitation_intensity = 5.0;
        let intensity = compute_snowflake_intensity(&weather, true);
        assert!(
            intensity <= MAX_SNOWFLAKE_INTENSITY,
            "snowflake intensity should be capped at {}, got {}",
            MAX_SNOWFLAKE_INTENSITY,
            intensity
        );
    }

    // -------------------------------------------------------------------------
    // Storm darkening tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_storm_darkening_ramps_during_storm() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let darkening = compute_storm_darkening(0.0, &weather, true);
        assert!(
            darkening > 0.0,
            "storm darkening should ramp during storm, got {}",
            darkening
        );
    }

    #[test]
    fn test_storm_darkening_decays_after_storm() {
        let weather = test_weather(Season::Summer, WeatherCondition::Sunny, 20.0);
        let darkening = compute_storm_darkening(0.5, &weather, true);
        assert!(
            darkening < 0.5,
            "storm darkening should decay after storm, got {}",
            darkening
        );
    }

    #[test]
    fn test_storm_darkening_capped() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let darkening = compute_storm_darkening(0.65, &weather, true);
        assert!(
            darkening <= STORM_DARKENING_INTENSITY,
            "storm darkening should be capped at {}, got {}",
            STORM_DARKENING_INTENSITY,
            darkening
        );
    }

    #[test]
    fn test_storm_darkening_disabled() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let darkening = compute_storm_darkening(0.5, &weather, false);
        assert_eq!(darkening, 0.0, "disabled storm darkening should be zero");
    }

    #[test]
    fn test_storm_darkening_floors_at_zero() {
        let weather = test_weather(Season::Summer, WeatherCondition::Sunny, 20.0);
        let darkening = compute_storm_darkening(0.05, &weather, true);
        assert!(
            darkening >= 0.0,
            "storm darkening should not go below 0, got {}",
            darkening
        );
    }

    // -------------------------------------------------------------------------
    // Lightning tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_lightning_only_during_storm() {
        let weather = test_weather(Season::Summer, WeatherCondition::Sunny, 20.0);
        // Try many tick values; none should trigger lightning outside storm
        for tick in 0..100 {
            assert!(
                !should_trigger_lightning(&weather, tick, true),
                "lightning should not trigger outside storm (tick {})",
                tick
            );
        }
    }

    #[test]
    fn test_lightning_disabled() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        for tick in 0..100 {
            assert!(
                !should_trigger_lightning(&weather, tick, false),
                "disabled lightning should never trigger (tick {})",
                tick
            );
        }
    }

    #[test]
    fn test_lightning_deterministic() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        let r1 = should_trigger_lightning(&weather, 42, true);
        let r2 = should_trigger_lightning(&weather, 42, true);
        assert_eq!(r1, r2, "same inputs should give same result");
    }

    #[test]
    fn test_lightning_can_trigger_during_storm() {
        let weather = test_weather(Season::Summer, WeatherCondition::Storm, 20.0);
        // With 30% probability, among 100 ticks we should get at least one
        let count = (0..100)
            .filter(|&tick| should_trigger_lightning(&weather, tick, true))
            .count();
        assert!(
            count > 0,
            "lightning should trigger at least once in 100 ticks during storm"
        );
    }

    // -------------------------------------------------------------------------
    // Cell counting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_count_tree_cells_empty() {
        let grid = TreeGrid::default();
        assert_eq!(count_tree_cells(&grid), 0);
    }

    #[test]
    fn test_count_tree_cells_some() {
        let mut grid = TreeGrid::default();
        grid.set(5, 5, true);
        grid.set(10, 10, true);
        grid.set(15, 15, true);
        assert_eq!(count_tree_cells(&grid), 3);
    }

    #[test]
    fn test_count_building_cells_empty() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert_eq!(count_building_cells(&grid), 0);
    }

    #[test]
    fn test_count_building_cells_some() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Simulate building presence via building_id
        grid.get_mut(5, 5).building_id = Some(Entity::from_raw(1));
        grid.get_mut(10, 10).building_id = Some(Entity::from_raw(2));
        assert_eq!(count_building_cells(&grid), 2);
    }

    // -------------------------------------------------------------------------
    // SeasonalEffectsConfig tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_config_default_all_enabled() {
        let config = SeasonalEffectsConfig::default();
        assert!(config.leaves_enabled);
        assert!(config.snow_roofs_enabled);
        assert!(config.flowers_enabled);
        assert!(config.heat_shimmer_enabled);
        assert!(config.rain_streaks_enabled);
        assert!(config.storm_effects_enabled);
        assert!(config.snowflakes_enabled);
        assert!(config.summer_shadows_enabled);
        assert!(config.spring_brightness_enabled);
    }

    // -------------------------------------------------------------------------
    // SeasonalRenderingState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_default() {
        let state = SeasonalRenderingState::default();
        assert_eq!(state.leaf_intensity, 0.0);
        assert_eq!(state.snow_roof_intensity, 0.0);
        assert_eq!(state.snowflake_intensity, 0.0);
        assert_eq!(state.flower_intensity, 0.0);
        assert_eq!(state.heat_shimmer_intensity, 0.0);
        assert_eq!(state.rain_streak_intensity, 0.0);
        assert_eq!(state.storm_darkening, 0.0);
        assert!(!state.lightning_active);
        assert_eq!(state.lightning_timer, 0);
        assert_eq!(state.shadow_multiplier, 1.0);
        assert_eq!(state.spring_brightness, 0.0);
        assert_eq!(state.leaf_source_cells, 0);
        assert_eq!(state.snow_roof_cells, 0);
        assert_eq!(state.flower_source_cells, 0);
        assert_eq!(state.current_season_id, 0); // Spring
        assert_eq!(state.current_condition_id, 0); // Sunny
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_key_state() {
        assert_eq!(
            SeasonalRenderingState::SAVE_KEY,
            "seasonal_rendering",
            "Save key should be 'seasonal_rendering'"
        );
    }

    #[test]
    fn test_saveable_key_config() {
        assert_eq!(
            SeasonalEffectsConfig::SAVE_KEY,
            "seasonal_effects_config",
            "Save key should be 'seasonal_effects_config'"
        );
    }

    #[test]
    fn test_saveable_state_default_returns_none() {
        let state = SeasonalRenderingState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_config_default_returns_none() {
        let config = SeasonalEffectsConfig::default();
        assert!(
            config.save_to_bytes().is_none(),
            "default config should skip saving"
        );
    }

    #[test]
    fn test_saveable_state_roundtrip() {
        let mut state = SeasonalRenderingState::default();
        state.leaf_intensity = 0.75;
        state.snow_roof_intensity = 0.5;
        state.storm_darkening = 0.3;
        state.lightning_timer = 1;
        state.lightning_active = true;
        state.current_season_id = season_to_id(Season::Autumn);
        state.current_condition_id = condition_to_id(WeatherCondition::Storm);

        let bytes = state.save_to_bytes().expect("should have bytes");
        let loaded = SeasonalRenderingState::load_from_bytes(&bytes);

        assert!((loaded.leaf_intensity - 0.75).abs() < f32::EPSILON);
        assert!((loaded.snow_roof_intensity - 0.5).abs() < f32::EPSILON);
        assert!((loaded.storm_darkening - 0.3).abs() < f32::EPSILON);
        assert_eq!(loaded.lightning_timer, 1);
        assert!(loaded.lightning_active);
        assert_eq!(loaded.active_season(), Season::Autumn);
        assert_eq!(loaded.active_condition(), WeatherCondition::Storm);
    }

    #[test]
    fn test_saveable_config_roundtrip() {
        let mut config = SeasonalEffectsConfig::default();
        config.leaves_enabled = false;
        config.heat_shimmer_enabled = false;

        let bytes = config.save_to_bytes().expect("should have bytes");
        let loaded = SeasonalEffectsConfig::load_from_bytes(&bytes);

        assert!(!loaded.leaves_enabled);
        assert!(loaded.snow_roofs_enabled);
        assert!(loaded.flowers_enabled);
        assert!(!loaded.heat_shimmer_enabled);
        assert!(loaded.rain_streaks_enabled);
        assert!(loaded.storm_effects_enabled);
        assert!(loaded.snowflakes_enabled);
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_positive() {
        assert!(MAX_LEAF_INTENSITY > 0.0);
        assert!(LEAF_RAMP_RATE > 0.0);
        assert!(LEAF_DECAY_RATE > 0.0);
        assert!(MAX_FLOWER_INTENSITY > 0.0);
        assert!(FLOWER_RAMP_RATE > 0.0);
        assert!(FLOWER_DECAY_RATE > 0.0);
        assert!(MAX_SNOW_ROOF_INTENSITY > 0.0);
        assert!(SNOW_ROOF_RAMP_RATE > 0.0);
        assert!(SNOW_ROOF_DECAY_RATE > 0.0);
        assert!(HEAT_SHIMMER_THRESHOLD > 0.0);
        assert!(MAX_HEAT_SHIMMER_INTENSITY > 0.0);
        assert!(RAIN_INTENSITY_SCALE > 0.0);
        assert!(MAX_RAIN_INTENSITY > 0.0);
        assert!(MAX_SNOWFLAKE_INTENSITY > 0.0);
        assert!(SNOWFLAKE_INTENSITY_SCALE > 0.0);
        assert!(STORM_DARKENING_INTENSITY > 0.0);
        assert!(STORM_DARKEN_RAMP_RATE > 0.0);
        assert!(STORM_DARKEN_DECAY_RATE > 0.0);
        assert!(LIGHTNING_FLASH_PROBABILITY > 0.0);
        assert!(LIGHTNING_FLASH_PROBABILITY <= 1.0);
        assert!(SUMMER_SHADOW_MULTIPLIER > 1.0);
        assert!(SPRING_BRIGHTNESS_BOOST > 0.0);
        assert!(SPRING_BRIGHTNESS_BOOST <= 1.0);
    }

    #[test]
    fn test_decay_rates_nonzero() {
        // Ensure effects actually decay and don't linger forever
        assert!(LEAF_DECAY_RATE >= LEAF_RAMP_RATE);
        assert!(FLOWER_DECAY_RATE >= FLOWER_RAMP_RATE);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests: effect combinations
    // -------------------------------------------------------------------------

    #[test]
    fn test_autumn_has_leaves_no_shimmer() {
        let leaves = compute_leaf_intensity(0.0, Season::Autumn, true);
        let shimmer = compute_heat_shimmer_intensity(15.0, Season::Autumn, true);
        assert!(leaves > 0.0, "autumn should have leaves");
        assert_eq!(shimmer, 0.0, "autumn should not have heat shimmer");
    }

    #[test]
    fn test_summer_has_shimmer_no_leaves() {
        let leaves = compute_leaf_intensity(0.0, Season::Summer, true);
        let shimmer = compute_heat_shimmer_intensity(35.0, Season::Summer, true);
        assert_eq!(leaves, 0.0, "summer should not ramp leaves");
        assert!(shimmer > 0.0, "summer should have heat shimmer at 35C");
    }

    #[test]
    fn test_winter_storm_has_snowflakes_and_darkening() {
        let weather = test_weather(Season::Winter, WeatherCondition::Storm, -5.0);
        let snowflakes = compute_snowflake_intensity(&weather, true);
        let darkening = compute_storm_darkening(0.0, &weather, true);
        assert!(snowflakes > 0.0, "winter storm should have snowflakes");
        assert!(darkening > 0.0, "storm should have sky darkening");
    }

    #[test]
    fn test_spring_has_flowers_and_brightness() {
        let flowers = compute_flower_intensity(0.0, Season::Spring, true);
        let brightness = compute_spring_brightness(Season::Spring, true);
        assert!(flowers > 0.0, "spring should have flowers");
        assert!(brightness > 0.0, "spring should have brightness boost");
    }

    #[test]
    fn test_rain_above_freezing_has_rain_streaks_no_snowflakes() {
        let weather = test_weather(Season::Summer, WeatherCondition::Rain, 20.0);
        let rain = compute_rain_intensity(&weather, true);
        let snowflakes = compute_snowflake_intensity(&weather, true);
        assert!(rain > 0.0, "rain above freezing should have rain streaks");
        assert_eq!(
            snowflakes, 0.0,
            "rain above freezing should not have snowflakes"
        );
    }

    #[test]
    fn test_storm_below_freezing_has_snowflakes_no_rain() {
        let weather = test_weather(Season::Winter, WeatherCondition::Storm, -5.0);
        let rain = compute_rain_intensity(&weather, true);
        let snowflakes = compute_snowflake_intensity(&weather, true);
        assert_eq!(
            rain, 0.0,
            "storm below freezing should not have rain streaks"
        );
        assert!(
            snowflakes > 0.0,
            "storm below freezing should have snowflakes"
        );
    }

    #[test]
    fn test_all_effects_disabled() {
        let weather = test_weather(Season::Winter, WeatherCondition::Storm, -5.0);
        assert_eq!(compute_leaf_intensity(0.5, Season::Autumn, false), 0.0);
        assert_eq!(compute_flower_intensity(0.5, Season::Spring, false), 0.0);
        assert_eq!(compute_snow_roof_intensity(0.5, &weather, 6.0, false), 0.0);
        assert_eq!(
            compute_heat_shimmer_intensity(40.0, Season::Summer, false),
            0.0
        );
        assert_eq!(compute_rain_intensity(&weather, false), 0.0);
        assert_eq!(compute_snowflake_intensity(&weather, false), 0.0);
        assert_eq!(compute_storm_darkening(0.5, &weather, false), 0.0);
        assert_eq!(compute_shadow_multiplier(Season::Summer, false), 1.0);
        assert_eq!(compute_spring_brightness(Season::Spring, false), 0.0);
    }

    // -------------------------------------------------------------------------
    // Flower cell counting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_count_flower_cells_empty() {
        let world_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let tree_grid = TreeGrid::default();
        // Default grass cells with no trees and ZoneType::None should not count
        assert_eq!(count_flower_cells(&world_grid, &tree_grid), 0);
    }

    #[test]
    fn test_count_flower_cells_with_trees() {
        let world_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut tree_grid = TreeGrid::default();
        tree_grid.set(5, 5, true);
        tree_grid.set(10, 10, true);
        // These grass cells have trees => count as flower cells
        assert_eq!(count_flower_cells(&world_grid, &tree_grid), 2);
    }

    #[test]
    fn test_count_flower_cells_residential_zones() {
        let mut world_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let tree_grid = TreeGrid::default();
        world_grid.get_mut(3, 3).zone = crate::grid::ZoneType::ResidentialLow;
        world_grid.get_mut(4, 4).zone = crate::grid::ZoneType::ResidentialMedium;
        // These are unbuilt residential grass cells => count as flower cells
        assert_eq!(count_flower_cells(&world_grid, &tree_grid), 2);
    }

    #[test]
    fn test_count_flower_cells_excludes_buildings() {
        let mut world_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut tree_grid = TreeGrid::default();
        tree_grid.set(5, 5, true);
        world_grid.get_mut(5, 5).building_id = Some(Entity::from_raw(1));
        // Has tree but also has building => excluded
        assert_eq!(count_flower_cells(&world_grid, &tree_grid), 0);
    }
}

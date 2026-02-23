//! Pure helper functions for computing seasonal rendering effect intensities.
//!
//! These functions are testable without ECS — they take current values and
//! weather parameters and return the new intensity.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::WorldGrid;
use crate::trees::TreeGrid;
use crate::weather::{Season, Weather, WeatherCondition};

use super::constants::*;

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

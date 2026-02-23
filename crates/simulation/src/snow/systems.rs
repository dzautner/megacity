//! Snow accumulation, melting, and plowing systems.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::weather::{Weather, WeatherCondition};
use crate::SlowTickTimer;

use super::types::{
    SnowGrid, SnowPlowingState, SnowStats, BASE_SNOW_ACCUMULATION_RATE, FREEZING_POINT_C,
    HEATING_INCREASE_PER_6_INCHES, MAX_SNOW_DEPTH, MAX_SNOW_SPEED_REDUCTION, MELT_RATE_PER_DEGREE,
    PLOW_COST_PER_CELL, PLOW_REMOVAL_DEPTH, PLOW_TRIGGER_DEPTH, SNOWMELT_RUNOFF_FACTOR,
    SPEED_REDUCTION_PER_INCH,
};

// =============================================================================
// Pure helper functions (testable without ECS)
// =============================================================================

/// Calculate the snow accumulation amount for this tick given weather conditions.
/// Returns inches of snow to add. Only accumulates during snow events below freezing.
pub fn snow_accumulation_amount(weather: &Weather) -> f32 {
    if weather.temperature >= FREEZING_POINT_C {
        return 0.0;
    }
    match weather.current_event {
        WeatherCondition::Snow => {
            // Scale accumulation with precipitation intensity
            let intensity_factor = (weather.precipitation_intensity * 2.0).clamp(0.5, 3.0);
            BASE_SNOW_ACCUMULATION_RATE * intensity_factor
        }
        WeatherCondition::Storm if weather.temperature < FREEZING_POINT_C => {
            // Heavy snow during storms
            let intensity_factor = (weather.precipitation_intensity * 2.0).clamp(1.0, 4.0);
            BASE_SNOW_ACCUMULATION_RATE * intensity_factor
        }
        _ => 0.0,
    }
}

/// Calculate the snow melt amount for this tick given temperature.
/// Returns inches of snow to remove. Only melts when above freezing.
pub fn snow_melt_amount(temperature: f32) -> f32 {
    if temperature <= FREEZING_POINT_C {
        return 0.0;
    }
    let excess = temperature - FREEZING_POINT_C;
    excess * MELT_RATE_PER_DEGREE
}

/// Calculate the travel speed multiplier for roads based on average road snow depth.
/// Returns a multiplier in [0.2, 1.0] where 1.0 = no snow effect.
pub fn snow_speed_multiplier(avg_road_snow_depth: f32) -> f32 {
    if avg_road_snow_depth <= 0.0 {
        return 1.0;
    }
    let reduction = (avg_road_snow_depth * SPEED_REDUCTION_PER_INCH).min(MAX_SNOW_SPEED_REDUCTION);
    (1.0 - reduction).max(0.2)
}

/// Calculate the heating demand modifier from snow depth.
/// Returns a multiplier >= 1.0 where 1.0 = no snow effect.
/// Each 6 inches of snow adds 10% heating demand.
pub fn snow_heating_modifier(avg_snow_depth: f32) -> f32 {
    if avg_snow_depth <= 0.0 {
        return 1.0;
    }
    let increments = avg_snow_depth / 6.0;
    1.0 + increments * HEATING_INCREASE_PER_6_INCHES
}

/// Road plowing priority: highways first, then boulevards/avenues, then local roads.
/// Returns a priority value (lower = higher priority, plowed first).
pub(crate) fn plow_priority(road_type: RoadType) -> u8 {
    match road_type {
        RoadType::Highway => 0,
        RoadType::Boulevard => 1,
        RoadType::Avenue => 2,
        RoadType::OneWay => 3,
        RoadType::Local => 4,
        RoadType::Path => 5, // Paths are not plowed (pedestrian only)
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Main snow accumulation and melt system. Runs every slow tick.
///
/// 1. If snowing and below freezing: accumulate snow on all non-water cells.
/// 2. If above freezing: melt snow proportional to temperature excess.
/// 3. Track snowmelt runoff for spring flooding integration.
/// 4. Update aggregate statistics.
#[allow(clippy::too_many_arguments)]
pub fn update_snow(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    grid: Res<WorldGrid>,
    mut snow_grid: ResMut<SnowGrid>,
    mut stats: ResMut<SnowStats>,
    mut stormwater: ResMut<crate::stormwater::StormwaterGrid>,
) {
    if !timer.should_run() {
        return;
    }

    let accumulation = snow_accumulation_amount(&weather);
    let melt = snow_melt_amount(weather.temperature);
    let mut total_melt_runoff = 0.0_f32;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            // Don't accumulate snow on water
            if cell.cell_type == CellType::Water {
                snow_grid.set(x, y, 0.0);
                continue;
            }

            let current = snow_grid.get(x, y);

            // Accumulate snow
            let after_accumulation = if accumulation > 0.0 {
                (current + accumulation).min(MAX_SNOW_DEPTH)
            } else {
                current
            };

            // Melt snow
            let after_melt = if melt > 0.0 && after_accumulation > 0.0 {
                let melted = melt.min(after_accumulation);
                total_melt_runoff += melted;
                (after_accumulation - melted).max(0.0)
            } else {
                after_accumulation
            };

            snow_grid.set(x, y, after_melt);
        }
    }

    // Contribute snowmelt to stormwater runoff (for spring flooding risk)
    if total_melt_runoff > 0.0 {
        let melt_per_cell =
            total_melt_runoff * SNOWMELT_RUNOFF_FACTOR / (GRID_WIDTH * GRID_HEIGHT) as f32;
        stormwater.total_runoff += total_melt_runoff * SNOWMELT_RUNOFF_FACTOR;
        // Distribute melt runoff across all non-water cells
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if grid.get(x, y).cell_type != CellType::Water {
                    let idx = y * stormwater.width + x;
                    if idx < stormwater.runoff.len() {
                        stormwater.runoff[idx] += melt_per_cell;
                    }
                }
            }
        }
    }

    // Compute average road snow depth for speed calculations
    let mut road_snow_sum = 0.0_f32;
    let mut road_count = 0u32;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type == CellType::Road {
                road_snow_sum += snow_grid.get(x, y);
                road_count += 1;
            }
        }
    }
    let avg_road_snow = if road_count > 0 {
        road_snow_sum / road_count as f32
    } else {
        0.0
    };

    // Update stats
    stats.avg_depth = snow_grid.average_depth();
    stats.max_depth = snow_grid.max_depth();
    stats.covered_cells = snow_grid.covered_cells();
    stats.road_speed_multiplier = snow_speed_multiplier(avg_road_snow);
    stats.heating_demand_modifier = snow_heating_modifier(stats.avg_depth);
    stats.snowmelt_runoff = total_melt_runoff * SNOWMELT_RUNOFF_FACTOR;
}

/// Snow plowing system. Runs every slow tick.
///
/// When plowing is enabled and road snow depth exceeds the trigger threshold,
/// plows roads in priority order: highways > boulevards > avenues > local roads.
/// Each plowing pass removes PLOW_REMOVAL_DEPTH inches and costs PLOW_COST_PER_CELL.
pub fn update_snow_plowing(
    timer: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    mut snow_grid: ResMut<SnowGrid>,
    mut plowing: ResMut<SnowPlowingState>,
    mut budget: ResMut<CityBudget>,
) {
    if !timer.should_run() {
        return;
    }

    if !plowing.enabled {
        plowing.cells_plowed_last = 0;
        plowing.last_plow_cost = 0.0;
        return;
    }

    // Collect road cells that need plowing, sorted by priority
    let mut cells_to_plow: Vec<(usize, usize, u8)> = Vec::new();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Road {
                continue;
            }
            // Don't plow pedestrian paths
            if cell.road_type == RoadType::Path {
                continue;
            }
            let snow_depth = snow_grid.get(x, y);
            if snow_depth >= PLOW_TRIGGER_DEPTH {
                cells_to_plow.push((x, y, plow_priority(cell.road_type)));
            }
        }
    }

    if cells_to_plow.is_empty() {
        plowing.cells_plowed_last = 0;
        plowing.last_plow_cost = 0.0;
        return;
    }

    // Sort by priority (lower = higher priority)
    cells_to_plow.sort_by_key(|&(_, _, priority)| priority);

    let mut plowed_count = 0u32;
    let mut plow_cost = 0.0_f64;

    for (x, y, _) in &cells_to_plow {
        let current = snow_grid.get(*x, *y);
        let new_depth = (current - PLOW_REMOVAL_DEPTH).max(0.0);
        snow_grid.set(*x, *y, new_depth);
        plowed_count += 1;
        plow_cost += PLOW_COST_PER_CELL;
    }

    // Deduct cost from city treasury
    budget.treasury -= plow_cost;

    // Update plowing stats
    plowing.cells_plowed_last = plowed_count;
    plowing.last_plow_cost = plow_cost;
    plowing.cells_plowed_season += plowed_count;
    plowing.season_cost += plow_cost;
}

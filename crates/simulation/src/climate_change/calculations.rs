//! Pure helper functions for climate change calculations.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::utilities::UtilityType;

use super::constants::*;

/// Determine the temperature increase in Fahrenheit based on cumulative CO2 tons.
pub fn temperature_increase_from_co2(cumulative_tons: f64) -> f32 {
    if cumulative_tons >= THRESHOLD_3F {
        3.0
    } else if cumulative_tons >= THRESHOLD_2F {
        // Interpolate between 2F and 3F
        let progress = (cumulative_tons - THRESHOLD_2F) / (THRESHOLD_3F - THRESHOLD_2F);
        2.0 + progress as f32
    } else if cumulative_tons >= THRESHOLD_1F {
        // Interpolate between 1F and 2F
        let progress = (cumulative_tons - THRESHOLD_1F) / (THRESHOLD_2F - THRESHOLD_1F);
        1.0 + progress as f32
    } else if cumulative_tons > 0.0 {
        // Interpolate between 0F and 1F
        let progress = cumulative_tons / THRESHOLD_1F;
        progress as f32
    } else {
        0.0
    }
}

/// Calculate the disaster frequency multiplier from temperature increase.
/// +10% per 1F of warming.
pub fn disaster_multiplier_from_temp_increase(temp_increase_f: f32) -> f32 {
    1.0 + temp_increase_f * DISASTER_FREQUENCY_INCREASE_PER_F
}

/// Calculate the drought duration multiplier from temperature increase.
/// Each degree F adds 15% to drought duration.
pub fn drought_multiplier_from_temp_increase(temp_increase_f: f32) -> f32 {
    1.0 + temp_increase_f * 0.15
}

/// Calculate environmental score (0-100) based on cumulative emissions and yearly rate.
///
/// The score degrades as cumulative emissions rise, with the yearly rate affecting
/// the rate of degradation.
pub fn calculate_environmental_score(cumulative_co2: f64, yearly_co2: f64) -> f32 {
    // Base score starts at 100 and degrades with cumulative emissions.
    // Lose 10 points per million tons cumulative, capped at 0.
    let cumulative_penalty = (cumulative_co2 / 1_000_000.0 * 10.0) as f32;
    // Additional penalty for high yearly emissions (1 point per 10k tons/year).
    let yearly_penalty = (yearly_co2 / 10_000.0) as f32;
    (100.0 - cumulative_penalty - yearly_penalty).clamp(0.0, 100.0)
}

/// Get the CO2 emission rate for a utility type (tons per MWh).
pub fn co2_rate_for_utility(utility_type: UtilityType) -> f32 {
    match utility_type {
        UtilityType::PowerPlant => CO2_OIL, // Generic power plant uses oil/fossil rate
        UtilityType::SolarFarm => CO2_BIOMASS,
        UtilityType::WindTurbine => CO2_BIOMASS,
        UtilityType::NuclearPlant => CO2_BIOMASS, // Nuclear is zero-carbon
        UtilityType::Geothermal => CO2_BIOMASS,   // Geothermal is zero-carbon
        // Water utilities don't produce CO2 directly
        UtilityType::WaterTower => 0.0,
        UtilityType::SewagePlant => 0.0,
        UtilityType::PumpingStation => 0.0,
        UtilityType::WaterTreatment => 0.0,
        UtilityType::HydroDam => 0.0, // Hydro is zero-carbon
    }
}

/// Determine if a cell is adjacent to water.
pub(crate) fn is_water_adjacent(grid: &WorldGrid, x: usize, y: usize) -> bool {
    let (neighbors, count) = grid.neighbors4(x, y);
    for &(nx, ny) in &neighbors[..count] {
        if grid.get(nx, ny).cell_type == CellType::Water {
            return true;
        }
    }
    false
}

/// Find the elevation threshold for sea level rise flooding.
/// Returns the elevation below which water-adjacent cells should flood.
pub(crate) fn sea_level_rise_threshold(grid: &WorldGrid) -> f32 {
    // Collect elevations of all non-water, water-adjacent cells
    let mut coastal_elevations: Vec<f32> = Vec::new();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Water && is_water_adjacent(grid, x, y) {
                coastal_elevations.push(cell.elevation);
            }
        }
    }

    if coastal_elevations.is_empty() {
        return 0.0;
    }

    coastal_elevations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Find the elevation at the given percentile
    let index = ((coastal_elevations.len() as f32 * SEA_LEVEL_RISE_ELEVATION_PERCENTILE) as usize)
        .min(coastal_elevations.len().saturating_sub(1));
    coastal_elevations[index]
}

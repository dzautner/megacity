//! Pure helper functions for computing heat mitigation effects and costs.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::trees::TreeGrid;

use super::constants::*;

/// Compute the average tree coverage fraction across the entire grid.
/// Returns a value in [0.0, 1.0].
pub fn average_tree_coverage(tree_grid: &TreeGrid) -> f32 {
    let total = (GRID_WIDTH * GRID_HEIGHT) as f32;
    if total == 0.0 {
        return 0.0;
    }
    let count = tree_grid.cells.iter().filter(|&&has_tree| has_tree).count() as f32;
    count / total
}

/// Compute the green canopy temperature reduction based on average tree coverage.
/// -5F per 20% coverage.
pub fn green_canopy_reduction(tree_coverage_fraction: f32) -> f32 {
    // Each 0.20 fraction of tree coverage = 5F reduction
    let increments = tree_coverage_fraction / 0.20;
    increments * GREEN_CANOPY_TEMP_REDUCTION_PER_20PCT
}

/// Compute the light-colored roof temperature reduction as a city-wide average.
/// Returns the reduction in Fahrenheit scaled by the fraction of buildings upgraded.
pub fn light_roof_reduction(upgraded_count: u32, total_buildings: u32) -> f32 {
    if total_buildings == 0 {
        return 0.0;
    }
    let fraction = (upgraded_count as f32 / total_buildings as f32).min(1.0);
    fraction * LIGHT_ROOF_TEMP_REDUCTION
}

/// Compute the misting station temperature reduction.
/// Scales with the number of stations, capped at the maximum reduction.
pub fn misting_reduction(station_count: u32) -> f32 {
    if station_count == 0 {
        return 0.0;
    }
    // Each station covers a portion of the city; diminishing returns after many.
    // Model: full effect at 50+ stations, linear ramp up.
    let fraction = (station_count as f32 / 50.0).min(1.0);
    fraction * MISTING_STATION_TEMP_REDUCTION
}

/// Compute the total mortality reduction factor from active mitigations.
/// Returns a value in [0.0, 1.0] where 1.0 means all heat mortality prevented.
pub fn total_mortality_reduction(cooling_centers_active: bool, dehydration_prevented: bool) -> f32 {
    let mut reduction = 0.0_f32;
    if cooling_centers_active {
        reduction += COOLING_CENTER_MORTALITY_REDUCTION;
    }
    // Emergency water prevents dehydration component (~30% of heat deaths)
    if dehydration_prevented {
        reduction += 0.30;
    }
    reduction.min(1.0)
}

/// Compute the daily operating cost of all active mitigations during a heat wave.
pub fn daily_operating_cost(
    cooling_centers_active: bool,
    emergency_water_active: bool,
    misting_station_count: u32,
) -> f64 {
    let mut cost = 0.0_f64;
    if cooling_centers_active {
        cost += COOLING_CENTER_DAILY_COST;
    }
    if emergency_water_active {
        cost += EMERGENCY_WATER_DAILY_COST;
    }
    cost += misting_station_count as f64 * MISTING_STATION_DAILY_COST;
    cost
}

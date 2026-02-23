//! Pure helper functions for inclusionary zoning calculations.
//!
//! These functions are free of ECS dependencies and can be tested in isolation.

use crate::districts::DistrictMap;

use super::config::*;

/// Calculate the FAR bonus for a given affordable percentage.
/// The bonus scales linearly from MIN_FAR_BONUS at MIN_AFFORDABLE_PERCENTAGE
/// to MAX_FAR_BONUS at MAX_AFFORDABLE_PERCENTAGE.
pub fn calculate_far_bonus(affordable_pct: f32) -> f32 {
    if affordable_pct <= 0.0 {
        return 0.0;
    }
    let clamped = affordable_pct.clamp(MIN_AFFORDABLE_PERCENTAGE, MAX_AFFORDABLE_PERCENTAGE);
    let t = (clamped - MIN_AFFORDABLE_PERCENTAGE)
        / (MAX_AFFORDABLE_PERCENTAGE - MIN_AFFORDABLE_PERCENTAGE);
    MIN_FAR_BONUS + t * (MAX_FAR_BONUS - MIN_FAR_BONUS)
}

/// Calculate the number of affordable units for a building given its capacity
/// and the district's affordable percentage.
pub fn calculate_affordable_units(capacity: u32, affordable_pct: f32) -> u32 {
    if affordable_pct <= 0.0 {
        return 0;
    }
    // Round to nearest; guarantee at least 1 unit if building has capacity and policy is active
    let raw = (capacity as f32 * affordable_pct).round() as u32;
    let raw = if capacity > 0 { raw.max(1) } else { raw };
    raw.min(capacity)
}

/// Calculate the effective (market-rate) capacity after removing affordable units.
pub fn calculate_effective_capacity(capacity: u32, affordable_pct: f32) -> u32 {
    let affordable = calculate_affordable_units(capacity, affordable_pct);
    capacity.saturating_sub(affordable)
}

/// Calculate the monthly admin cost for the given number of enabled districts.
pub fn calculate_monthly_admin_cost(enabled_count: usize) -> f64 {
    enabled_count as f64 * MONTHLY_ADMIN_COST_PER_DISTRICT
}

/// Check if a cell is in a district with inclusionary zoning enabled.
pub fn is_cell_in_inclusionary_district(
    x: usize,
    y: usize,
    state: &InclusionaryZoningState,
    district_map: &DistrictMap,
) -> bool {
    district_map
        .get_district_index_at(x, y)
        .is_some_and(|di| state.is_enabled(di))
}

/// Get the affordable percentage for the cell's district, or 0.0 if not in
/// an inclusionary zoning district.
pub fn affordable_percentage_for_cell(
    x: usize,
    y: usize,
    state: &InclusionaryZoningState,
    district_map: &DistrictMap,
) -> f32 {
    district_map
        .get_district_index_at(x, y)
        .map(|di| state.affordable_percentage(di))
        .unwrap_or(0.0)
}

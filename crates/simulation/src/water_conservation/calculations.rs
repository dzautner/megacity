use super::constants::*;
use super::types::WaterConservationState;

/// Compute the effective rainwater harvesting reduction given the current
/// precipitation intensity. Full effectiveness at >= 1.0 in/hr, linearly
/// scaled below that.
pub(crate) fn rainwater_effectiveness(precipitation_intensity: f32) -> f32 {
    if precipitation_intensity <= 0.0 {
        0.0
    } else {
        // Linear scale: full effect at 1.0 in/hr, proportional below.
        (precipitation_intensity / 1.0).min(1.0) * RAINWATER_DEMAND_REDUCTION
    }
}

/// Calculate total demand reduction percentage from the set of active policies
/// and current precipitation. Result is capped at `MAX_TOTAL_DEMAND_REDUCTION`.
pub(crate) fn calculate_demand_reduction(
    state: &WaterConservationState,
    precipitation: f32,
) -> f32 {
    let mut reduction = 0.0_f32;

    if state.low_flow_fixtures {
        reduction += LOW_FLOW_DEMAND_REDUCTION;
    }
    if state.xeriscaping {
        reduction += XERISCAPING_DEMAND_REDUCTION;
    }
    if state.tiered_pricing {
        reduction += TIERED_PRICING_DEMAND_REDUCTION;
    }
    if state.greywater_recycling {
        reduction += GREYWATER_DEMAND_REDUCTION;
    }
    if state.rainwater_harvesting {
        reduction += rainwater_effectiveness(precipitation);
    }

    reduction.min(MAX_TOTAL_DEMAND_REDUCTION)
}

/// Calculate total retrofit cost for all policies that require per-building investment.
pub(crate) fn calculate_retrofit_cost(state: &WaterConservationState, building_count: u32) -> f64 {
    let count = building_count as f64;
    let mut cost = 0.0_f64;

    if state.low_flow_fixtures {
        cost += LOW_FLOW_COST_PER_BUILDING * count;
    }
    if state.greywater_recycling {
        cost += GREYWATER_COST_PER_BUILDING * count;
    }
    if state.rainwater_harvesting {
        cost += RAINWATER_COST_PER_BUILDING * count;
    }

    cost
}

/// Estimate annual water savings in gallons based on demand reduction and building count.
pub(crate) fn calculate_annual_savings(demand_reduction_pct: f32, building_count: u32) -> f64 {
    let daily_savings =
        BASE_DAILY_DEMAND_PER_BUILDING * building_count as f64 * demand_reduction_pct as f64;
    daily_savings * DAYS_PER_YEAR
}

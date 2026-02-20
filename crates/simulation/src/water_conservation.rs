use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::weather::Weather;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Demand reduction from low-flow fixtures (applied to residential buildings only).
pub const LOW_FLOW_DEMAND_REDUCTION: f32 = 0.20;

/// Demand reduction from xeriscaping (reduced irrigation area, applies to total demand).
pub const XERISCAPING_DEMAND_REDUCTION: f32 = 0.10;

/// Demand reduction from tiered water pricing (behavioural change, applies to total demand).
pub const TIERED_PRICING_DEMAND_REDUCTION: f32 = 0.15;

/// Demand reduction from greywater recycling (applies to total demand).
pub const GREYWATER_DEMAND_REDUCTION: f32 = 0.15;

/// Sewage volume reduction from greywater recycling (greywater reused instead of discharged).
pub const GREYWATER_SEWAGE_REDUCTION: f32 = 0.30;

/// Demand reduction from rainwater harvesting (effective only when precipitation > 0).
pub const RAINWATER_DEMAND_REDUCTION: f32 = 0.10;

/// Hard cap on total demand reduction from all combined conservation policies.
pub const MAX_TOTAL_DEMAND_REDUCTION: f32 = 0.60;

/// Per-building retrofit cost for low-flow fixtures (dollars).
pub const LOW_FLOW_COST_PER_BUILDING: f64 = 500.0;

/// Per-building retrofit cost for greywater recycling (dollars).
pub const GREYWATER_COST_PER_BUILDING: f64 = 3000.0;

/// Per-building retrofit cost for rainwater harvesting (dollars).
pub const RAINWATER_COST_PER_BUILDING: f64 = 1000.0;

/// Base daily water demand per building used for annual savings estimates (gallons).
/// This is a rough average across building types for estimating conservation savings.
const BASE_DAILY_DEMAND_PER_BUILDING: f64 = 1200.0;

/// Days in a year for annual savings calculation.
const DAYS_PER_YEAR: f64 = 365.0;

// =============================================================================
// Resource
// =============================================================================

/// City-wide water conservation policy state.
///
/// Each boolean field represents an individually toggleable conservation policy.
/// Other simulation systems (e.g. `water_demand`) read `demand_reduction_pct` to
/// apply the aggregate reduction. This resource does NOT modify the `Policy` enum;
/// it is a standalone system with its own policy toggles.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct WaterConservationState {
    /// Low-flow fixtures installed in residential buildings (-20% residential demand).
    pub low_flow_fixtures: bool,
    /// Xeriscaping programme active (-10% total demand from reduced irrigation).
    pub xeriscaping: bool,
    /// Tiered water pricing in effect (-15% total demand from behavioural change).
    pub tiered_pricing: bool,
    /// Greywater recycling enabled (-15% demand AND -30% sewage volume).
    pub greywater_recycling: bool,
    /// Rainwater harvesting systems installed (-10% demand when raining).
    pub rainwater_harvesting: bool,
    /// Aggregate demand reduction percentage from all active policies (0.0 .. 0.60).
    pub demand_reduction_pct: f32,
    /// Sewage volume reduction percentage from greywater recycling (0.0 or 0.30).
    pub sewage_reduction_pct: f32,
    /// Cumulative dollar cost of all building retrofits.
    pub total_retrofit_cost: f64,
    /// Estimated annual water savings in gallons based on current policies and building count.
    pub annual_savings_gallons: f64,
    /// Number of buildings that have been (or would be) retrofitted.
    pub buildings_retrofitted: u32,
}

impl Default for WaterConservationState {
    fn default() -> Self {
        Self {
            low_flow_fixtures: false,
            xeriscaping: false,
            tiered_pricing: false,
            greywater_recycling: false,
            rainwater_harvesting: false,
            demand_reduction_pct: 0.0,
            sewage_reduction_pct: 0.0,
            total_retrofit_cost: 0.0,
            annual_savings_gallons: 0.0,
            buildings_retrofitted: 0,
        }
    }
}

// =============================================================================
// Helper functions (pure, testable)
// =============================================================================

/// Compute the effective rainwater harvesting reduction given the current
/// precipitation intensity. Full effectiveness at >= 1.0 in/hr, linearly
/// scaled below that.
fn rainwater_effectiveness(precipitation_intensity: f32) -> f32 {
    if precipitation_intensity <= 0.0 {
        0.0
    } else {
        // Linear scale: full effect at 1.0 in/hr, proportional below.
        (precipitation_intensity / 1.0).min(1.0) * RAINWATER_DEMAND_REDUCTION
    }
}

/// Calculate total demand reduction percentage from the set of active policies
/// and current precipitation. Result is capped at `MAX_TOTAL_DEMAND_REDUCTION`.
fn calculate_demand_reduction(state: &WaterConservationState, precipitation: f32) -> f32 {
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
fn calculate_retrofit_cost(state: &WaterConservationState, building_count: u32) -> f64 {
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
fn calculate_annual_savings(demand_reduction_pct: f32, building_count: u32) -> f64 {
    let daily_savings =
        BASE_DAILY_DEMAND_PER_BUILDING * building_count as f64 * demand_reduction_pct as f64;
    daily_savings * DAYS_PER_YEAR
}

// =============================================================================
// System
// =============================================================================

/// System: Recalculate water conservation metrics every slow tick.
///
/// 1. Counts buildings to determine retrofit scope.
/// 2. Computes aggregate `demand_reduction_pct` (capped at 0.60).
/// 3. Computes `sewage_reduction_pct` from greywater policy.
/// 4. Computes `total_retrofit_cost` from per-building policy costs.
/// 5. Adjusts rainwater harvesting effectiveness by current precipitation.
/// 6. Updates estimated `annual_savings_gallons`.
pub fn update_water_conservation(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    mut conservation: ResMut<WaterConservationState>,
    buildings: Query<&Building>,
) {
    if !timer.should_run() {
        return;
    }

    let building_count = buildings.iter().count() as u32;

    // 1. Demand reduction (precipitation-aware for rainwater harvesting)
    let precipitation = weather.precipitation_intensity;
    conservation.demand_reduction_pct = calculate_demand_reduction(&conservation, precipitation);

    // 2. Sewage reduction
    conservation.sewage_reduction_pct = if conservation.greywater_recycling {
        GREYWATER_SEWAGE_REDUCTION
    } else {
        0.0
    };

    // 3. Retrofit costs
    conservation.buildings_retrofitted = building_count;
    conservation.total_retrofit_cost = calculate_retrofit_cost(&conservation, building_count);

    // 4. Annual savings estimate
    conservation.annual_savings_gallons =
        calculate_annual_savings(conservation.demand_reduction_pct, building_count);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a default conservation state with all policies off.
    fn default_state() -> WaterConservationState {
        WaterConservationState::default()
    }

    // -------------------------------------------------------------------------
    // Default / initial state tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_all_policies_off() {
        let state = default_state();
        assert!(!state.low_flow_fixtures);
        assert!(!state.xeriscaping);
        assert!(!state.tiered_pricing);
        assert!(!state.greywater_recycling);
        assert!(!state.rainwater_harvesting);
    }

    #[test]
    fn test_default_zero_reductions() {
        let state = default_state();
        assert_eq!(state.demand_reduction_pct, 0.0);
        assert_eq!(state.sewage_reduction_pct, 0.0);
    }

    #[test]
    fn test_default_zero_costs_and_savings() {
        let state = default_state();
        assert_eq!(state.total_retrofit_cost, 0.0);
        assert_eq!(state.annual_savings_gallons, 0.0);
        assert_eq!(state.buildings_retrofitted, 0);
    }

    // -------------------------------------------------------------------------
    // Individual policy demand reduction tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_low_flow_fixtures_reduction() {
        let mut state = default_state();
        state.low_flow_fixtures = true;
        let reduction = calculate_demand_reduction(&state, 0.0);
        assert!((reduction - LOW_FLOW_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_xeriscaping_reduction() {
        let mut state = default_state();
        state.xeriscaping = true;
        let reduction = calculate_demand_reduction(&state, 0.0);
        assert!((reduction - XERISCAPING_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tiered_pricing_reduction() {
        let mut state = default_state();
        state.tiered_pricing = true;
        let reduction = calculate_demand_reduction(&state, 0.0);
        assert!((reduction - TIERED_PRICING_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_greywater_demand_reduction() {
        let mut state = default_state();
        state.greywater_recycling = true;
        let reduction = calculate_demand_reduction(&state, 0.0);
        assert!((reduction - GREYWATER_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainwater_full_precipitation() {
        let mut state = default_state();
        state.rainwater_harvesting = true;
        // Full precipitation (>= 1.0 in/hr) gives full reduction.
        let reduction = calculate_demand_reduction(&state, 1.5);
        assert!((reduction - RAINWATER_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainwater_no_precipitation() {
        let mut state = default_state();
        state.rainwater_harvesting = true;
        // No rain means zero rainwater reduction.
        let reduction = calculate_demand_reduction(&state, 0.0);
        assert_eq!(reduction, 0.0);
    }

    #[test]
    fn test_rainwater_partial_precipitation() {
        let mut state = default_state();
        state.rainwater_harvesting = true;
        // 0.5 in/hr should give 50% of the full RAINWATER_DEMAND_REDUCTION.
        let reduction = calculate_demand_reduction(&state, 0.5);
        let expected = RAINWATER_DEMAND_REDUCTION * 0.5;
        assert!((reduction - expected).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Combined policy tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_two_policies_additive() {
        let mut state = default_state();
        state.low_flow_fixtures = true;
        state.xeriscaping = true;
        let reduction = calculate_demand_reduction(&state, 0.0);
        let expected = LOW_FLOW_DEMAND_REDUCTION + XERISCAPING_DEMAND_REDUCTION;
        assert!((reduction - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_three_policies_additive() {
        let mut state = default_state();
        state.low_flow_fixtures = true;
        state.xeriscaping = true;
        state.tiered_pricing = true;
        let reduction = calculate_demand_reduction(&state, 0.0);
        let expected = LOW_FLOW_DEMAND_REDUCTION
            + XERISCAPING_DEMAND_REDUCTION
            + TIERED_PRICING_DEMAND_REDUCTION;
        assert!((reduction - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_all_policies_no_rain_capped() {
        let mut state = default_state();
        state.low_flow_fixtures = true;
        state.xeriscaping = true;
        state.tiered_pricing = true;
        state.greywater_recycling = true;
        state.rainwater_harvesting = true;
        // Without rain, sum is 0.20 + 0.10 + 0.15 + 0.15 + 0.00 = 0.60, exactly at cap.
        let reduction = calculate_demand_reduction(&state, 0.0);
        assert!((reduction - MAX_TOTAL_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_all_policies_with_rain_capped() {
        let mut state = default_state();
        state.low_flow_fixtures = true;
        state.xeriscaping = true;
        state.tiered_pricing = true;
        state.greywater_recycling = true;
        state.rainwater_harvesting = true;
        // With rain, sum would be 0.20 + 0.10 + 0.15 + 0.15 + 0.10 = 0.70, must be capped.
        let reduction = calculate_demand_reduction(&state, 2.0);
        assert!((reduction - MAX_TOTAL_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cap_enforced_even_with_excess() {
        let mut state = default_state();
        state.low_flow_fixtures = true;
        state.tiered_pricing = true;
        state.greywater_recycling = true;
        state.rainwater_harvesting = true;
        // 0.20 + 0.15 + 0.15 + 0.10 = 0.60, exactly at cap.
        let reduction = calculate_demand_reduction(&state, 1.0);
        assert!(reduction <= MAX_TOTAL_DEMAND_REDUCTION);
        assert!((reduction - MAX_TOTAL_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Sewage reduction tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sewage_reduction_with_greywater() {
        let mut state = default_state();
        state.greywater_recycling = true;
        // Directly test the sewage logic.
        let sewage = if state.greywater_recycling {
            GREYWATER_SEWAGE_REDUCTION
        } else {
            0.0
        };
        assert!((sewage - GREYWATER_SEWAGE_REDUCTION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sewage_reduction_without_greywater() {
        let state = default_state();
        let sewage = if state.greywater_recycling {
            GREYWATER_SEWAGE_REDUCTION
        } else {
            0.0
        };
        assert_eq!(sewage, 0.0);
    }

    // -------------------------------------------------------------------------
    // Retrofit cost tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_retrofit_cost_no_policies() {
        let state = default_state();
        let cost = calculate_retrofit_cost(&state, 100);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_retrofit_cost_low_flow_only() {
        let mut state = default_state();
        state.low_flow_fixtures = true;
        let cost = calculate_retrofit_cost(&state, 50);
        assert!((cost - LOW_FLOW_COST_PER_BUILDING * 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_retrofit_cost_greywater_only() {
        let mut state = default_state();
        state.greywater_recycling = true;
        let cost = calculate_retrofit_cost(&state, 10);
        assert!((cost - GREYWATER_COST_PER_BUILDING * 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_retrofit_cost_rainwater_only() {
        let mut state = default_state();
        state.rainwater_harvesting = true;
        let cost = calculate_retrofit_cost(&state, 20);
        assert!((cost - RAINWATER_COST_PER_BUILDING * 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_retrofit_cost_all_building_policies() {
        let mut state = default_state();
        state.low_flow_fixtures = true;
        state.greywater_recycling = true;
        state.rainwater_harvesting = true;
        let n = 100;
        let expected = (LOW_FLOW_COST_PER_BUILDING
            + GREYWATER_COST_PER_BUILDING
            + RAINWATER_COST_PER_BUILDING)
            * n as f64;
        let cost = calculate_retrofit_cost(&state, n);
        assert!((cost - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_retrofit_cost_zero_buildings() {
        let mut state = default_state();
        state.low_flow_fixtures = true;
        state.greywater_recycling = true;
        state.rainwater_harvesting = true;
        let cost = calculate_retrofit_cost(&state, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_xeriscaping_and_tiered_no_retrofit_cost() {
        // Xeriscaping and tiered pricing are policy-level changes, not per-building retrofits.
        let mut state = default_state();
        state.xeriscaping = true;
        state.tiered_pricing = true;
        let cost = calculate_retrofit_cost(&state, 200);
        assert_eq!(cost, 0.0);
    }

    // -------------------------------------------------------------------------
    // Annual savings tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_annual_savings_zero_reduction() {
        let savings = calculate_annual_savings(0.0, 100);
        assert_eq!(savings, 0.0);
    }

    #[test]
    fn test_annual_savings_zero_buildings() {
        let savings = calculate_annual_savings(0.20, 0);
        assert_eq!(savings, 0.0);
    }

    #[test]
    fn test_annual_savings_calculation() {
        let reduction = 0.20_f32;
        let buildings = 100_u32;
        let savings = calculate_annual_savings(reduction, buildings);
        let expected =
            BASE_DAILY_DEMAND_PER_BUILDING * buildings as f64 * reduction as f64 * DAYS_PER_YEAR;
        assert!((savings - expected).abs() < 1.0); // within 1 gallon tolerance
    }

    #[test]
    fn test_annual_savings_scales_with_buildings() {
        let reduction = 0.15;
        let savings_10 = calculate_annual_savings(reduction, 10);
        let savings_20 = calculate_annual_savings(reduction, 20);
        assert!((savings_20 - savings_10 * 2.0).abs() < 1.0);
    }

    #[test]
    fn test_annual_savings_scales_with_reduction() {
        let buildings = 50;
        let savings_10 = calculate_annual_savings(0.10, buildings);
        let savings_20 = calculate_annual_savings(0.20, buildings);
        assert!((savings_20 - savings_10 * 2.0).abs() < 1.0);
    }

    // -------------------------------------------------------------------------
    // Rainwater effectiveness helper tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_rainwater_effectiveness_zero() {
        assert_eq!(rainwater_effectiveness(0.0), 0.0);
    }

    #[test]
    fn test_rainwater_effectiveness_negative() {
        // Negative precipitation treated as none.
        assert_eq!(rainwater_effectiveness(-0.5), 0.0);
    }

    #[test]
    fn test_rainwater_effectiveness_half() {
        let eff = rainwater_effectiveness(0.5);
        let expected = 0.5 * RAINWATER_DEMAND_REDUCTION;
        assert!((eff - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainwater_effectiveness_full() {
        let eff = rainwater_effectiveness(1.0);
        assert!((eff - RAINWATER_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rainwater_effectiveness_excess_capped() {
        // Precipitation above 1.0 in/hr should not exceed full reduction.
        let eff = rainwater_effectiveness(3.0);
        assert!((eff - RAINWATER_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests (combined behaviour)
    // -------------------------------------------------------------------------

    #[test]
    fn test_no_policies_no_reduction() {
        let state = default_state();
        let reduction = calculate_demand_reduction(&state, 1.0);
        assert_eq!(reduction, 0.0);
    }

    #[test]
    fn test_greywater_provides_both_demand_and_sewage_reduction() {
        let mut state = default_state();
        state.greywater_recycling = true;

        let demand = calculate_demand_reduction(&state, 0.0);
        assert!((demand - GREYWATER_DEMAND_REDUCTION).abs() < f32::EPSILON);

        let sewage = if state.greywater_recycling {
            GREYWATER_SEWAGE_REDUCTION
        } else {
            0.0
        };
        assert!((sewage - GREYWATER_SEWAGE_REDUCTION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combined_cost_and_savings_coherent() {
        let mut state = default_state();
        state.low_flow_fixtures = true;
        state.greywater_recycling = true;

        let buildings = 75_u32;
        let reduction = calculate_demand_reduction(&state, 0.0);
        let cost = calculate_retrofit_cost(&state, buildings);
        let savings = calculate_annual_savings(reduction, buildings);

        // Costs should be positive when building policies are active.
        assert!(cost > 0.0);
        // Savings should be positive when there is a reduction.
        assert!(savings > 0.0);
        // Reduction should be the sum of two policies.
        let expected_reduction = LOW_FLOW_DEMAND_REDUCTION + GREYWATER_DEMAND_REDUCTION;
        assert!((reduction - expected_reduction).abs() < f32::EPSILON);
    }

    #[test]
    fn test_constant_values_are_correct() {
        // Verify the documented constant values match the requirements.
        assert_eq!(LOW_FLOW_DEMAND_REDUCTION, 0.20);
        assert_eq!(XERISCAPING_DEMAND_REDUCTION, 0.10);
        assert_eq!(TIERED_PRICING_DEMAND_REDUCTION, 0.15);
        assert_eq!(GREYWATER_DEMAND_REDUCTION, 0.15);
        assert_eq!(GREYWATER_SEWAGE_REDUCTION, 0.30);
        assert_eq!(RAINWATER_DEMAND_REDUCTION, 0.10);
        assert_eq!(MAX_TOTAL_DEMAND_REDUCTION, 0.60);
        assert_eq!(LOW_FLOW_COST_PER_BUILDING, 500.0);
        assert_eq!(GREYWATER_COST_PER_BUILDING, 3000.0);
        assert_eq!(RAINWATER_COST_PER_BUILDING, 1000.0);
    }

    #[test]
    fn test_max_reduction_sum_without_rain_equals_cap() {
        // Verify that the non-rain policies alone can reach the cap.
        let sum = LOW_FLOW_DEMAND_REDUCTION
            + XERISCAPING_DEMAND_REDUCTION
            + TIERED_PRICING_DEMAND_REDUCTION
            + GREYWATER_DEMAND_REDUCTION;
        assert!((sum - MAX_TOTAL_DEMAND_REDUCTION).abs() < f32::EPSILON);
    }
}

pub struct WaterConservationPlugin;

impl Plugin for WaterConservationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterConservationState>().add_systems(
            FixedUpdate,
            update_water_conservation.after(crate::imports_exports::process_trade),
        );
    }
}

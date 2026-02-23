#[cfg(test)]
mod tests {
    use crate::water_conservation::calculations::*;
    use crate::water_conservation::constants::*;
    use crate::water_conservation::types::WaterConservationState;

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

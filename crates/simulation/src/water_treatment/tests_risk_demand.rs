//! Unit tests for disease risk, demand estimation, and integration-style scenarios.

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::water_treatment::{
        calculate_disease_risk, estimate_demand_mgd, PlantState, TreatmentLevel,
        WaterTreatmentState,
    };

    // -------------------------------------------------------------------------
    // Disease risk tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_disease_risk_pure_water() {
        let risk = calculate_disease_risk(1.0);
        assert!(
            risk.abs() < f32::EPSILON,
            "Pure water should have zero disease risk, got {}",
            risk
        );
    }

    #[test]
    fn test_disease_risk_high_quality() {
        // Quality 0.95+ = zero risk
        let risk = calculate_disease_risk(0.95);
        assert!(
            risk.abs() < f32::EPSILON,
            "Quality 0.95 should have zero risk, got {}",
            risk
        );
    }

    #[test]
    fn test_disease_risk_moderate_quality() {
        // Quality 0.85: deficit=0.15, risk=0.15^2=0.0225
        let risk = calculate_disease_risk(0.85);
        assert!(
            (risk - 0.0225).abs() < 0.01,
            "Quality 0.85 should have ~0.0225 risk, got {}",
            risk
        );
    }

    #[test]
    fn test_disease_risk_low_quality() {
        // Quality 0.5 = moderate risk
        let risk = calculate_disease_risk(0.5);
        let expected = 0.25; // (1.0 - 0.5)^2 = 0.25
        assert!(
            (risk - expected).abs() < 0.01,
            "Quality 0.5 should have ~0.25 risk, got {}",
            risk
        );
    }

    #[test]
    fn test_disease_risk_fully_contaminated() {
        // Quality 0.0 = max risk (1.0)
        let risk = calculate_disease_risk(0.0);
        assert!(
            (risk - 1.0).abs() < 0.01,
            "Fully contaminated should have risk ~1.0, got {}",
            risk
        );
    }

    #[test]
    fn test_disease_risk_monotonically_decreases_with_quality() {
        let mut prev_risk = calculate_disease_risk(0.0);
        for q in 1..=20 {
            let quality = q as f32 * 0.05;
            let risk = calculate_disease_risk(quality);
            assert!(
                risk <= prev_risk + f32::EPSILON,
                "Risk should decrease with quality: q={}, risk={}, prev={}",
                quality,
                risk,
                prev_risk
            );
            prev_risk = risk;
        }
    }

    // -------------------------------------------------------------------------
    // Demand estimation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_estimate_demand_mgd_zero_population() {
        let demand = estimate_demand_mgd(0);
        assert!(
            demand.abs() < f32::EPSILON,
            "Zero population should have zero demand"
        );
    }

    #[test]
    fn test_estimate_demand_mgd_small_city() {
        // 10,000 people * 150 GPCD = 1,500,000 GPD = 1.5 MGD
        let demand = estimate_demand_mgd(10_000);
        assert!(
            (demand - 1.5).abs() < 0.001,
            "10K population should need 1.5 MGD, got {}",
            demand
        );
    }

    #[test]
    fn test_estimate_demand_mgd_large_city() {
        // 1,000,000 people * 150 GPCD = 150,000,000 GPD = 150 MGD
        let demand = estimate_demand_mgd(1_000_000);
        assert!(
            (demand - 150.0).abs() < 0.1,
            "1M population should need 150 MGD, got {}",
            demand
        );
    }

    // -------------------------------------------------------------------------
    // Integration-style tests (simulating update logic)
    // -------------------------------------------------------------------------

    #[test]
    fn test_treatment_cost_calculation() {
        // A Primary plant processing 5 MGD: 5 * $1,000 = $5,000
        let flow = 5.0_f32;
        let cost = TreatmentLevel::Primary.cost_per_million_gallons() * flow as f64;
        assert!(
            (cost - 5_000.0).abs() < 0.01,
            "Expected $5,000, got ${}",
            cost
        );
    }

    #[test]
    fn test_treatment_cost_advanced_plant() {
        // An Advanced plant processing 2 MGD: 2 * $10,000 = $20,000
        let flow = 2.0_f32;
        let cost = TreatmentLevel::Advanced.cost_per_million_gallons() * flow as f64;
        assert!(
            (cost - 20_000.0).abs() < 0.01,
            "Expected $20,000, got ${}",
            cost
        );
    }

    #[test]
    fn test_capacity_limits_flow() {
        // Plant capacity 10 MGD, city demand 15 MGD => only 10 MGD processed
        let capacity = TreatmentLevel::Primary.base_capacity_mgd();
        let demand = 15.0_f32;
        let flow = demand.min(capacity);
        assert!(
            (flow - 10.0).abs() < f32::EPSILON,
            "Flow should be capped at capacity, got {}",
            flow
        );
    }

    #[test]
    fn test_multiple_plants_aggregate_capacity() {
        let mut state = WaterTreatmentState::default();
        state.register_plant(Entity::from_raw(1), TreatmentLevel::Primary); // 10 MGD
        state.register_plant(Entity::from_raw(2), TreatmentLevel::Secondary); // 8 MGD
        state.register_plant(Entity::from_raw(3), TreatmentLevel::Tertiary); // 5 MGD

        let total: f32 = state.plants.values().map(|p| p.capacity_mgd).sum();
        assert!(
            (total - 23.0).abs() < f32::EPSILON,
            "Total capacity should be 23 MGD, got {}",
            total
        );
    }

    #[test]
    fn test_flow_distribution_under_capacity() {
        // Two plants: Primary (10 MGD) + Secondary (8 MGD) = 18 MGD capacity
        // City demand: 12 MGD
        // First plant gets 10 MGD (full), second gets 2 MGD
        let plants = vec![
            PlantState::new(TreatmentLevel::Primary),
            PlantState::new(TreatmentLevel::Secondary),
        ];

        let mut remaining = 12.0_f32;
        let mut flows = Vec::new();

        for plant in &plants {
            let flow = remaining.min(plant.capacity_mgd);
            flows.push(flow);
            remaining -= flow;
        }

        assert!((flows[0] - 10.0).abs() < f32::EPSILON);
        assert!((flows[1] - 2.0).abs() < f32::EPSILON);
        assert!(remaining.abs() < f32::EPSILON);
    }

    #[test]
    fn test_flow_distribution_over_capacity() {
        // Single Primary plant (10 MGD), demand 15 MGD => 5 MGD untreated
        let plant = PlantState::new(TreatmentLevel::Primary);
        let demand = 15.0_f32;
        let flow = demand.min(plant.capacity_mgd);
        let untreated = demand - flow;

        assert!((flow - 10.0).abs() < f32::EPSILON);
        assert!((untreated - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_blended_quality_partial_treatment() {
        // 60% of water treated to quality 0.9, 40% untreated at quality 0.3
        let treated_fraction = 0.6_f32;
        let untreated_fraction = 0.4_f32;
        let treated_quality = 0.9_f32;
        let input_quality = 0.3_f32;

        let blended = treated_quality * treated_fraction + input_quality * untreated_fraction;
        // 0.9 * 0.6 + 0.3 * 0.4 = 0.54 + 0.12 = 0.66
        assert!(
            (blended - 0.66).abs() < 0.001,
            "Blended quality should be 0.66, got {}",
            blended
        );
    }

    #[test]
    fn test_no_plants_no_treatment() {
        let state = WaterTreatmentState::default();
        assert!(state.plants.is_empty());
        assert_eq!(state.total_capacity_mgd, 0.0);
        assert_eq!(state.total_flow_mgd, 0.0);
        assert_eq!(state.avg_effluent_quality, 0.0);
    }
}

//! Tests for WTE diversion calculations and combined policy effects.

use super::*;

// -------------------------------------------------------------------------
// WTE diversion tests
// -------------------------------------------------------------------------

#[test]
fn test_wte_no_mandate_no_diversion() {
    let diversion = calculate_wte_diversion(false, 100.0, 2);
    assert!(diversion.abs() < f64::EPSILON);
}

#[test]
fn test_wte_mandate_no_incinerators_no_diversion() {
    let diversion = calculate_wte_diversion(true, 100.0, 0);
    assert!(diversion.abs() < f64::EPSILON);
}

#[test]
fn test_wte_mandate_with_incinerator() {
    // 100 tons generated, 80% divertable = 80 tons
    // 1 incinerator at 250 tons capacity
    // Diversion = min(80, 250) = 80
    let diversion = calculate_wte_diversion(true, 100.0, 1);
    let expected = 100.0 * WTE_MANDATE_DIVERSION_FRACTION as f64;
    assert!((diversion - expected).abs() < 0.01);
}

#[test]
fn test_wte_diversion_capped_by_capacity() {
    // 1000 tons generated, 80% divertable = 800 tons
    // 1 incinerator at 250 tons capacity
    // Diversion = min(800, 250) = 250
    let diversion = calculate_wte_diversion(true, 1000.0, 1);
    assert!((diversion - 250.0).abs() < 0.01);
}

#[test]
fn test_wte_diversion_multiple_incinerators() {
    // 1000 tons generated, 80% divertable = 800 tons
    // 2 incinerators at 250 each = 500 tons capacity
    // Diversion = min(800, 500) = 500
    let diversion = calculate_wte_diversion(true, 1000.0, 2);
    assert!((diversion - 500.0).abs() < 0.01);
}

#[test]
fn test_wte_diversion_zero_waste() {
    let diversion = calculate_wte_diversion(true, 0.0, 3);
    assert!(diversion.abs() < f64::EPSILON);
}

// -------------------------------------------------------------------------
// Integration-style tests
// -------------------------------------------------------------------------

#[test]
fn test_combined_effects_plastic_ban_and_composting() {
    let mut state = WastePolicyState::default();
    state.plastic_bag_ban = true;
    state.composting_mandate = true;

    let multiplier = calculate_waste_multiplier(&state);
    assert!((multiplier - 0.95).abs() < f32::EPSILON);

    let penalty = calculate_happiness_penalty(&state);
    let expected_penalty = PLASTIC_BAG_BAN_HAPPINESS_PENALTY + COMPOSTING_MANDATE_HAPPINESS_PENALTY;
    assert!((penalty - expected_penalty).abs() < f32::EPSILON);

    let cost = calculate_monthly_cost(&state);
    let expected_cost = PLASTIC_BAG_BAN_MONTHLY_COST + COMPOSTING_MANDATE_MONTHLY_COST;
    assert!((cost - expected_cost).abs() < f64::EPSILON);
}

#[test]
fn test_infrastructure_costs_only_paid_once() {
    let mut state = WastePolicyState::default();
    state.deposit_return_program = true;
    state.composting_mandate = true;

    // First time: both costs pending
    let cost1 = calculate_pending_infrastructure_cost(&state);
    assert!(
        (cost1 - (DEPOSIT_RETURN_INFRASTRUCTURE_COST + COMPOSTING_MANDATE_ENFORCEMENT_COST)).abs()
            < f64::EPSILON
    );

    // Mark as paid
    state.deposit_return_built = true;
    state.composting_mandate_setup_paid = true;

    // Second time: no costs
    let cost2 = calculate_pending_infrastructure_cost(&state);
    assert!(cost2.abs() < f64::EPSILON);
}

#[test]
fn test_wte_requires_both_mandate_and_incinerators() {
    // No mandate, has incinerators -> no diversion
    assert!(calculate_wte_diversion(false, 100.0, 2).abs() < f64::EPSILON);
    // Mandate, no incinerators -> no diversion
    assert!(calculate_wte_diversion(true, 100.0, 0).abs() < f64::EPSILON);
    // Mandate + incinerators -> diversion
    assert!(calculate_wte_diversion(true, 100.0, 1) > 0.0);
}

#[test]
fn test_deposit_return_provides_recycling_bonus() {
    let mut state = WastePolicyState::default();
    state.deposit_return_program = true;

    // Verify that the bonus is exactly 10%
    assert_eq!(DEPOSIT_RETURN_RECYCLING_BONUS, 0.10);

    // No happiness penalty from deposit/return
    let penalty = calculate_happiness_penalty(&state);
    assert!(penalty.abs() < f32::EPSILON);
}

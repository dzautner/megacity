//! Tests for waste policy state defaults, helper functions, and save/load.

use super::*;

// -------------------------------------------------------------------------
// Default state tests
// -------------------------------------------------------------------------

#[test]
fn test_default_all_policies_off() {
    let state = WastePolicyState::default();
    assert!(!state.plastic_bag_ban);
    assert!(!state.deposit_return_program);
    assert!(!state.composting_mandate);
    assert!(!state.wte_mandate);
}

#[test]
fn test_default_no_costs() {
    let state = WastePolicyState::default();
    assert_eq!(state.total_infrastructure_cost, 0.0);
    assert_eq!(state.total_operating_cost, 0.0);
}

#[test]
fn test_default_setup_flags_false() {
    let state = WastePolicyState::default();
    assert!(!state.deposit_return_built);
    assert!(!state.composting_mandate_setup_paid);
}

#[test]
fn test_default_effects_neutral() {
    let effects = WastePolicyEffects::default();
    assert_eq!(effects.waste_generation_multiplier, 0.0);
    assert_eq!(effects.recycling_rate_bonus, 0.0);
    assert_eq!(effects.composting_diversion_bonus, 0.0);
    assert_eq!(effects.wte_diversion_tons, 0.0);
    assert!(!effects.wte_active);
    assert_eq!(effects.happiness_penalty, 0.0);
    assert_eq!(effects.total_monthly_cost, 0.0);
    assert_eq!(effects.active_policy_count, 0);
}

// -------------------------------------------------------------------------
// Waste generation multiplier tests
// -------------------------------------------------------------------------

#[test]
fn test_no_policies_multiplier_is_one() {
    let state = WastePolicyState::default();
    let mult = calculate_waste_multiplier(&state);
    assert!((mult - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_plastic_bag_ban_reduces_waste_5pct() {
    let mut state = WastePolicyState::default();
    state.plastic_bag_ban = true;
    let mult = calculate_waste_multiplier(&state);
    assert!((mult - 0.95).abs() < f32::EPSILON);
}

#[test]
fn test_other_policies_dont_affect_multiplier() {
    let mut state = WastePolicyState::default();
    state.deposit_return_program = true;
    state.composting_mandate = true;
    state.wte_mandate = true;
    let mult = calculate_waste_multiplier(&state);
    assert!((mult - 1.0).abs() < f32::EPSILON);
}

// -------------------------------------------------------------------------
// Happiness penalty tests
// -------------------------------------------------------------------------

#[test]
fn test_no_policies_no_happiness_penalty() {
    let state = WastePolicyState::default();
    let penalty = calculate_happiness_penalty(&state);
    assert!(penalty.abs() < f32::EPSILON);
}

#[test]
fn test_plastic_bag_ban_happiness_penalty() {
    let mut state = WastePolicyState::default();
    state.plastic_bag_ban = true;
    let penalty = calculate_happiness_penalty(&state);
    assert!((penalty - PLASTIC_BAG_BAN_HAPPINESS_PENALTY).abs() < f32::EPSILON);
}

#[test]
fn test_composting_mandate_happiness_penalty() {
    let mut state = WastePolicyState::default();
    state.composting_mandate = true;
    let penalty = calculate_happiness_penalty(&state);
    assert!((penalty - COMPOSTING_MANDATE_HAPPINESS_PENALTY).abs() < f32::EPSILON);
}

#[test]
fn test_combined_happiness_penalty() {
    let mut state = WastePolicyState::default();
    state.plastic_bag_ban = true;
    state.composting_mandate = true;
    let penalty = calculate_happiness_penalty(&state);
    let expected = PLASTIC_BAG_BAN_HAPPINESS_PENALTY + COMPOSTING_MANDATE_HAPPINESS_PENALTY;
    assert!((penalty - expected).abs() < f32::EPSILON);
}

#[test]
fn test_deposit_return_no_happiness_penalty() {
    let mut state = WastePolicyState::default();
    state.deposit_return_program = true;
    let penalty = calculate_happiness_penalty(&state);
    assert!(penalty.abs() < f32::EPSILON);
}

#[test]
fn test_wte_mandate_no_happiness_penalty() {
    let mut state = WastePolicyState::default();
    state.wte_mandate = true;
    let penalty = calculate_happiness_penalty(&state);
    assert!(penalty.abs() < f32::EPSILON);
}

// -------------------------------------------------------------------------
// Monthly cost tests
// -------------------------------------------------------------------------

#[test]
fn test_no_policies_no_monthly_cost() {
    let state = WastePolicyState::default();
    let cost = calculate_monthly_cost(&state);
    assert!(cost.abs() < f64::EPSILON);
}

#[test]
fn test_plastic_bag_ban_monthly_cost() {
    let mut state = WastePolicyState::default();
    state.plastic_bag_ban = true;
    let cost = calculate_monthly_cost(&state);
    assert!((cost - PLASTIC_BAG_BAN_MONTHLY_COST).abs() < f64::EPSILON);
}

#[test]
fn test_deposit_return_monthly_cost() {
    let mut state = WastePolicyState::default();
    state.deposit_return_program = true;
    let cost = calculate_monthly_cost(&state);
    assert!((cost - DEPOSIT_RETURN_MONTHLY_COST).abs() < f64::EPSILON);
}

#[test]
fn test_composting_mandate_monthly_cost() {
    let mut state = WastePolicyState::default();
    state.composting_mandate = true;
    let cost = calculate_monthly_cost(&state);
    assert!((cost - COMPOSTING_MANDATE_MONTHLY_COST).abs() < f64::EPSILON);
}

#[test]
fn test_wte_mandate_monthly_cost() {
    let mut state = WastePolicyState::default();
    state.wte_mandate = true;
    let cost = calculate_monthly_cost(&state);
    assert!((cost - WTE_MANDATE_MONTHLY_COST).abs() < f64::EPSILON);
}

#[test]
fn test_all_policies_monthly_cost() {
    let mut state = WastePolicyState::default();
    state.plastic_bag_ban = true;
    state.deposit_return_program = true;
    state.composting_mandate = true;
    state.wte_mandate = true;
    let cost = calculate_monthly_cost(&state);
    let expected = PLASTIC_BAG_BAN_MONTHLY_COST
        + DEPOSIT_RETURN_MONTHLY_COST
        + COMPOSTING_MANDATE_MONTHLY_COST
        + WTE_MANDATE_MONTHLY_COST;
    assert!((cost - expected).abs() < f64::EPSILON);
}

// -------------------------------------------------------------------------
// Infrastructure cost tests
// -------------------------------------------------------------------------

#[test]
fn test_no_policies_no_infrastructure_cost() {
    let state = WastePolicyState::default();
    let cost = calculate_pending_infrastructure_cost(&state);
    assert!(cost.abs() < f64::EPSILON);
}

#[test]
fn test_deposit_return_infrastructure_cost() {
    let mut state = WastePolicyState::default();
    state.deposit_return_program = true;
    let cost = calculate_pending_infrastructure_cost(&state);
    assert!((cost - DEPOSIT_RETURN_INFRASTRUCTURE_COST).abs() < f64::EPSILON);
}

#[test]
fn test_deposit_return_already_built_no_cost() {
    let mut state = WastePolicyState::default();
    state.deposit_return_program = true;
    state.deposit_return_built = true;
    let cost = calculate_pending_infrastructure_cost(&state);
    assert!(cost.abs() < f64::EPSILON);
}

#[test]
fn test_composting_mandate_enforcement_cost() {
    let mut state = WastePolicyState::default();
    state.composting_mandate = true;
    let cost = calculate_pending_infrastructure_cost(&state);
    assert!((cost - COMPOSTING_MANDATE_ENFORCEMENT_COST).abs() < f64::EPSILON);
}

#[test]
fn test_composting_mandate_already_paid_no_cost() {
    let mut state = WastePolicyState::default();
    state.composting_mandate = true;
    state.composting_mandate_setup_paid = true;
    let cost = calculate_pending_infrastructure_cost(&state);
    assert!(cost.abs() < f64::EPSILON);
}

#[test]
fn test_both_infrastructure_costs() {
    let mut state = WastePolicyState::default();
    state.deposit_return_program = true;
    state.composting_mandate = true;
    let cost = calculate_pending_infrastructure_cost(&state);
    let expected = DEPOSIT_RETURN_INFRASTRUCTURE_COST + COMPOSTING_MANDATE_ENFORCEMENT_COST;
    assert!((cost - expected).abs() < f64::EPSILON);
}

// -------------------------------------------------------------------------
// Active policy count tests
// -------------------------------------------------------------------------

#[test]
fn test_no_active_policies() {
    let state = WastePolicyState::default();
    assert_eq!(count_active_policies(&state), 0);
}

#[test]
fn test_one_active_policy() {
    let mut state = WastePolicyState::default();
    state.plastic_bag_ban = true;
    assert_eq!(count_active_policies(&state), 1);
}

#[test]
fn test_all_active_policies() {
    let mut state = WastePolicyState::default();
    state.plastic_bag_ban = true;
    state.deposit_return_program = true;
    state.composting_mandate = true;
    state.wte_mandate = true;
    assert_eq!(count_active_policies(&state), 4);
}

// -------------------------------------------------------------------------
// Constant value verification tests
// -------------------------------------------------------------------------

#[test]
fn test_constant_values() {
    assert_eq!(PLASTIC_BAG_BAN_WASTE_REDUCTION, 0.05);
    assert_eq!(PLASTIC_BAG_BAN_HAPPINESS_PENALTY, 1.0);
    assert_eq!(DEPOSIT_RETURN_RECYCLING_BONUS, 0.10);
    assert_eq!(DEPOSIT_RETURN_INFRASTRUCTURE_COST, 500_000.0);
    assert_eq!(COMPOSTING_MANDATE_DIVERSION_BONUS, 0.15);
    assert_eq!(COMPOSTING_MANDATE_HAPPINESS_PENALTY, 2.0);
    assert_eq!(COMPOSTING_MANDATE_ENFORCEMENT_COST, 1_000_000.0);
    assert_eq!(WTE_MANDATE_DIVERSION_FRACTION, 0.80);
}

// -------------------------------------------------------------------------
// Saveable trait tests
// -------------------------------------------------------------------------

#[test]
fn test_saveable_skips_default() {
    use crate::Saveable;
    let state = WastePolicyState::default();
    assert!(state.save_to_bytes().is_none());
}

#[test]
fn test_saveable_saves_when_active() {
    use crate::Saveable;
    let mut state = WastePolicyState::default();
    state.plastic_bag_ban = true;
    assert!(state.save_to_bytes().is_some());
}

#[test]
fn test_saveable_roundtrip() {
    use crate::Saveable;
    let mut state = WastePolicyState::default();
    state.plastic_bag_ban = true;
    state.deposit_return_program = true;
    state.deposit_return_built = true;
    state.composting_mandate = true;
    state.composting_mandate_setup_paid = true;
    state.wte_mandate = true;
    state.total_infrastructure_cost = 1_500_000.0;
    state.total_operating_cost = 50_000.0;

    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = WastePolicyState::load_from_bytes(&bytes);

    assert_eq!(restored.plastic_bag_ban, state.plastic_bag_ban);
    assert_eq!(
        restored.deposit_return_program,
        state.deposit_return_program
    );
    assert_eq!(restored.deposit_return_built, state.deposit_return_built);
    assert_eq!(restored.composting_mandate, state.composting_mandate);
    assert_eq!(
        restored.composting_mandate_setup_paid,
        state.composting_mandate_setup_paid
    );
    assert_eq!(restored.wte_mandate, state.wte_mandate);
    assert!(
        (restored.total_infrastructure_cost - state.total_infrastructure_cost).abs() < f64::EPSILON
    );
    assert!((restored.total_operating_cost - state.total_operating_cost).abs() < f64::EPSILON);
}

#[test]
fn test_saveable_key() {
    use crate::Saveable;
    assert_eq!(WastePolicyState::SAVE_KEY, "waste_policies");
}

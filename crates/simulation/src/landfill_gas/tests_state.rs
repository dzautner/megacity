//! Unit tests for LandfillGasState defaults, serialization, and integration scenarios.

use super::calculations::*;
use super::constants::*;
use super::state::*;

// -------------------------------------------------------------------------
// LandfillGasState default tests
// -------------------------------------------------------------------------

#[test]
fn test_default_state_gas_generation_zero() {
    let state = LandfillGasState::default();
    assert!((state.total_gas_generation_cf_per_year).abs() < f64::EPSILON);
}

#[test]
fn test_default_state_fractions() {
    let state = LandfillGasState::default();
    assert!((state.methane_fraction - METHANE_FRACTION).abs() < f32::EPSILON);
    assert!((state.co2_fraction - CO2_FRACTION).abs() < f32::EPSILON);
}

#[test]
fn test_default_state_collection_inactive() {
    let state = LandfillGasState::default();
    assert!(!state.collection_active);
}

#[test]
fn test_default_state_collection_efficiency() {
    let state = LandfillGasState::default();
    assert!((state.collection_efficiency - COLLECTION_EFFICIENCY_DEFAULT).abs() < f32::EPSILON);
}

#[test]
fn test_default_state_electricity_zero() {
    let state = LandfillGasState::default();
    assert!((state.electricity_generated_mw).abs() < f32::EPSILON);
}

#[test]
fn test_default_state_uncaptured_methane_zero() {
    let state = LandfillGasState::default();
    assert!((state.uncaptured_methane_cf).abs() < f32::EPSILON);
}

#[test]
fn test_default_state_costs_zero() {
    let state = LandfillGasState::default();
    assert!((state.infrastructure_cost).abs() < f64::EPSILON);
    assert!((state.maintenance_cost_per_year).abs() < f64::EPSILON);
}

#[test]
fn test_default_state_fire_risk_zero() {
    let state = LandfillGasState::default();
    assert!((state.fire_explosion_risk).abs() < f32::EPSILON);
}

#[test]
fn test_default_state_landfill_counts_zero() {
    let state = LandfillGasState::default();
    assert_eq!(state.landfills_with_collection, 0);
    assert_eq!(state.total_landfills, 0);
}

// -------------------------------------------------------------------------
// Integration-style tests (simulating update_landfill_gas logic)
// -------------------------------------------------------------------------

#[test]
fn test_full_cycle_no_collection() {
    // Simulate: 2 landfills, 50 tons/day waste, no collection
    let mut state = LandfillGasState::default();
    let daily_waste_tons = 50.0;
    let total_landfills = 2_u32;

    state.total_landfills = total_landfills;
    state.collection_active = false;

    // Gas generation
    let gas_gen = calculate_gas_generation(daily_waste_tons);
    state.total_gas_generation_cf_per_year = gas_gen;
    assert!((gas_gen - 5_000.0).abs() < f64::EPSILON);

    // No collection => no electricity
    state.electricity_generated_mw = 0.0;

    // All methane uncaptured
    let uncaptured = calculate_uncaptured_methane(gas_gen, false, state.collection_efficiency);
    state.uncaptured_methane_cf = uncaptured as f32;
    // 5000 * 0.50 = 2500
    assert!((uncaptured - 2_500.0).abs() < 0.01);

    // No infra costs
    state.infrastructure_cost = 0.0;
    state.maintenance_cost_per_year = 0.0;

    // Fire risk from 2 unprotected landfills
    state.fire_explosion_risk = calculate_fire_risk(total_landfills);
    assert!(state.fire_explosion_risk > 0.0);
}

#[test]
fn test_full_cycle_with_collection() {
    // Simulate: 3 landfills, 1000 tons/day waste, collection active at 75%
    let mut state = LandfillGasState::default();
    let daily_waste_tons = 1000.0;
    let total_landfills = 3_u32;

    state.total_landfills = total_landfills;
    state.collection_active = true;
    state.collection_efficiency = COLLECTION_EFFICIENCY_DEFAULT;

    let landfills_with_collection = total_landfills;
    state.landfills_with_collection = landfills_with_collection;

    // Gas generation: 1000 * 100 = 100,000 cf/year
    let gas_gen = calculate_gas_generation(daily_waste_tons);
    state.total_gas_generation_cf_per_year = gas_gen;
    assert!((gas_gen - 100_000.0).abs() < f64::EPSILON);

    // Electricity: 1000 * 0.75 / 1000 = 0.75 MW
    let mw = calculate_electricity_mw(daily_waste_tons, state.collection_efficiency);
    state.electricity_generated_mw = mw as f32;
    assert!((mw - 0.75).abs() < 0.001);

    // Uncaptured methane: 100,000 * 0.50 * (1 - 0.75) = 12,500
    let uncaptured = calculate_uncaptured_methane(gas_gen, true, state.collection_efficiency);
    state.uncaptured_methane_cf = uncaptured as f32;
    assert!((uncaptured - 12_500.0).abs() < 0.01);

    // Infrastructure cost: 3 * $500,000 = $1,500,000
    state.infrastructure_cost =
        landfills_with_collection as f64 * COLLECTION_INFRA_COST_PER_LANDFILL;
    assert!((state.infrastructure_cost - 1_500_000.0).abs() < f64::EPSILON);

    // Maintenance: 3 * $20,000 = $60,000/year
    state.maintenance_cost_per_year =
        landfills_with_collection as f64 * MAINTENANCE_COST_PER_LANDFILL_YEAR;
    assert!((state.maintenance_cost_per_year - 60_000.0).abs() < f64::EPSILON);

    // Fire risk: all landfills have collection, so 0 without => risk = 0
    let landfills_without = total_landfills - landfills_with_collection;
    state.fire_explosion_risk = calculate_fire_risk(landfills_without);
    assert!((state.fire_explosion_risk).abs() < f32::EPSILON);
}

#[test]
fn test_full_cycle_mixed_collection_not_yet_modeled() {
    // Currently collection_active is a city-wide boolean.
    // When active, all landfills get collection.
    // When inactive, none get collection.
    // This test documents that behavior.
    let mut state = LandfillGasState::default();
    state.total_landfills = 5;
    state.collection_active = false;

    let landfills_with = if state.collection_active {
        state.total_landfills
    } else {
        0
    };
    state.landfills_with_collection = landfills_with;

    // All 5 landfills are without collection
    let without = state.total_landfills - landfills_with;
    assert_eq!(without, 5);

    state.fire_explosion_risk = calculate_fire_risk(without);
    // Risk should be > single landfill risk
    let risk_1 = calculate_fire_risk(1);
    assert!(state.fire_explosion_risk > risk_1);
}

#[test]
fn test_no_landfills_no_effects() {
    let mut state = LandfillGasState::default();
    state.total_landfills = 0;
    let daily_waste_tons = 100.0; // waste exists but no landfills

    let gas_gen = calculate_gas_generation(daily_waste_tons);
    state.total_gas_generation_cf_per_year = gas_gen;

    // Gas is still generated (waste decomposing wherever it ends up)
    assert!(gas_gen > 0.0);

    // Fire risk is 0 with 0 landfills
    state.fire_explosion_risk = calculate_fire_risk(0);
    assert!((state.fire_explosion_risk).abs() < f32::EPSILON);
}

#[test]
fn test_collection_activation_changes_outputs() {
    let daily_waste_tons = 500.0;
    let gas_gen = calculate_gas_generation(daily_waste_tons);

    // Before collection
    let methane_before = calculate_uncaptured_methane(gas_gen, false, 0.75);
    let mw_before = 0.0_f64; // no electricity without collection

    // After collection
    let methane_after = calculate_uncaptured_methane(gas_gen, true, 0.75);
    let mw_after = calculate_electricity_mw(daily_waste_tons, 0.75);

    // Methane should decrease with collection
    assert!(methane_after < methane_before);

    // Electricity should increase with collection
    assert!(mw_after > mw_before);

    // Specifically: methane_after should be 25% of methane_before
    assert!((methane_after / methane_before - 0.25).abs() < 0.001);
}

#[test]
fn test_higher_efficiency_reduces_uncaptured_methane() {
    let gas_gen = 10_000.0;
    let methane_50 = calculate_uncaptured_methane(gas_gen, true, 0.50);
    let methane_75 = calculate_uncaptured_methane(gas_gen, true, 0.75);
    let methane_90 = calculate_uncaptured_methane(gas_gen, true, 0.90);

    assert!(methane_75 < methane_50);
    assert!(methane_90 < methane_75);
}

#[test]
fn test_higher_efficiency_increases_electricity() {
    let daily_waste = 1000.0;
    let mw_50 = calculate_electricity_mw(daily_waste, 0.50);
    let mw_75 = calculate_electricity_mw(daily_waste, 0.75);
    let mw_90 = calculate_electricity_mw(daily_waste, 0.90);

    assert!(mw_75 > mw_50);
    assert!(mw_90 > mw_75);
}

#[test]
fn test_state_serialization_roundtrip() {
    let mut state = LandfillGasState::default();
    state.total_gas_generation_cf_per_year = 50_000.0;
    state.collection_active = true;
    state.electricity_generated_mw = 1.5;
    state.uncaptured_methane_cf = 6_250.0;
    state.infrastructure_cost = 2_000_000.0;
    state.maintenance_cost_per_year = 80_000.0;
    state.fire_explosion_risk = 0.0;
    state.landfills_with_collection = 4;
    state.total_landfills = 4;

    // Serialize to JSON and back
    let json = serde_json::to_string(&state).expect("serialize");
    let deserialized: LandfillGasState = serde_json::from_str(&json).expect("deserialize");

    assert!((deserialized.total_gas_generation_cf_per_year - 50_000.0).abs() < f64::EPSILON);
    assert!(deserialized.collection_active);
    assert!((deserialized.electricity_generated_mw - 1.5).abs() < f32::EPSILON);
    assert!((deserialized.uncaptured_methane_cf - 6_250.0).abs() < f32::EPSILON);
    assert!((deserialized.infrastructure_cost - 2_000_000.0).abs() < f64::EPSILON);
    assert!((deserialized.maintenance_cost_per_year - 80_000.0).abs() < f64::EPSILON);
    assert!((deserialized.fire_explosion_risk).abs() < f32::EPSILON);
    assert_eq!(deserialized.landfills_with_collection, 4);
    assert_eq!(deserialized.total_landfills, 4);
}

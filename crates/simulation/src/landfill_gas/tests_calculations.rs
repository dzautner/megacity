//! Unit tests for landfill gas calculation functions and constants.

use super::calculations::*;
use super::constants::*;

// -------------------------------------------------------------------------
// Constants verification tests
// -------------------------------------------------------------------------

#[test]
fn test_gas_generation_constant() {
    assert!((GAS_GENERATION_CF_PER_TON_PER_YEAR - 100.0).abs() < f64::EPSILON);
}

#[test]
fn test_methane_fraction_constant() {
    assert!((METHANE_FRACTION - 0.50).abs() < f32::EPSILON);
}

#[test]
fn test_co2_fraction_constant() {
    assert!((CO2_FRACTION - 0.50).abs() < f32::EPSILON);
}

#[test]
fn test_fractions_sum_to_one() {
    assert!((METHANE_FRACTION + CO2_FRACTION - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_mw_per_1000_tons_day_constant() {
    assert!((MW_PER_1000_TONS_DAY - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_collection_efficiency_default_constant() {
    assert!((COLLECTION_EFFICIENCY_DEFAULT - 0.75).abs() < f32::EPSILON);
}

#[test]
fn test_infra_cost_per_landfill_constant() {
    assert!((COLLECTION_INFRA_COST_PER_LANDFILL - 500_000.0).abs() < f64::EPSILON);
}

#[test]
fn test_maintenance_cost_per_landfill_year_constant() {
    assert!((MAINTENANCE_COST_PER_LANDFILL_YEAR - 20_000.0).abs() < f64::EPSILON);
}

#[test]
fn test_fire_risk_annual_constant() {
    assert!((FIRE_RISK_ANNUAL_NO_COLLECTION - 0.001).abs() < f32::EPSILON);
}

#[test]
fn test_slow_ticks_per_year_constant() {
    assert!((SLOW_TICKS_PER_YEAR - 365.0).abs() < f64::EPSILON);
}

// -------------------------------------------------------------------------
// Gas generation calculation tests
// -------------------------------------------------------------------------

#[test]
fn test_gas_generation_zero_waste() {
    let result = calculate_gas_generation(0.0);
    assert!((result).abs() < f64::EPSILON);
}

#[test]
fn test_gas_generation_1000_tons() {
    // 1000 tons/day * 100 cf/ton/year = 100,000 cf/year
    let result = calculate_gas_generation(1000.0);
    assert!((result - 100_000.0).abs() < f64::EPSILON);
}

#[test]
fn test_gas_generation_scales_linearly() {
    let gen_10 = calculate_gas_generation(10.0);
    let gen_20 = calculate_gas_generation(20.0);
    assert!((gen_20 - 2.0 * gen_10).abs() < f64::EPSILON);
}

#[test]
fn test_gas_generation_small_input() {
    // 0.5 tons/day * 100 = 50 cf/year
    let result = calculate_gas_generation(0.5);
    assert!((result - 50.0).abs() < f64::EPSILON);
}

// -------------------------------------------------------------------------
// Electricity generation calculation tests
// -------------------------------------------------------------------------

#[test]
fn test_electricity_1000_tons_full_capture() {
    // 1000 tons/day, 100% capture => 1000 * 1.0 * 1.0 / 1000 = 1.0 MW
    let mw = calculate_electricity_mw(1000.0, 1.0);
    assert!((mw - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_electricity_1000_tons_75_capture() {
    // 1000 tons/day, 75% capture => 1000 * 0.75 / 1000 = 0.75 MW
    let mw = calculate_electricity_mw(1000.0, 0.75);
    assert!((mw - 0.75).abs() < 0.001);
}

#[test]
fn test_electricity_2000_tons_75_capture() {
    // 2000 tons/day, 75% capture => 2000 * 0.75 / 1000 = 1.5 MW
    let mw = calculate_electricity_mw(2000.0, 0.75);
    assert!((mw - 1.5).abs() < 0.001);
}

#[test]
fn test_electricity_zero_waste() {
    let mw = calculate_electricity_mw(0.0, 0.75);
    assert!((mw).abs() < f64::EPSILON);
}

#[test]
fn test_electricity_zero_efficiency() {
    let mw = calculate_electricity_mw(1000.0, 0.0);
    assert!((mw).abs() < f64::EPSILON);
}

#[test]
fn test_electricity_500_tons_default_efficiency() {
    // 500 tons/day, 75% => 500 * 0.75 / 1000 = 0.375 MW
    let mw = calculate_electricity_mw(500.0, COLLECTION_EFFICIENCY_DEFAULT);
    assert!((mw - 0.375).abs() < 0.001);
}

// -------------------------------------------------------------------------
// Uncaptured methane calculation tests
// -------------------------------------------------------------------------

#[test]
fn test_uncaptured_methane_no_collection() {
    // No collection: all methane escapes
    // 10,000 cf/year * 0.50 methane = 5,000 cf methane
    let result = calculate_uncaptured_methane(10_000.0, false, 0.75);
    assert!((result - 5_000.0).abs() < 0.01);
}

#[test]
fn test_uncaptured_methane_with_collection() {
    // With 75% collection: only 25% of methane escapes
    // 10,000 cf/year * 0.50 methane * (1 - 0.75) = 1,250 cf methane
    let result = calculate_uncaptured_methane(10_000.0, true, 0.75);
    assert!((result - 1_250.0).abs() < 0.01);
}

#[test]
fn test_uncaptured_methane_perfect_collection() {
    // With 100% collection: no methane escapes
    let result = calculate_uncaptured_methane(10_000.0, true, 1.0);
    assert!((result).abs() < 0.01);
}

#[test]
fn test_uncaptured_methane_zero_gas() {
    let result = calculate_uncaptured_methane(0.0, false, 0.75);
    assert!((result).abs() < f64::EPSILON);
}

#[test]
fn test_uncaptured_methane_collection_active_but_zero_efficiency() {
    // Collection active but 0% efficiency: all methane escapes
    // 10,000 * 0.50 * (1 - 0) = 5,000
    let result = calculate_uncaptured_methane(10_000.0, true, 0.0);
    assert!((result - 5_000.0).abs() < 0.01);
}

// -------------------------------------------------------------------------
// Fire/explosion risk calculation tests
// -------------------------------------------------------------------------

#[test]
fn test_fire_risk_zero_landfills() {
    let risk = calculate_fire_risk(0);
    assert!((risk).abs() < f32::EPSILON);
}

#[test]
fn test_fire_risk_one_landfill() {
    // 1 landfill: risk = 1 - (1 - 0.001)^1 = 0.001
    let risk = calculate_fire_risk(1);
    assert!((risk - 0.001).abs() < 0.0001);
}

#[test]
fn test_fire_risk_two_landfills() {
    // 2 landfills: risk = 1 - (1 - 0.001)^2 = 1 - 0.999^2 = 1 - 0.998001 = 0.001999
    let risk = calculate_fire_risk(2);
    let expected = 1.0 - (0.999_f32).powi(2);
    assert!((risk - expected).abs() < 0.0001);
}

#[test]
fn test_fire_risk_ten_landfills() {
    // 10 landfills: risk = 1 - (0.999)^10
    let risk = calculate_fire_risk(10);
    let expected = 1.0 - (0.999_f32).powi(10);
    assert!((risk - expected).abs() < 0.0001);
}

#[test]
fn test_fire_risk_increases_with_more_landfills() {
    let risk_1 = calculate_fire_risk(1);
    let risk_5 = calculate_fire_risk(5);
    let risk_10 = calculate_fire_risk(10);
    assert!(risk_5 > risk_1);
    assert!(risk_10 > risk_5);
}

#[test]
fn test_fire_risk_never_exceeds_one() {
    let risk = calculate_fire_risk(10_000);
    assert!(risk <= 1.0);
    assert!(risk > 0.0);
}

// -------------------------------------------------------------------------
// Annual risk to per-tick conversion tests
// -------------------------------------------------------------------------

#[test]
fn test_per_tick_zero_annual_risk() {
    let per_tick = annual_risk_to_per_tick(0.0);
    assert!((per_tick).abs() < f32::EPSILON);
}

#[test]
fn test_per_tick_one_annual_risk() {
    let per_tick = annual_risk_to_per_tick(1.0);
    assert!((per_tick - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_per_tick_small_annual_risk() {
    // For small p, per-tick ~ p/N
    let annual = 0.001_f32;
    let per_tick = annual_risk_to_per_tick(annual);
    // Should be approximately 0.001 / 365 ~ 0.00000274
    let approx = annual / SLOW_TICKS_PER_YEAR as f32;
    assert!((per_tick - approx).abs() < 0.0001);
}

#[test]
fn test_per_tick_reconstructs_annual() {
    // Verify: (1 - per_tick)^365 should approximate (1 - annual)
    let annual = 0.01_f32;
    let per_tick = annual_risk_to_per_tick(annual);
    let reconstructed = 1.0 - (1.0 - per_tick).powi(SLOW_TICKS_PER_YEAR as i32);
    assert!((reconstructed - annual).abs() < 0.001);
}

// -------------------------------------------------------------------------
// Infrastructure and maintenance cost tests
// -------------------------------------------------------------------------

#[test]
fn test_infra_cost_zero_landfills() {
    let cost = 0_u32 as f64 * COLLECTION_INFRA_COST_PER_LANDFILL;
    assert!((cost).abs() < f64::EPSILON);
}

#[test]
fn test_infra_cost_one_landfill() {
    let cost = 1_u32 as f64 * COLLECTION_INFRA_COST_PER_LANDFILL;
    assert!((cost - 500_000.0).abs() < f64::EPSILON);
}

#[test]
fn test_infra_cost_three_landfills() {
    let cost = 3_u32 as f64 * COLLECTION_INFRA_COST_PER_LANDFILL;
    assert!((cost - 1_500_000.0).abs() < f64::EPSILON);
}

#[test]
fn test_maintenance_cost_two_landfills() {
    let cost = 2_u32 as f64 * MAINTENANCE_COST_PER_LANDFILL_YEAR;
    assert!((cost - 40_000.0).abs() < f64::EPSILON);
}

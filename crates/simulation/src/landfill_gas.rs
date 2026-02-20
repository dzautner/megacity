//! Landfill Gas (LFG) Collection and Energy Generation (WASTE-956).
//!
//! Landfills produce landfill gas (LFG) — a mixture of roughly 50% methane and
//! 50% CO2 — as organic waste decomposes anaerobically. Without active gas
//! collection infrastructure, all methane escapes into the atmosphere as a potent
//! greenhouse gas and poses fire/explosion risks.
//!
//! When gas collection is active, a fraction of the generated gas is captured
//! (default 75% efficiency) and can be converted to electricity via gas-to-energy
//! turbines. The conversion rate is approximately 1 MW per 1,000 tons/day of
//! waste in the landfill.
//!
//! Key design points:
//! - Gas generation: 100 cubic feet per ton of waste per year
//! - Methane/CO2 split: 50/50
//! - Collection efficiency: 75% default (configurable)
//! - Electricity conversion: 1 MW per 1,000 tons/day of waste
//! - Fire/explosion risk: 0.1% annual probability without collection
//! - Infrastructure cost: $500K per landfill, $20K/year maintenance

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::garbage::WasteSystem;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Cubic feet of landfill gas generated per ton of waste per year.
pub const GAS_GENERATION_CF_PER_TON_PER_YEAR: f64 = 100.0;

/// Fraction of landfill gas that is methane (CH4).
pub const METHANE_FRACTION: f32 = 0.50;

/// Fraction of landfill gas that is carbon dioxide (CO2).
pub const CO2_FRACTION: f32 = 0.50;

/// Megawatts of electricity generated per 1,000 tons/day of waste in landfill.
pub const MW_PER_1000_TONS_DAY: f64 = 1.0;

/// Default collection efficiency (75% of generated gas is captured).
pub const COLLECTION_EFFICIENCY_DEFAULT: f32 = 0.75;

/// Capital cost to install gas collection infrastructure at one landfill ($500K).
pub const COLLECTION_INFRA_COST_PER_LANDFILL: f64 = 500_000.0;

/// Annual maintenance cost per landfill with gas collection ($20K/year).
pub const MAINTENANCE_COST_PER_LANDFILL_YEAR: f64 = 20_000.0;

/// Annual probability of fire/explosion at a landfill without gas collection.
pub const FIRE_RISK_ANNUAL_NO_COLLECTION: f32 = 0.001;

/// Number of slow ticks that represent one year (for annualizing per-tick calculations).
/// Each slow tick represents roughly 1 game-day, so 365 ticks = 1 year.
pub const SLOW_TICKS_PER_YEAR: f64 = 365.0;

// =============================================================================
// LandfillGasState resource
// =============================================================================

/// City-wide landfill gas generation and collection state.
///
/// Tracks gas generation from all landfills, collection infrastructure status,
/// electricity output, uncaptured methane emissions, and fire/explosion risk.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct LandfillGasState {
    /// Total gas generation across all landfills in cubic feet per year.
    pub total_gas_generation_cf_per_year: f64,
    /// Fraction of generated gas that is methane (nominally 0.50).
    pub methane_fraction: f32,
    /// Fraction of generated gas that is CO2 (nominally 0.50).
    pub co2_fraction: f32,
    /// Whether gas collection infrastructure exists in the city.
    pub collection_active: bool,
    /// Fraction of generated gas that is captured when collection is active (0.0-1.0).
    pub collection_efficiency: f32,
    /// Electricity generated from captured landfill gas in megawatts.
    pub electricity_generated_mw: f32,
    /// Uncaptured methane escaping to atmosphere in cubic feet per year.
    pub uncaptured_methane_cf: f32,
    /// Total capital cost of all collection systems installed.
    pub infrastructure_cost: f64,
    /// Total annual maintenance cost for all collection systems.
    pub maintenance_cost_per_year: f64,
    /// Annual probability of fire/explosion (aggregated across landfills without collection).
    pub fire_explosion_risk: f32,
    /// Number of landfills that have gas collection infrastructure.
    pub landfills_with_collection: u32,
    /// Total number of landfill service buildings in the city.
    pub total_landfills: u32,
}

impl Default for LandfillGasState {
    fn default() -> Self {
        Self {
            total_gas_generation_cf_per_year: 0.0,
            methane_fraction: METHANE_FRACTION,
            co2_fraction: CO2_FRACTION,
            collection_active: false,
            collection_efficiency: COLLECTION_EFFICIENCY_DEFAULT,
            electricity_generated_mw: 0.0,
            uncaptured_methane_cf: 0.0,
            infrastructure_cost: 0.0,
            maintenance_cost_per_year: 0.0,
            fire_explosion_risk: 0.0,
            landfills_with_collection: 0,
            total_landfills: 0,
        }
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Calculate total gas generation in cubic feet per year from daily waste input.
///
/// Each ton of waste in the landfill generates `GAS_GENERATION_CF_PER_TON_PER_YEAR`
/// cubic feet of gas per year. The `daily_waste_tons` parameter represents the
/// total daily waste throughput across all landfills (treated as the accumulated
/// waste mass for generation purposes).
pub fn calculate_gas_generation(daily_waste_tons: f64) -> f64 {
    daily_waste_tons * GAS_GENERATION_CF_PER_TON_PER_YEAR
}

/// Calculate electricity generated in MW from captured gas.
///
/// Conversion: 1 MW per 1,000 tons/day of waste in the landfill.
/// The captured gas fraction determines how much of the gas is available for
/// energy generation.
pub fn calculate_electricity_mw(daily_waste_tons: f64, collection_efficiency: f32) -> f64 {
    let captured_fraction = collection_efficiency as f64;
    let effective_waste = daily_waste_tons * captured_fraction;
    effective_waste * MW_PER_1000_TONS_DAY / 1000.0
}

/// Calculate uncaptured methane in cubic feet per year.
///
/// If collection is active, only the uncaptured fraction escapes.
/// If collection is not active, all methane escapes.
pub fn calculate_uncaptured_methane(
    total_gas_cf_per_year: f64,
    collection_active: bool,
    collection_efficiency: f32,
) -> f64 {
    let methane_cf = total_gas_cf_per_year * METHANE_FRACTION as f64;
    if collection_active {
        methane_cf * (1.0 - collection_efficiency as f64)
    } else {
        methane_cf
    }
}

/// Calculate the annual fire/explosion risk probability.
///
/// Each landfill without gas collection contributes an independent
/// `FIRE_RISK_ANNUAL_NO_COLLECTION` probability per year. We convert to
/// per-tick probability and then check per tick, but we also report the
/// annualized aggregate risk for display.
///
/// For N independent landfills without collection, the combined annual
/// probability of at least one fire is: 1 - (1 - p)^N
pub fn calculate_fire_risk(landfills_without_collection: u32) -> f32 {
    if landfills_without_collection == 0 {
        return 0.0;
    }
    let p_no_fire_each = 1.0 - FIRE_RISK_ANNUAL_NO_COLLECTION;
    let p_no_fire_all = p_no_fire_each.powi(landfills_without_collection as i32);
    1.0 - p_no_fire_all
}

/// Calculate the per-tick fire/explosion risk from the annual risk.
///
/// Converts annual probability to per-slow-tick probability so that
/// over SLOW_TICKS_PER_YEAR ticks, the aggregate matches the annual rate.
///
/// Per-tick probability: 1 - (1 - p_annual)^(1/N)
pub fn annual_risk_to_per_tick(annual_risk: f32) -> f32 {
    if annual_risk <= 0.0 {
        return 0.0;
    }
    if annual_risk >= 1.0 {
        return 1.0;
    }
    let exponent = 1.0 / SLOW_TICKS_PER_YEAR as f32;
    1.0 - (1.0 - annual_risk).powf(exponent)
}

// =============================================================================
// System
// =============================================================================

/// Updates landfill gas state each slow tick.
///
/// 1. Queries ServiceBuilding for Landfill type, counts total landfills.
/// 2. Calculates gas generation: daily_waste_input * 100 cubic_ft_per_ton_per_year.
/// 3. If collection_active: electricity = captured_gas * conversion_factor.
/// 4. Conversion: 1 MW per 1,000 tons/day of waste in landfill.
/// 5. If no collection: all methane is uncaptured (GHG emission).
/// 6. Fire/explosion risk: 0.001 per year without collection (checked each slow tick).
/// 7. Updates all state fields.
pub fn update_landfill_gas(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<LandfillGasState>,
    waste_system: Res<WasteSystem>,
    services: Query<&ServiceBuilding>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Count total landfills ---
    let mut total_landfills: u32 = 0;
    for service in &services {
        if service.service_type == ServiceType::Landfill {
            total_landfills += 1;
        }
    }
    state.total_landfills = total_landfills;

    // --- Phase 2: Calculate gas generation ---
    // Use the city-wide waste generation from WasteSystem as the daily waste input.
    // total_generated_tons is the cumulative total; period_generated_tons is per-tick.
    // Each slow tick represents ~1 game-day, so period_generated_tons ~ daily waste.
    let daily_waste_tons = waste_system.period_generated_tons;
    let gas_generation_cf_per_year = calculate_gas_generation(daily_waste_tons);
    state.total_gas_generation_cf_per_year = gas_generation_cf_per_year;

    // --- Phase 3: Determine collection status ---
    // Collection is active if the flag is set AND there are landfills with collection.
    // The number of landfills with collection equals total_landfills when collection
    // infrastructure has been built (controlled by collection_active flag).
    let landfills_with_collection = if state.collection_active {
        total_landfills
    } else {
        0
    };
    state.landfills_with_collection = landfills_with_collection;

    let landfills_without_collection = total_landfills - landfills_with_collection;

    // --- Phase 4: Calculate electricity generation ---
    if state.collection_active && total_landfills > 0 {
        let electricity_mw =
            calculate_electricity_mw(daily_waste_tons, state.collection_efficiency);
        state.electricity_generated_mw = electricity_mw as f32;
    } else {
        state.electricity_generated_mw = 0.0;
    }

    // --- Phase 5: Calculate uncaptured methane ---
    let uncaptured_methane = calculate_uncaptured_methane(
        gas_generation_cf_per_year,
        state.collection_active,
        state.collection_efficiency,
    );
    state.uncaptured_methane_cf = uncaptured_methane as f32;

    // --- Phase 6: Calculate infrastructure and maintenance costs ---
    state.infrastructure_cost =
        landfills_with_collection as f64 * COLLECTION_INFRA_COST_PER_LANDFILL;
    state.maintenance_cost_per_year =
        landfills_with_collection as f64 * MAINTENANCE_COST_PER_LANDFILL_YEAR;

    // --- Phase 7: Calculate fire/explosion risk ---
    state.fire_explosion_risk = calculate_fire_risk(landfills_without_collection);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
}

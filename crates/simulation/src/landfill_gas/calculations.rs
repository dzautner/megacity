//! Helper functions for landfill gas calculations.

use super::constants::*;

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

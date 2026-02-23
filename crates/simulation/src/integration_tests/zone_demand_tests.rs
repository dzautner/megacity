//! TEST-006: Unit and integration tests for zone demand calculation.
//!
//! Covers acceptance criteria:
//! - Residential demand increases with job surplus
//! - Commercial demand increases with population surplus
//! - Industrial demand follows economic conditions (labor supply)
//! - Zero-population edge cases
//!
//! Uses the pure `compute_market_demand` function directly for deterministic,
//! ECS-free testing and `TestCity` for full integration validation.

use crate::zones::demand::ZoneDemand;
use crate::zones::market::compute_market_demand;
use crate::zones::stats::ZoneStats;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Construct a `ZoneStats` for testing the pure demand function.
fn stats(
    has_roads: bool,
    pop: u32,
    r_cap: u32,
    r_occ: u32,
    c_cap: u32,
    c_occ: u32,
    i_cap: u32,
    i_occ: u32,
    o_cap: u32,
    o_occ: u32,
) -> ZoneStats {
    ZoneStats {
        population: pop,
        residential_capacity: r_cap,
        residential_occupants: r_occ,
        commercial_capacity: c_cap,
        commercial_occupants: c_occ,
        industrial_capacity: i_cap,
        industrial_occupants: i_occ,
        office_capacity: o_cap,
        office_occupants: o_occ,
        total_job_capacity: c_cap + i_cap + o_cap,
        total_job_occupants: c_occ + i_occ + o_occ,
        has_roads,
    }
}

// ===========================================================================
// AC-1: Residential demand increases with job surplus
// ===========================================================================

#[test]
fn test_residential_demand_rises_when_jobs_outnumber_workers() {
    // Baseline: balanced city — 500 residents, 500 job slots, all occupied.
    let balanced = stats(true, 500, 500, 500, 200, 200, 200, 200, 100, 100);
    let (r_balanced, _, _, _) = compute_market_demand(&balanced);

    // Job surplus: same residents but we add 1500 empty job slots.
    let job_surplus = stats(true, 500, 500, 500, 700, 200, 700, 200, 600, 100);
    let (r_surplus, _, _, _) = compute_market_demand(&job_surplus);

    assert!(
        r_surplus > r_balanced,
        "Residential demand should increase with job surplus: surplus={r_surplus} vs balanced={r_balanced}"
    );
}

#[test]
fn test_residential_demand_scales_with_increasing_job_availability() {
    // Three tiers of job availability with the same residential setup.
    // Residential at ~5% vacancy so the vacancy signal is neutral.
    let small_surplus = stats(true, 475, 500, 475, 200, 150, 100, 80, 50, 40);
    let medium_surplus = stats(true, 475, 500, 475, 500, 150, 300, 80, 200, 40);
    let large_surplus = stats(true, 475, 500, 475, 1000, 150, 800, 80, 500, 40);

    let (r_small, _, _, _) = compute_market_demand(&small_surplus);
    let (r_medium, _, _, _) = compute_market_demand(&medium_surplus);
    let (r_large, _, _, _) = compute_market_demand(&large_surplus);

    assert!(
        r_medium >= r_small,
        "Medium job surplus should produce >= residential demand than small: {r_medium} vs {r_small}"
    );
    assert!(
        r_large >= r_medium,
        "Large job surplus should produce >= residential demand than medium: {r_large} vs {r_medium}"
    );
}

// ===========================================================================
// AC-2: Commercial demand increases with population surplus
// ===========================================================================

#[test]
fn test_commercial_demand_rises_with_population_surplus() {
    // Scenario A: small population, some commercial.
    let low_pop = stats(true, 100, 500, 100, 200, 100, 100, 50, 50, 25);
    let (_, c_low, _, _) = compute_market_demand(&low_pop);

    // Scenario B: large population, same commercial capacity.
    // More people = more spending pressure on commercial.
    let high_pop = stats(true, 1000, 1500, 1000, 200, 100, 100, 50, 50, 25);
    let (_, c_high, _, _) = compute_market_demand(&high_pop);

    assert!(
        c_high > c_low,
        "Commercial demand should rise with more population: high_pop={c_high} vs low_pop={c_low}"
    );
}

#[test]
fn test_commercial_demand_high_when_population_exceeds_commercial_capacity() {
    // Large population, tiny commercial capacity = extreme spending pressure.
    let underserved = stats(true, 2000, 2000, 2000, 50, 50, 200, 200, 100, 100);
    let (_, c, _, _) = compute_market_demand(&underserved);

    assert!(
        c > 0.3,
        "Commercial demand should be high when population far exceeds commercial capacity, got {c}"
    );
}

#[test]
fn test_commercial_demand_low_when_population_small_and_capacity_large() {
    // Few residents, lots of empty commercial space.
    let oversupplied = stats(true, 50, 200, 50, 1000, 50, 200, 100, 100, 50);
    let (_, c, _, _) = compute_market_demand(&oversupplied);

    assert!(
        c < 0.3,
        "Commercial demand should be low when commercial is oversupplied relative to population, got {c}"
    );
}

// ===========================================================================
// AC-3: Industrial demand follows economic conditions
// ===========================================================================

#[test]
fn test_industrial_demand_rises_with_labor_supply() {
    // Keep industrial vacancy within natural range (~5-8%) so the vacancy
    // signal is near-neutral, isolating the labor supply factor.
    // Scenario A: low population = scarce labor, industrial ~6% vacancy.
    let scarce_labor = stats(true, 50, 100, 50, 100, 95, 100, 94, 50, 47);
    let (_, _, i_scarce, _) = compute_market_demand(&scarce_labor);

    // Scenario B: high population = abundant labor, same industrial vacancy.
    let abundant_labor = stats(true, 2000, 2100, 2000, 100, 95, 100, 94, 50, 47);
    let (_, _, i_abundant, _) = compute_market_demand(&abundant_labor);

    assert!(
        i_abundant > i_scarce,
        "Industrial demand should rise with more available labor: abundant={i_abundant} vs scarce={i_scarce}"
    );
}

#[test]
fn test_industrial_demand_drops_with_high_vacancy() {
    // Lots of industrial capacity, barely occupied = oversupplied.
    let oversupplied = stats(true, 500, 500, 500, 200, 200, 2000, 100, 100, 100);
    let (_, _, i, _) = compute_market_demand(&oversupplied);

    assert!(
        i < 0.3,
        "Industrial demand should be low when industrial vacancy is high, got {i}"
    );
}

#[test]
fn test_industrial_demand_high_when_fully_occupied_with_labor() {
    // All industrial slots full + large population (labor available).
    let tight_market = stats(true, 3000, 3000, 3000, 500, 500, 300, 300, 200, 200);
    let (_, _, i, _) = compute_market_demand(&tight_market);

    assert!(
        i > 0.3,
        "Industrial demand should be elevated when fully occupied with ample labor, got {i}"
    );
}

// ===========================================================================
// AC-4: Zero-population edge cases
// ===========================================================================

#[test]
fn test_zero_population_no_roads_produces_zero_demand() {
    let zs = stats(false, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    let (r, c, i, o) = compute_market_demand(&zs);
    assert_eq!(r, 0.0, "No roads: residential demand must be 0");
    assert_eq!(c, 0.0, "No roads: commercial demand must be 0");
    assert_eq!(i, 0.0, "No roads: industrial demand must be 0");
    assert_eq!(o, 0.0, "No roads: office demand must be 0");
}

#[test]
fn test_zero_population_with_roads_triggers_bootstrap() {
    // Roads exist but zero population and zero buildings = bootstrap demand.
    let zs = stats(true, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    let (r, c, i, o) = compute_market_demand(&zs);

    assert!(r > 0.0, "Bootstrap should produce positive residential demand");
    assert!(c > 0.0, "Bootstrap should produce positive commercial demand");
    assert!(i > 0.0, "Bootstrap should produce positive industrial demand");
    assert!(o > 0.0, "Bootstrap should produce positive office demand");

    // Residential bootstrap should be the strongest signal.
    assert!(
        r > c && r > i && r > o,
        "Residential bootstrap should dominate: r={r}, c={c}, i={i}, o={o}"
    );
}

#[test]
fn test_zero_population_with_empty_buildings() {
    // Buildings exist but nobody lives in them (ghost town scenario).
    // All occupants are 0, population is 0.
    let ghost = stats(true, 0, 500, 0, 200, 0, 100, 0, 50, 0);
    let (r, c, i, o) = compute_market_demand(&ghost);

    // 100% vacancy everywhere: vacancy signals should be strongly negative,
    // pushing demands down. Demand values should be at or near zero.
    assert!(
        r < 0.25,
        "Ghost town residential demand should be low, got {r}"
    );
    assert!(
        c < 0.15,
        "Ghost town commercial demand should be low, got {c}"
    );
    assert!(
        i < 0.15,
        "Ghost town industrial demand should be low, got {i}"
    );
    assert!(
        o < 0.15,
        "Ghost town office demand should be low, got {o}"
    );
}

#[test]
fn test_zero_population_commercial_demand_is_zero_without_roads() {
    // No roads, no population, some commercial capacity (shouldn't happen but
    // tests that the guard clause fires).
    let zs = stats(false, 0, 0, 0, 500, 0, 0, 0, 0, 0);
    let (_, c, _, _) = compute_market_demand(&zs);
    assert_eq!(
        c, 0.0,
        "Commercial demand must be 0 when there are no roads"
    );
}

// ===========================================================================
// Additional edge-case and regression tests
// ===========================================================================

#[test]
fn test_demand_for_accessor_returns_correct_values() {
    let demand = ZoneDemand {
        residential: 0.7,
        commercial: 0.4,
        industrial: 0.2,
        office: 0.1,
        ..Default::default()
    };
    use crate::grid::ZoneType;

    assert_eq!(demand.demand_for(ZoneType::ResidentialLow), 0.7);
    assert_eq!(demand.demand_for(ZoneType::ResidentialMedium), 0.7);
    assert_eq!(demand.demand_for(ZoneType::ResidentialHigh), 0.7);
    assert_eq!(demand.demand_for(ZoneType::CommercialLow), 0.4);
    assert_eq!(demand.demand_for(ZoneType::CommercialHigh), 0.4);
    assert_eq!(demand.demand_for(ZoneType::Industrial), 0.2);
    assert_eq!(demand.demand_for(ZoneType::Office), 0.1);
    // MixedUse picks max(residential, commercial) = max(0.7, 0.4) = 0.7
    assert_eq!(demand.demand_for(ZoneType::MixedUse), 0.7);
    assert_eq!(demand.demand_for(ZoneType::None), 0.0);
}

#[test]
fn test_all_demand_outputs_clamped_to_unit_interval() {
    // Stress-test a wide range of stats to ensure outputs are always in [0,1].
    let cases = vec![
        stats(true, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        stats(true, 1, 1, 1, 1, 1, 1, 1, 1, 1),
        stats(true, 100_000, 1, 100_000, 1, 100_000, 1, 100_000, 1, 100_000),
        stats(true, 0, 100_000, 0, 100_000, 0, 100_000, 0, 100_000, 0),
        stats(true, 50_000, 50_000, 50_000, 50_000, 50_000, 50_000, 50_000, 50_000, 50_000),
        stats(false, 999, 999, 999, 999, 999, 999, 999, 999, 999),
    ];
    for (idx, zs) in cases.iter().enumerate() {
        let (r, c, i, o) = compute_market_demand(zs);
        assert!(
            (0.0..=1.0).contains(&r),
            "Case {idx}: residential {r} out of [0,1]"
        );
        assert!(
            (0.0..=1.0).contains(&c),
            "Case {idx}: commercial {c} out of [0,1]"
        );
        assert!(
            (0.0..=1.0).contains(&i),
            "Case {idx}: industrial {i} out of [0,1]"
        );
        assert!(
            (0.0..=1.0).contains(&o),
            "Case {idx}: office {o} out of [0,1]"
        );
    }
}

#[test]
fn test_office_demand_grows_with_city_scale() {
    // Small city: office near natural vacancy, modest population.
    // Office at ~10% vacancy (within natural range of 8-12%).
    let small_city = stats(true, 200, 300, 200, 100, 95, 50, 47, 100, 90);
    let (_, _, _, o_small) = compute_market_demand(&small_city);

    // Large city: same office vacancy rate, much bigger population.
    // The population scale factor in office_workforce_factor should push
    // demand higher.
    let big_city = stats(true, 10_000, 12_000, 10_000, 3000, 2850, 1500, 1410, 100, 90);
    let (_, _, _, o_big) = compute_market_demand(&big_city);

    assert!(
        o_big > o_small,
        "Office demand should grow with city scale: big={o_big} vs small={o_small}"
    );
}

// ===========================================================================
// ECS integration tests using TestCity
// ===========================================================================

#[test]
fn test_zone_demand_resource_exists_in_testcity() {
    let city = crate::test_harness::TestCity::new();
    let demand = city.resource::<ZoneDemand>();
    // Default ZoneDemand should be all zeros.
    assert_eq!(demand.residential, 0.0);
    assert_eq!(demand.commercial, 0.0);
    assert_eq!(demand.industrial, 0.0);
    assert_eq!(demand.office, 0.0);
}

#[test]
fn test_zone_demand_updates_after_slow_tick_with_roads() {
    let mut city = crate::test_harness::TestCity::new()
        .with_road(128, 100, 128, 140, crate::grid::RoadType::Local);

    // Before any ticks, demand should be at default (0).
    let before = city.resource::<ZoneDemand>().clone();
    assert_eq!(before.residential, 0.0);

    // After a slow tick cycle, the demand system should have run and computed
    // bootstrap demand (roads exist, no buildings).
    city.tick_slow_cycle();

    let after = city.resource::<ZoneDemand>().clone();
    assert!(
        after.residential > 0.0,
        "Residential demand should be positive after slow tick with roads, got {}",
        after.residential
    );
}

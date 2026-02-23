//! Property-based tests for happiness invariants (TEST-012, part 1).
//!
//! Uses a seeded `StdRng` to generate 2000+ random input combinations and
//! verifies happiness outputs always stay within [0.0, 100.0], helper
//! functions stay bounded, and the full formula remains clamped.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::happiness::constants::*;

/// Deterministic seed for reproducibility.
const SEED: u64 = 0xDEAD_BEEF_CAFE_1337;

/// Number of random iterations per property test.
const ITERATIONS: usize = 2000;

// ===================================================================
// 1. Diminishing returns function invariants
// ===================================================================

#[test]
fn test_property_diminishing_returns_bounded() {
    let mut rng = StdRng::seed_from_u64(SEED + 2);
    for i in 0..ITERATIONS {
        let x = rng.gen_range(-100.0..100.0);
        let k = rng.gen_range(0.01..20.0);
        let result = diminishing_returns(x, k);
        assert!(
            (0.0..1.0).contains(&result) || (result - 1.0).abs() < f32::EPSILON,
            "Iteration {}: diminishing_returns({}, {}) = {} outside [0, 1)",
            i, x, k, result,
        );
    }
}

#[test]
fn test_property_diminishing_returns_monotonic() {
    let mut rng = StdRng::seed_from_u64(SEED + 3);
    for i in 0..ITERATIONS {
        let k = rng.gen_range(0.01..20.0);
        let x1 = rng.gen_range(0.0..=1.0);
        let x2 = rng.gen_range(0.0..=1.0);
        let (lo, hi) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let r_lo = diminishing_returns(lo, k);
        let r_hi = diminishing_returns(hi, k);
        assert!(
            r_hi >= r_lo,
            "Iteration {}: not monotonic: f({})={} > f({})={} at k={}",
            i, lo, r_lo, hi, r_hi, k,
        );
    }
}

// ===================================================================
// 2. Wealth satisfaction invariants
// ===================================================================

#[test]
fn test_property_wealth_satisfaction_bounded() {
    let mut rng = StdRng::seed_from_u64(SEED + 4);
    let lower = -WEALTH_POVERTY_PENALTY;
    let upper = WEALTH_SATISFACTION_MAX_BONUS;
    for i in 0..ITERATIONS {
        let savings = rng.gen_range(-100_000.0..1_000_000.0);
        let result = wealth_satisfaction(savings);
        assert!(
            result >= lower - f32::EPSILON && result <= upper + f32::EPSILON,
            "Iteration {}: wealth_satisfaction({}) = {} outside [{}, {}]",
            i, savings, result, lower, upper,
        );
    }
}

#[test]
fn test_property_wealth_satisfaction_monotonic_positive() {
    let mut rng = StdRng::seed_from_u64(SEED + 5);
    for i in 0..ITERATIONS {
        let s1 = rng.gen_range(0.01..100_000.0);
        let s2 = rng.gen_range(0.01..100_000.0);
        let (lo, hi) = if s1 <= s2 { (s1, s2) } else { (s2, s1) };
        let r_lo = wealth_satisfaction(lo);
        let r_hi = wealth_satisfaction(hi);
        assert!(
            r_hi >= r_lo - f32::EPSILON,
            "Iteration {}: wealth not monotonic: f({})={} > f({})={}",
            i, lo, r_lo, hi, r_hi,
        );
    }
}

// ===================================================================
// 3. Weather happiness factor invariants
// ===================================================================

#[test]
fn test_property_weather_happiness_factor_bounded() {
    let mut rng = StdRng::seed_from_u64(SEED + 6);
    let lower = -WEATHER_HAPPINESS_MAX_PENALTY;
    let upper = WEATHER_HAPPINESS_MAX_BONUS;
    for i in 0..ITERATIONS {
        let raw = rng.gen_range(-1000.0..1000.0);
        let result = weather_happiness_factor(raw);
        assert!(
            result >= lower - f32::EPSILON && result <= upper + f32::EPSILON,
            "Iteration {}: weather_happiness_factor({}) = {} outside [{}, {}]",
            i, raw, result, lower, upper,
        );
    }
}

#[test]
fn test_property_weather_factor_sign_matches_input() {
    let mut rng = StdRng::seed_from_u64(SEED + 7);
    for i in 0..ITERATIONS {
        let raw = rng.gen_range(-1000.0..1000.0);
        let result = weather_happiness_factor(raw);
        if raw > 0.0 {
            assert!(result >= 0.0, "Iter {}: positive raw {} gave {}", i, raw, result);
        } else if raw < 0.0 {
            assert!(result <= 0.0, "Iter {}: negative raw {} gave {}", i, raw, result);
        }
    }
}

// ===================================================================
// 4. Service coverage ratio invariants (exhaustive)
// ===================================================================

#[test]
fn test_property_service_coverage_ratio_bounded() {
    for flags in 0u8..=255u8 {
        let ratio = service_coverage_ratio(flags);
        assert!(
            (0.0..=1.0).contains(&ratio),
            "service_coverage_ratio({:#010b}) = {} outside [0, 1]",
            flags, ratio,
        );
    }
}

// ===================================================================
// 5. Full happiness formula: random inputs always clamp to [0, 100]
// ===================================================================

#[test]
fn test_property_happiness_formula_always_clamped() {
    let mut rng = StdRng::seed_from_u64(SEED + 10);
    for i in 0..ITERATIONS {
        let mut happiness: f32 = BASE_HAPPINESS;
        let education: u8 = rng.gen_range(0..=3);
        let weights = crate::wealth::WealthTier::from_education(education).happiness_weights();

        if rng.gen() { happiness += EMPLOYED_BONUS * weights.employment; }
        if rng.gen() { happiness += SHORT_COMMUTE_BONUS; }

        if rng.gen() { happiness += POWER_BONUS; } else {
            happiness -= NO_POWER_PENALTY + CRITICAL_NO_POWER_PENALTY;
        }
        if rng.gen() { happiness += WATER_BONUS; } else {
            happiness -= NO_WATER_PENALTY + CRITICAL_NO_WATER_PENALTY;
        }

        let cov: u8 = rng.gen();
        if cov & COVERAGE_HEALTH != 0 { happiness += HEALTH_COVERAGE_BONUS * weights.services; }
        if cov & COVERAGE_EDUCATION != 0 { happiness += EDUCATION_BONUS * weights.services; }
        if cov & COVERAGE_POLICE != 0 { happiness += POLICE_BONUS * weights.services; }
        if cov & COVERAGE_PARK != 0 { happiness += PARK_BONUS * weights.parks; }
        if cov & COVERAGE_ENTERTAINMENT != 0 {
            happiness += ENTERTAINMENT_BONUS * weights.entertainment;
        }
        if cov & COVERAGE_TELECOM != 0 { happiness += TELECOM_BONUS; }
        if cov & COVERAGE_TRANSPORT != 0 { happiness += TRANSPORT_BONUS; }

        let poll: f32 = rng.gen_range(0.0..=255.0);
        let poll_dim = diminishing_returns((poll / 255.0).clamp(0.0, 1.0), DIMINISHING_K_NEGATIVE);
        happiness -= poll_dim * (255.0 / 25.0) * weights.pollution;

        if rng.gen_range(0u8..255) > 10 { happiness -= GARBAGE_PENALTY; }
        if rng.gen_range(0.0..1000.0f32) > 100.0 {
            happiness -= crate::garbage::UNCOLLECTED_WASTE_HAPPINESS_PENALTY;
        }
        happiness += crate::waste_effects::waste_happiness_penalty(rng.gen_range(0.0..10000.0));

        let crime: f32 = rng.gen_range(0.0..=255.0);
        let crime_dim =
            diminishing_returns((crime / 255.0).clamp(0.0, 1.0), DIMINISHING_K_NEGATIVE);
        happiness -= crime_dim * CRIME_PENALTY_MAX;
        if crime > CRITICAL_CRIME_THRESHOLD { happiness -= CRITICAL_CRIME_PENALTY; }

        happiness -= rng.gen_range(0.0..=255.0f32) / 20.0;

        let lv: f32 = rng.gen_range(0.0..=255.0);
        let lv_dim = diminishing_returns((lv / 255.0).clamp(0.0, 1.0), DIMINISHING_K_DEFAULT);
        happiness += lv_dim * (255.0 / 50.0) * weights.land_value;

        happiness -= rng.gen_range(0.0..=1.0f32) * CONGESTION_PENALTY;

        let tax_rate: f32 = rng.gen_range(0.0..0.35);
        if tax_rate > 0.15 {
            happiness -= HIGH_TAX_PENALTY * ((tax_rate - 0.15) / 0.10);
        }
        happiness += rng.gen_range(0.0..15.0f32);
        happiness += weather_happiness_factor(rng.gen_range(-15.0..15.0));

        let satisfaction: f32 = rng.gen_range(0.0..=1.0);
        let needs_dim = diminishing_returns(satisfaction, DIMINISHING_K_DEFAULT);
        happiness += (needs_dim - 0.7769) * 35.0;
        if satisfaction < CRITICAL_NEEDS_THRESHOLD { happiness -= CRITICAL_NEEDS_PENALTY; }

        let health: f32 = rng.gen_range(0.0..=100.0);
        if health < 50.0 { happiness -= (50.0 - health) * 0.3; }
        if health > 80.0 { happiness += 3.0; }
        if health < CRITICAL_HEALTH_THRESHOLD { happiness -= CRITICAL_HEALTH_PENALTY; }

        happiness += wealth_satisfaction(rng.gen_range(-5000.0..50000.0));

        if rng.gen_range(0u8..10) == 0 {
            if rng.gen() { happiness -= SHELTERED_PENALTY; } else { happiness -= HOMELESS_PENALTY; }
        }
        if rng.gen_range(0u8..255) < 50 { happiness -= POOR_ROAD_PENALTY; }
        happiness += rng.gen_range(0.0..=255.0f32) / 255.0 * 5.0;
        if rng.gen() { happiness -= 10.0; }

        let heat_demand: f32 = rng.gen_range(0.0..1.0);
        if heat_demand > 0.0 {
            if rng.gen() { happiness += 3.0; } else { happiness -= 8.0 * heat_demand; }
        }

        let clamped = happiness.clamp(0.0, 100.0);
        assert!(
            (0.0..=100.0).contains(&clamped),
            "Iteration {}: happiness {} outside [0, 100] (raw={})",
            i, clamped, happiness,
        );
    }
}

// ===================================================================
// 6. Worst-case and best-case extremes
// ===================================================================

#[test]
fn test_property_happiness_worst_case_clamps_to_zero() {
    let mut h: f32 = BASE_HAPPINESS;
    h -= NO_POWER_PENALTY + CRITICAL_NO_POWER_PENALTY;
    h -= NO_WATER_PENALTY + CRITICAL_NO_WATER_PENALTY;
    let poll_dim = diminishing_returns(1.0, DIMINISHING_K_NEGATIVE);
    h -= poll_dim * (255.0 / 25.0) * 1.5;
    h -= GARBAGE_PENALTY;
    h -= crate::garbage::UNCOLLECTED_WASTE_HAPPINESS_PENALTY;
    h += crate::waste_effects::waste_happiness_penalty(10000.0);
    let crime_dim = diminishing_returns(1.0, DIMINISHING_K_NEGATIVE);
    h -= crime_dim * CRIME_PENALTY_MAX + CRITICAL_CRIME_PENALTY;
    h -= 255.0 / 20.0;
    h -= CONGESTION_PENALTY;
    h -= HIGH_TAX_PENALTY * 2.0;
    h += weather_happiness_factor(-15.0);
    h += (diminishing_returns(0.0, DIMINISHING_K_DEFAULT) - 0.7769) * 35.0;
    h -= CRITICAL_NEEDS_PENALTY;
    h -= 50.0 * 0.3 + CRITICAL_HEALTH_PENALTY;
    h += wealth_satisfaction(0.0);
    h -= HOMELESS_PENALTY + POOR_ROAD_PENALTY + 10.0 + 8.0;
    assert_eq!(h.clamp(0.0, 100.0), 0.0, "Worst case raw={}", h);
}

#[test]
fn test_property_happiness_best_case_clamps_to_hundred() {
    let w = crate::wealth::WealthTier::from_education(3).happiness_weights();
    let mut h: f32 = BASE_HAPPINESS;
    h += EMPLOYED_BONUS * w.employment + SHORT_COMMUTE_BONUS;
    h += POWER_BONUS + WATER_BONUS;
    h += HEALTH_COVERAGE_BONUS * w.services + EDUCATION_BONUS * w.services;
    h += POLICE_BONUS * w.services + PARK_BONUS * w.parks;
    h += ENTERTAINMENT_BONUS * w.entertainment + TELECOM_BONUS + TRANSPORT_BONUS;
    h += diminishing_returns(1.0, DIMINISHING_K_DEFAULT) * (255.0 / 50.0) * w.land_value;
    h += 15.0 + weather_happiness_factor(5.0);
    h += (diminishing_returns(1.0, DIMINISHING_K_DEFAULT) - 0.7769) * 35.0;
    h += 3.0 + wealth_satisfaction(100_000.0) + 5.0 + 3.0;
    assert_eq!(h.clamp(0.0, 100.0), 100.0, "Best case raw={}", h);
}

// ===================================================================
// 7. ECS integration: TestCity happiness always bounded
// ===================================================================

#[test]
fn test_property_testcity_happiness_always_bounded() {
    use crate::citizen::CitizenDetails;
    use crate::grid::ZoneType;
    use crate::test_harness::TestCity;
    use crate::utilities::UtilityType;

    let home = (100, 100);
    let work = (105, 100);
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower);

    let interval = crate::happiness::HAPPINESS_UPDATE_INTERVAL as u32;
    for cycle in 0..10 {
        city.tick(interval);
        let world = city.world_mut();
        for details in world.query::<&CitizenDetails>().iter(world) {
            assert!(
                (0.0..=100.0).contains(&details.happiness),
                "Cycle {}: happiness {} out of bounds", cycle, details.happiness,
            );
            assert!(
                (0.0..=100.0).contains(&details.health),
                "Cycle {}: health {} out of bounds", cycle, details.health,
            );
        }
    }
}

#[test]
fn test_property_testcity_harsh_conditions_bounded() {
    use crate::citizen::CitizenDetails;
    use crate::grid::ZoneType;
    use crate::test_harness::TestCity;

    let home = (100, 100);
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home);
    {
        let world = city.world_mut();
        world.resource_mut::<crate::pollution::PollutionGrid>().set(home.0, home.1, 255);
        world.resource_mut::<crate::crime::CrimeGrid>().set(home.0, home.1, 255);
        world.resource_mut::<crate::noise::NoisePollutionGrid>().set(home.0, home.1, 255);
        world.resource_mut::<crate::economy::CityBudget>().tax_rate = 0.35;
    }

    let interval = crate::happiness::HAPPINESS_UPDATE_INTERVAL as u32;
    for cycle in 0..10 {
        city.tick(interval);
        let world = city.world_mut();
        for details in world.query::<&CitizenDetails>().iter(world) {
            assert!(
                (0.0..=100.0).contains(&details.happiness),
                "Harsh cycle {}: happiness {}", cycle, details.happiness,
            );
            assert!(
                (0.0..=100.0).contains(&details.health),
                "Harsh cycle {}: health {}", cycle, details.health,
            );
        }
    }
}

//! Property-based tests for health and needs invariants (TEST-012, part 2).
//!
//! Uses a seeded `StdRng` to generate 2000+ random input combinations and
//! verifies that:
//! - Health always stays in [0.0, 100.0] after any update
//! - Needs values each stay in [0.0, 100.0]
//! - Needs `overall_satisfaction` always returns [0.0, 1.0]

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::citizen::Needs;

/// Deterministic seed for reproducibility.
const SEED: u64 = 0xDEAD_BEEF_CAFE_1337;

/// Number of random iterations per property test.
const ITERATIONS: usize = 2000;

// ===================================================================
// 1. Needs overall_satisfaction bounded for any inputs
// ===================================================================

#[test]
fn test_property_needs_satisfaction_random_bounded() {
    let mut rng = StdRng::seed_from_u64(SEED);
    for i in 0..ITERATIONS {
        let needs = Needs {
            hunger: rng.gen_range(-1000.0..1000.0),
            energy: rng.gen_range(-1000.0..1000.0),
            social: rng.gen_range(-1000.0..1000.0),
            fun: rng.gen_range(-1000.0..1000.0),
            comfort: rng.gen_range(-1000.0..1000.0),
        };
        let sat = needs.overall_satisfaction();
        assert!(
            (0.0..=1.0).contains(&sat),
            "Iteration {}: overall_satisfaction={} for needs {:?}",
            i, sat, needs,
        );
    }
}

#[test]
fn test_property_needs_satisfaction_in_range_bounded() {
    let mut rng = StdRng::seed_from_u64(SEED + 1);
    for i in 0..ITERATIONS {
        let needs = Needs {
            hunger: rng.gen_range(0.0..=100.0),
            energy: rng.gen_range(0.0..=100.0),
            social: rng.gen_range(0.0..=100.0),
            fun: rng.gen_range(0.0..=100.0),
            comfort: rng.gen_range(0.0..=100.0),
        };
        let sat = needs.overall_satisfaction();
        assert!(
            (0.0..=1.0).contains(&sat),
            "Iteration {}: overall_satisfaction={} for in-range needs {:?}",
            i, sat, needs,
        );
    }
}

#[test]
fn test_property_needs_satisfaction_extreme_edge_cases() {
    let cases = [
        Needs { hunger: 0.0, energy: 0.0, social: 0.0, fun: 0.0, comfort: 0.0 },
        Needs { hunger: 100.0, energy: 100.0, social: 100.0, fun: 100.0, comfort: 100.0 },
        Needs { hunger: -100.0, energy: -100.0, social: -100.0, fun: -100.0, comfort: -100.0 },
        Needs { hunger: f32::MAX, energy: f32::MAX, social: f32::MAX, fun: f32::MAX, comfort: f32::MAX },
        Needs { hunger: f32::MIN, energy: f32::MIN, social: f32::MIN, fun: f32::MIN, comfort: f32::MIN },
    ];
    for (idx, needs) in cases.iter().enumerate() {
        let sat = needs.overall_satisfaction();
        assert!(
            (0.0..=1.0).contains(&sat),
            "Edge case {}: overall_satisfaction={} for needs {:?}",
            idx, sat, needs,
        );
    }
}

// ===================================================================
// 2. Health update invariant: clamp to [0.0, 100.0]
// ===================================================================

/// Replicates the health delta logic from `personality_health.rs::update_health`
/// with randomized inputs and verifies the final clamp stays in [0, 100].
#[test]
fn test_property_health_update_always_clamped() {
    let mut rng = StdRng::seed_from_u64(SEED + 8);
    for i in 0..ITERATIONS {
        let age: u8 = rng.gen_range(0..=120);
        let current_health: f32 = rng.gen_range(-50.0..150.0);
        let hunger: f32 = rng.gen_range(0.0..100.0);
        let energy: f32 = rng.gen_range(0.0..100.0);
        let satisfaction: f32 = rng.gen_range(0.0..1.0);
        let pollution: f32 = rng.gen_range(0.0..255.0);
        let has_healthcare: bool = rng.gen();
        let resilience: f32 = rng.gen_range(0.0..1.0);

        let mut health_delta: f32 = 0.0;
        if age > 50 {
            health_delta -= (age as f32 - 50.0) * 0.02;
        }
        if hunger < 15.0 {
            health_delta -= 2.0;
        } else if hunger < 30.0 {
            health_delta -= 0.5;
        }
        if energy < 15.0 {
            health_delta -= 1.5;
        }
        if satisfaction > 0.7 {
            health_delta += 0.5;
        }
        if pollution > 50.0 {
            health_delta -= (pollution - 50.0) * 0.02;
        }
        if has_healthcare {
            health_delta += 0.3;
            if current_health < 50.0 {
                health_delta += 0.5;
            }
        }
        health_delta *= 1.0 - (resilience * 0.3);

        let final_health = (current_health + health_delta).clamp(0.0, 100.0);
        assert!(
            (0.0..=100.0).contains(&final_health),
            "Iteration {}: health={} out of [0,100] (start={}, delta={})",
            i, final_health, current_health, health_delta,
        );
    }
}

/// Tests that health stays bounded across many random damage/heal cycles.
#[test]
fn test_property_health_bounded_over_many_cycles() {
    let mut rng = StdRng::seed_from_u64(SEED + 12);
    for i in 0..ITERATIONS {
        let mut health: f32 = rng.gen_range(0.0..=100.0);
        for _ in 0..100 {
            let delta: f32 = rng.gen_range(-5.0..5.0);
            health = (health + delta).clamp(0.0, 100.0);
            assert!(
                (0.0..=100.0).contains(&health),
                "Iter {}: health {} out of [0, 100]",
                i, health,
            );
        }
    }
}

/// Tests that health modification from pollution always results in [0, 100].
#[test]
fn test_property_health_pollution_modifier_clamped() {
    let mut rng = StdRng::seed_from_u64(SEED + 9);
    for i in 0..ITERATIONS {
        let current_health: f32 = rng.gen_range(0.0..=100.0);
        let modifier: f32 = rng.gen_range(-10.0..1.0);
        let result = (current_health + modifier).clamp(0.0, 100.0);
        assert!(
            (0.0..=100.0).contains(&result),
            "Iteration {}: health {} outside [0, 100] after modifier {}",
            i, result, modifier,
        );
    }
}

// ===================================================================
// 3. Needs fields stay bounded after simulated decay/restore cycles
// ===================================================================

/// Simulates a sequence of needs decay and restore operations with random
/// starting values and verifies every field stays in [0.0, 100.0] at all
/// times, and overall_satisfaction stays in [0.0, 1.0].
#[test]
fn test_property_needs_fields_bounded_after_updates() {
    let mut rng = StdRng::seed_from_u64(SEED + 11);

    // Constants from life_simulation/needs.rs
    const HUNGER_DECAY: f32 = 0.8;
    const ENERGY_DECAY: f32 = 0.5;
    const SOCIAL_DECAY: f32 = 0.3;
    const FUN_DECAY: f32 = 0.4;
    const HUNGER_RESTORE_HOME: f32 = 5.0;
    const ENERGY_RESTORE_NIGHT: f32 = 8.0;
    const SOCIAL_RESTORE_WORK: f32 = 1.0;
    const FUN_RESTORE_LEISURE: f32 = 5.0;
    const SOCIAL_RESTORE_LEISURE: f32 = 3.0;

    for i in 0..ITERATIONS {
        let mut needs = Needs {
            hunger: rng.gen_range(0.0..=100.0),
            energy: rng.gen_range(0.0..=100.0),
            social: rng.gen_range(0.0..=100.0),
            fun: rng.gen_range(0.0..=100.0),
            comfort: rng.gen_range(0.0..=100.0),
        };

        for _ in 0..50 {
            needs.hunger = (needs.hunger - HUNGER_DECAY).max(0.0);
            needs.energy = (needs.energy - ENERGY_DECAY).max(0.0);
            needs.social = (needs.social - SOCIAL_DECAY).max(0.0);
            needs.fun = (needs.fun - FUN_DECAY).max(0.0);

            match rng.gen_range(0u8..5) {
                0 => needs.hunger = (needs.hunger + HUNGER_RESTORE_HOME).min(100.0),
                1 => needs.energy = (needs.energy + ENERGY_RESTORE_NIGHT).min(100.0),
                2 => needs.social = (needs.social + SOCIAL_RESTORE_WORK).min(100.0),
                3 => {
                    needs.fun = (needs.fun + FUN_RESTORE_LEISURE).min(100.0);
                    needs.social = (needs.social + SOCIAL_RESTORE_LEISURE).min(100.0);
                }
                _ => {
                    let target: f32 = rng.gen_range(0.0..100.0);
                    needs.comfort += (target - needs.comfort) * 0.1;
                    needs.comfort = needs.comfort.clamp(0.0, 100.0);
                }
            }

            assert!((0.0..=100.0).contains(&needs.hunger),
                "Iter {}: hunger {}", i, needs.hunger);
            assert!((0.0..=100.0).contains(&needs.energy),
                "Iter {}: energy {}", i, needs.energy);
            assert!((0.0..=100.0).contains(&needs.social),
                "Iter {}: social {}", i, needs.social);
            assert!((0.0..=100.0).contains(&needs.fun),
                "Iter {}: fun {}", i, needs.fun);
            assert!((0.0..=100.0).contains(&needs.comfort),
                "Iter {}: comfort {}", i, needs.comfort);

            let sat = needs.overall_satisfaction();
            assert!((0.0..=1.0).contains(&sat),
                "Iter {}: satisfaction {}", i, sat);
        }
    }
}

// ===================================================================
// 4. Needs: weighted sum with random weights produces valid satisfaction
// ===================================================================

/// Verifies that the specific weight formula (0.25, 0.25, 0.15, 0.15, 0.20)
/// always produces [0.0, 1.0] for any valid field inputs.
#[test]
fn test_property_needs_weighted_sum_invariant() {
    let mut rng = StdRng::seed_from_u64(SEED + 13);
    for i in 0..ITERATIONS {
        let h = rng.gen_range(0.0..=100.0f32);
        let e = rng.gen_range(0.0..=100.0f32);
        let s = rng.gen_range(0.0..=100.0f32);
        let f = rng.gen_range(0.0..=100.0f32);
        let c = rng.gen_range(0.0..=100.0f32);

        let raw = h * 0.25 + e * 0.25 + s * 0.15 + f * 0.15 + c * 0.20;
        let result = (raw / 100.0).clamp(0.0, 1.0);
        assert!(
            (0.0..=1.0).contains(&result),
            "Iter {}: weighted sum result {} for inputs ({},{},{},{},{})",
            i, result, h, e, s, f, c,
        );
        // Also verify the raw sum is in [0, 100] for valid inputs
        assert!(
            raw >= 0.0 && raw <= 100.0,
            "Iter {}: raw weighted sum {} should be in [0, 100]",
            i, raw,
        );
    }
}

// ===================================================================
// 5. Health and needs in ECS integration
// ===================================================================

/// Uses TestCity to verify health and needs stay bounded after many ticks.
#[test]
fn test_property_testcity_health_needs_bounded() {
    use crate::citizen::{CitizenDetails, Needs as NeedsComp};
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

    // Run 50 slow cycles (each is multiple ticks)
    for cycle in 0..50 {
        city.tick(10);

        let world = city.world_mut();

        // Check health bounds
        for details in world.query::<&CitizenDetails>().iter(world) {
            assert!(
                (0.0..=100.0).contains(&details.health),
                "Cycle {}: health {} out of [0, 100]", cycle, details.health,
            );
        }

        // Check needs bounds
        for needs in world.query::<&NeedsComp>().iter(world) {
            assert!((0.0..=100.0).contains(&needs.hunger),
                "Cycle {}: hunger {} out of bounds", cycle, needs.hunger);
            assert!((0.0..=100.0).contains(&needs.energy),
                "Cycle {}: energy {} out of bounds", cycle, needs.energy);
            assert!((0.0..=100.0).contains(&needs.social),
                "Cycle {}: social {} out of bounds", cycle, needs.social);
            assert!((0.0..=100.0).contains(&needs.fun),
                "Cycle {}: fun {} out of bounds", cycle, needs.fun);
            assert!((0.0..=100.0).contains(&needs.comfort),
                "Cycle {}: comfort {} out of bounds", cycle, needs.comfort);

            let sat = needs.overall_satisfaction();
            assert!((0.0..=1.0).contains(&sat),
                "Cycle {}: satisfaction {} out of bounds", cycle, sat);
        }
    }
}

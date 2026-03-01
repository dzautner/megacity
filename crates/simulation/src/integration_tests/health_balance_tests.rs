//! Integration tests for health happiness balance (issue #1950).
//!
//! Verifies that the rebalanced health happiness penalties and bonuses
//! stay within the new target bounds:
//! - Max health penalty (health=0): no worse than -20 (was -35)
//! - Healthy bonus (health=100): at least +8 (was +3)

use bevy::prelude::*;

use crate::citizen::{Citizen, CitizenDetails};
use crate::grid::ZoneType;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

/// Collect all citizen entities currently in the world (sorted for stability).
fn citizen_entities(city: &mut TestCity) -> Vec<Entity> {
    let world = city.world_mut();
    let mut entities: Vec<Entity> = world
        .query_filtered::<Entity, With<Citizen>>()
        .iter(world)
        .collect();
    entities.sort();
    entities
}

/// Test that a citizen with health=0 gets no more than -20 happiness from
/// health factors compared to a health=50 citizen (neutral baseline).
///
/// New balance:
/// - Linear penalty: (50 - 0) * 0.2 = 10
/// - Critical penalty (health < 30): 10
/// - Total: 20
#[test]
fn test_health_zero_penalty_capped_at_20() {
    let home = (100, 100);
    let work = (105, 100);
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_citizen(home, work)
        .with_utility(home.0 + 1, home.1, UtilityType::PowerPlant)
        .with_utility(home.0 - 1, home.1, UtilityType::WaterTower);

    // Capture the two entities we spawned (before any immigration runs)
    let initial = citizen_entities(&mut city);
    assert!(initial.len() >= 2, "Need at least 2 citizens spawned");
    let citizen_a = initial[0]; // Will have health=0
    let citizen_b = initial[1]; // Will have health=50

    // Set health: citizen_a = 0 (worst case), citizen_b = 50 (neutral)
    {
        let world = city.world_mut();
        world
            .get_mut::<CitizenDetails>(citizen_a)
            .unwrap()
            .health = 0.0;
        world
            .get_mut::<CitizenDetails>(citizen_b)
            .unwrap()
            .health = 50.0;
    }
    city.tick(21);

    // Re-set health (may have drifted from health system) and tick again
    {
        let world = city.world_mut();
        world
            .get_mut::<CitizenDetails>(citizen_a)
            .unwrap()
            .health = 0.0;
        world
            .get_mut::<CitizenDetails>(citizen_b)
            .unwrap()
            .health = 50.0;
    }
    city.tick(21);

    let world = city.world_mut();
    let h0 = world.get::<CitizenDetails>(citizen_a).unwrap().happiness;
    let h50 = world.get::<CitizenDetails>(citizen_b).unwrap().happiness;

    // The health=0 citizen should have at most 20 less happiness than health=50
    // (linear: 50*0.2=10, critical: 10, total: 20)
    let penalty = h50 - h0;
    assert!(
        penalty <= 21.0,
        "Health=0 penalty vs health=50 should be <= 20 (with 1.0 margin), \
         got {:.1}. h0={:.1}, h50={:.1}",
        penalty,
        h0,
        h50,
    );
    assert!(
        penalty >= 0.0,
        "Health=0 citizen should not be happier than health=50 citizen, \
         penalty={:.1}",
        penalty,
    );
}

/// Test that a citizen with health=100 gets at least +8 happiness bonus
/// compared to a health=50 citizen (neutral baseline).
///
/// Uses unemployed citizens (lower base happiness) with power/water to keep
/// happiness in the 30-70 range, avoiding both the 0 floor and 100 cap.
///
/// New balance:
/// - Healthy bonus (health > 80): +8
#[test]
fn test_health_100_bonus_at_least_8() {
    let home = (120, 120);
    // Use unemployed citizens: lower base happiness avoids the 100 cap
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home)
        .with_unemployed_citizen(home)
        .with_utility(home.0 + 1, home.1, UtilityType::PowerPlant)
        .with_utility(home.0 - 1, home.1, UtilityType::WaterTower);

    // Capture the two entities we spawned
    let initial = citizen_entities(&mut city);
    assert!(initial.len() >= 2, "Need at least 2 citizens spawned");
    let citizen_a = initial[0]; // Will have health=100
    let citizen_b = initial[1]; // Will have health=50

    // Set health and zero out savings to avoid wealth-based differences
    {
        let world = city.world_mut();
        let mut da = world.get_mut::<CitizenDetails>(citizen_a).unwrap();
        da.health = 100.0;
        da.savings = 0.0;
        let mut db = world.get_mut::<CitizenDetails>(citizen_b).unwrap();
        db.health = 50.0;
        db.savings = 0.0;
    }
    city.tick(21);

    // Re-set health and savings, then tick again for stability
    {
        let world = city.world_mut();
        let mut da = world.get_mut::<CitizenDetails>(citizen_a).unwrap();
        da.health = 100.0;
        da.savings = 0.0;
        let mut db = world.get_mut::<CitizenDetails>(citizen_b).unwrap();
        db.health = 50.0;
        db.savings = 0.0;
    }
    city.tick(21);

    let world = city.world_mut();
    let h100 = world
        .get::<CitizenDetails>(citizen_a)
        .unwrap()
        .happiness;
    let h50 = world.get::<CitizenDetails>(citizen_b).unwrap().happiness;

    // Verify neither citizen hit the floor or cap
    assert!(
        h100 < 99.0 && h100 > 1.0,
        "Health=100 citizen happiness should be in mid-range, got {:.1}",
        h100,
    );
    assert!(
        h50 < 99.0 && h50 > 1.0,
        "Health=50 citizen happiness should be in mid-range, got {:.1}",
        h50,
    );

    let bonus = h100 - h50;
    assert!(
        bonus >= 7.5,
        "Health=100 bonus vs health=50 should be >= 8 (with 0.5 margin), \
         got {:.1}. h100={:.1}, h50={:.1}",
        bonus,
        h100,
        h50,
    );
}

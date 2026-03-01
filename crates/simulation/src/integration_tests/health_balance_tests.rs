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

/// Set health for all citizens to the given values (indexed by spawn order).
fn set_citizen_health(city: &mut TestCity, health_values: &[f32]) {
    let world = city.world_mut();
    let mut entities: Vec<Entity> = world
        .query_filtered::<Entity, With<Citizen>>()
        .iter(world)
        .collect();
    entities.sort();

    for (i, &health) in health_values.iter().enumerate() {
        if i < entities.len() {
            if let Some(mut details) = world.get_mut::<CitizenDetails>(entities[i]) {
                details.health = health;
            }
        }
    }
}

/// Read happiness for all citizens (sorted by entity).
fn read_citizen_happiness(city: &mut TestCity) -> Vec<f32> {
    let world = city.world_mut();
    let mut entities: Vec<Entity> = world
        .query_filtered::<Entity, With<Citizen>>()
        .iter(world)
        .collect();
    entities.sort();

    entities
        .iter()
        .map(|&e| world.get::<CitizenDetails>(e).unwrap().happiness)
        .collect()
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

    // Set health: citizen 0 = 0 (worst case), citizen 1 = 50 (neutral)
    set_citizen_health(&mut city, &[0.0, 50.0]);
    city.tick(21);

    // Re-set health (may have drifted from health system) and tick again
    set_citizen_health(&mut city, &[0.0, 50.0]);
    city.tick(21);

    let happiness = read_citizen_happiness(&mut city);
    assert_eq!(happiness.len(), 2, "Expected 2 citizens");

    let penalty = happiness[1] - happiness[0];
    assert!(
        penalty <= 21.0,
        "Health=0 penalty vs health=50 should be <= 20 (with 1.0 margin), \
         got {:.1}. h0={:.1}, h50={:.1}",
        penalty,
        happiness[0],
        happiness[1],
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
/// New balance:
/// - Healthy bonus (health > 80): +8
#[test]
fn test_health_100_bonus_at_least_8() {
    let home = (120, 120);
    let work = (125, 120);
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_citizen(home, work)
        .with_utility(home.0 + 1, home.1, UtilityType::PowerPlant)
        .with_utility(home.0 - 1, home.1, UtilityType::WaterTower);

    // Set health: citizen 0 = 100 (healthy), citizen 1 = 50 (neutral)
    set_citizen_health(&mut city, &[100.0, 50.0]);
    city.tick(21);

    // Re-set health and tick again for stability
    set_citizen_health(&mut city, &[100.0, 50.0]);
    city.tick(21);

    let happiness = read_citizen_happiness(&mut city);
    assert_eq!(happiness.len(), 2, "Expected 2 citizens");

    let bonus = happiness[0] - happiness[1];
    assert!(
        bonus >= 7.5,
        "Health=100 bonus vs health=50 should be >= 8 (with 0.5 margin), \
         got {:.1}. h100={:.1}, h50={:.1}",
        bonus,
        happiness[0],
        happiness[1],
    );
}

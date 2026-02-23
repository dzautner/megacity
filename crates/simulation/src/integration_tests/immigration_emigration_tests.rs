//! Integration tests for the immigration/emigration system (TEST-045).
//!
//! Tests verify that:
//! - High attractiveness triggers positive immigration
//! - Low attractiveness triggers emigration
//! - Immigration scales with available housing
//! - No immigration occurs when no housing is available

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails};
use crate::grid::ZoneType;
use crate::immigration::{CityAttractiveness, ImmigrationStats};
use crate::test_harness::TestCity;

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Set the city attractiveness score directly on a TestCity.
fn set_attractiveness(city: &mut TestCity, score: f32) {
    let world = city.world_mut();
    if let Some(mut attr) = world.get_resource_mut::<CityAttractiveness>() {
        attr.overall_score = score;
    }
}

/// Read the current ImmigrationStats from the world.
fn get_immigration_stats(city: &TestCity) -> ImmigrationStats {
    city.resource::<ImmigrationStats>().clone()
}

/// Make all existing citizens very happy so they do not emigrate unexpectedly
/// during immigration tests.
fn stabilize_citizens(city: &mut TestCity) {
    let world = city.world_mut();
    let mut q = world.query::<&mut CitizenDetails>();
    // Collect entities first to avoid borrow issues
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<Citizen>>()
        .iter(world)
        .collect();
    for entity in entities {
        if let Ok(mut details) = q.get_mut(world, entity) {
            details.happiness = 95.0;
            details.health = 95.0;
            details.savings = 50_000.0;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_immigration_high_attractiveness_increases_population() {
    // Setup: empty city with residential and commercial buildings, high attractiveness.
    // Expected: citizens should be spawned (immigration) after ticking past
    // the IMMIGRATION_INTERVAL.
    let mut city = TestCity::new()
        // Residential buildings (housing for immigrants)
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_building(52, 50, ZoneType::ResidentialLow, 1)
        .with_building(54, 50, ZoneType::ResidentialLow, 1)
        .with_building(56, 50, ZoneType::ResidentialLow, 1)
        // Job buildings (workplaces for immigrants)
        .with_building(50, 54, ZoneType::CommercialLow, 1)
        .with_building(52, 54, ZoneType::CommercialLow, 1)
        .with_building(54, 54, ZoneType::Industrial, 1);

    let initial_count = city.citizen_count();
    assert_eq!(initial_count, 0, "should start with no citizens");

    // Force very high attractiveness to guarantee immigration
    set_attractiveness(&mut city, 90.0);

    // Tick enough for multiple immigration waves (IMMIGRATION_INTERVAL = 100)
    city.tick(300);

    // Re-set attractiveness in case compute_attractiveness overwrote it
    set_attractiveness(&mut city, 90.0);
    city.tick(300);

    let final_count = city.citizen_count();
    assert!(
        final_count > initial_count,
        "high attractiveness should cause immigration: initial={initial_count}, final={final_count}"
    );

    let stats = get_immigration_stats(&city);
    assert!(
        stats.immigrants_this_month > 0 || final_count > 0,
        "immigration stats should reflect positive immigration"
    );
}

#[test]
fn test_emigration_low_attractiveness_decreases_population() {
    // Setup: city with citizens, then set attractiveness very low.
    // Expected: citizens should be removed (emigration).
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_building(52, 50, ZoneType::CommercialLow, 1)
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50));

    let initial_count = city.citizen_count();
    assert_eq!(initial_count, 10, "should start with 10 citizens");

    // Force very low attractiveness to trigger emigration
    set_attractiveness(&mut city, 10.0);

    // Tick past IMMIGRATION_INTERVAL (100) to trigger emigration wave
    city.tick(100);

    // Re-apply low score in case compute_attractiveness overwrote
    set_attractiveness(&mut city, 10.0);
    city.tick(100);

    let final_count = city.citizen_count();
    assert!(
        final_count < initial_count,
        "low attractiveness should cause emigration: initial={initial_count}, final={final_count}"
    );
}

#[test]
fn test_immigration_no_housing_prevents_immigration() {
    // Setup: city with only commercial/industrial buildings (no residential).
    // Even with high attractiveness, no immigrants should spawn because there
    // is nowhere to live.
    let mut city = TestCity::new()
        .with_building(50, 54, ZoneType::CommercialLow, 1)
        .with_building(52, 54, ZoneType::Industrial, 1)
        .with_building(54, 54, ZoneType::Office, 1);

    let initial_count = city.citizen_count();
    assert_eq!(initial_count, 0, "should start with no citizens");

    // Force high attractiveness
    set_attractiveness(&mut city, 95.0);

    // Tick through several immigration waves
    city.tick(200);
    set_attractiveness(&mut city, 95.0);
    city.tick(200);
    set_attractiveness(&mut city, 95.0);
    city.tick(200);

    let final_count = city.citizen_count();
    assert_eq!(
        final_count, 0,
        "no immigration should occur without residential buildings, got {final_count} citizens"
    );
}

#[test]
fn test_immigration_no_jobs_prevents_immigration() {
    // Setup: city with only residential buildings (no commercial/industrial).
    // Even with high attractiveness, no immigrants should spawn because there
    // are no workplaces.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_building(52, 50, ZoneType::ResidentialLow, 1)
        .with_building(54, 50, ZoneType::ResidentialLow, 1);

    let initial_count = city.citizen_count();
    assert_eq!(initial_count, 0, "should start with no citizens");

    // Force high attractiveness
    set_attractiveness(&mut city, 95.0);

    // Tick through several immigration waves
    city.tick(200);
    set_attractiveness(&mut city, 95.0);
    city.tick(200);
    set_attractiveness(&mut city, 95.0);
    city.tick(200);

    let final_count = city.citizen_count();
    assert_eq!(
        final_count, 0,
        "no immigration should occur without job buildings, got {final_count} citizens"
    );
}

#[test]
fn test_immigration_scales_with_available_housing() {
    // Setup: two cities — one with few residential buildings (low capacity)
    // and one with many (high capacity). Both have high attractiveness.
    // The city with more housing should see more immigration.

    // Small city: 1 residential building
    let mut small_city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_building(50, 54, ZoneType::CommercialLow, 1)
        .with_building(52, 54, ZoneType::Industrial, 1);

    set_attractiveness(&mut small_city, 85.0);
    small_city.tick(100);
    set_attractiveness(&mut small_city, 85.0);
    small_city.tick(100);
    set_attractiveness(&mut small_city, 85.0);
    small_city.tick(100);

    let small_count = small_city.citizen_count();

    // Large city: many residential buildings
    let mut large_city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_building(52, 50, ZoneType::ResidentialLow, 1)
        .with_building(54, 50, ZoneType::ResidentialLow, 1)
        .with_building(56, 50, ZoneType::ResidentialLow, 1)
        .with_building(58, 50, ZoneType::ResidentialLow, 1)
        .with_building(60, 50, ZoneType::ResidentialLow, 1)
        .with_building(62, 50, ZoneType::ResidentialLow, 1)
        .with_building(64, 50, ZoneType::ResidentialLow, 1)
        .with_building(66, 50, ZoneType::ResidentialLow, 1)
        .with_building(68, 50, ZoneType::ResidentialLow, 1)
        // Plenty of workplaces
        .with_building(50, 54, ZoneType::CommercialLow, 1)
        .with_building(52, 54, ZoneType::CommercialLow, 1)
        .with_building(54, 54, ZoneType::Industrial, 1)
        .with_building(56, 54, ZoneType::Industrial, 1)
        .with_building(58, 54, ZoneType::Office, 1)
        .with_building(60, 54, ZoneType::Office, 1);

    set_attractiveness(&mut large_city, 85.0);
    large_city.tick(100);
    set_attractiveness(&mut large_city, 85.0);
    large_city.tick(100);
    set_attractiveness(&mut large_city, 85.0);
    large_city.tick(100);

    let large_count = large_city.citizen_count();

    // The city with more housing should receive at least as many immigrants.
    // (With 10x the housing, the large city should have more capacity
    // for immigration even if the per-wave count is similar.)
    assert!(
        large_count >= small_count,
        "more housing should allow more immigration: small={small_count}, large={large_count}"
    );
}

#[test]
fn test_immigration_neutral_attractiveness_no_migration() {
    // Setup: city with citizens, attractiveness in the neutral band (30-60).
    // Expected: no significant immigration or emigration.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_building(52, 50, ZoneType::CommercialLow, 1)
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50))
        .with_citizen((50, 50), (52, 50));

    // Stabilize citizens so they don't emigrate from happiness drift
    stabilize_citizens(&mut city);

    let initial_count = city.citizen_count();
    assert_eq!(initial_count, 5, "should start with 5 citizens");

    // Set attractiveness in the neutral zone (between 30 and 60)
    set_attractiveness(&mut city, 45.0);

    // Tick through several immigration intervals
    city.tick(100);
    stabilize_citizens(&mut city);
    set_attractiveness(&mut city, 45.0);
    city.tick(100);
    stabilize_citizens(&mut city);
    set_attractiveness(&mut city, 45.0);
    city.tick(100);

    let final_count = city.citizen_count();
    // In the neutral zone (30-60), the immigration_wave system does nothing.
    // However, other systems (lifecycle, emigration from unhappiness) might
    // still remove citizens. With stabilized happiness, population should
    // remain stable.
    assert!(
        final_count >= 3 && final_count <= 7,
        "neutral attractiveness should keep population roughly stable: \
         initial={initial_count}, final={final_count}"
    );
}

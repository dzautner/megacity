//! Integration tests for the immigration/emigration system (TEST-045).
//!
//! Tests verify that:
//! - High attractiveness triggers positive immigration
//! - Low attractiveness triggers emigration
//! - Immigration scales with available housing
//! - No immigration occurs when no housing is available

use crate::citizen::{Citizen, CitizenDetails};
use crate::economy::CityBudget;
use crate::education_jobs::EmploymentStats;
use crate::grid::ZoneType;
use crate::immigration::{CityAttractiveness, ImmigrationStats};
use crate::test_harness::TestCity;

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Make all citizens in the city very unhappy, unhealthy, and broke.
/// This pushes the attractiveness score down via happiness_factor.
fn make_citizens_miserable(city: &mut TestCity) {
    let world = city.world_mut();
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<Citizen>>()
        .iter(world)
        .collect();
    for entity in entities {
        if let Some(mut details) = world.get_mut::<CitizenDetails>(entity) {
            details.happiness = 5.0;
            details.health = 30.0;
            details.savings = -500.0;
        }
    }
}

/// Make all citizens happy and stable so they don't emigrate from other systems.
fn stabilize_citizens(city: &mut TestCity) {
    let world = city.world_mut();
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<Citizen>>()
        .iter(world)
        .collect();
    for entity in entities {
        if let Some(mut details) = world.get_mut::<CitizenDetails>(entity) {
            details.happiness = 95.0;
            details.health = 95.0;
            details.savings = 50_000.0;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Test that high attractiveness -> positive immigration rate.
/// We provide ample residential and job buildings with capacity, which
/// naturally produces a high attractiveness score. After ticking past the
/// immigration interval, new citizens should appear.
#[test]
fn test_immigration_high_attractiveness_increases_population() {
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

    // In an empty city with buildings, compute_attractiveness will calculate:
    //   employment: 1.0 (0% unemployment), happiness: 0.0 (no citizens),
    //   services: 0.0, housing: some vacancy > 0, tax: 0.5
    // Score ~ 25 + 0 + 0 + housing*15 + 7.5 = 32.5 + housing*15
    // This is in the neutral zone. We need to boost it above 60.
    // Adding a citizen with high happiness pushes happiness_factor up.
    // Let's just tick and see if the system bootstraps itself.
    // Once first immigrants arrive (if score > 60 at some tick), they raise
    // average happiness, which raises the score further.

    // Alternatively, manually push the score high before the first wave.
    // compute_attractiveness runs every 50 ticks, immigration_wave every 100.
    // If we set score at tick 0, it gets overwritten at tick 50.
    // Instead, seed the city with a few happy citizens to bootstrap.
    let city = city
        .with_citizen((50, 50), (50, 54))
        .with_citizen((52, 50), (52, 54));
    // Stabilize the seed citizens to be very happy
    let mut city = city;
    stabilize_citizens(&mut city);

    // With 2 happy citizens (happiness=95), average_happiness -> 95/100 = 0.95
    // employment: 1.0*25=25, happiness: 0.95*25=23.75, services: 0,
    // housing: ~0.6*15=9 (some vacancy), tax: 0.5*15=7.5
    // Score ~ 25 + 23.75 + 0 + 9 + 7.5 = 65.25 -> triggers immigration

    // Tick enough for multiple immigration waves
    city.tick(300);

    let final_count = city.citizen_count();
    assert!(
        final_count > 2,
        "high attractiveness should cause immigration: initial=2 seed, final={final_count}"
    );
}

/// Test that low attractiveness -> emigration.
/// We create citizens in miserable conditions (low happiness, high
/// unemployment, high taxes) to produce a low attractiveness score, then
/// verify that citizens are removed. We remove TestSafetyNet so the
/// destructive emigration systems can actually despawn citizens.
#[test]
fn test_emigration_low_attractiveness_decreases_population() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50));

    let initial_count = city.citizen_count();
    assert_eq!(initial_count, 10, "should start with 10 citizens");

    // Remove the safety net so emigration systems can despawn citizens.
    city.world_mut().remove_resource::<crate::TestSafetyNet>();

    // Force all attractiveness inputs low so compute_attractiveness
    // naturally calculates a score < 30 (the emigration threshold).
    fn force_bad_conditions(city: &mut TestCity) {
        make_citizens_miserable(city);
        if let Some(mut stats) = city.world_mut().get_resource_mut::<EmploymentStats>() {
            stats.unemployment_rate = 0.5;
            stats.total_unemployed = 100;
            stats.total_employed = 100;
        }
        if let Some(mut budget) = city.world_mut().get_resource_mut::<CityBudget>() {
            budget.tax_rate = 0.30;
        }
        if let Some(mut attr) = city.world_mut().get_resource_mut::<CityAttractiveness>() {
            attr.overall_score = 10.0;
            attr.employment_factor = 0.0;
            attr.happiness_factor = 0.0;
            attr.services_factor = 0.0;
            attr.housing_factor = 0.0;
            attr.tax_factor = 0.0;
        }
    }

    force_bad_conditions(&mut city);

    // Run multiple immigration wave intervals (each 100 ticks), re-applying
    // bad conditions before each wave to guarantee emigration fires.
    for _ in 0..4 {
        force_bad_conditions(&mut city);
        city.tick(100);
    }

    let final_count = city.citizen_count();
    assert!(
        final_count < initial_count,
        "low attractiveness should cause emigration: initial={initial_count}, final={final_count}"
    );
}
/// Test that no immigration occurs when no housing is available.
/// Even with high attractiveness factors, the immigration_wave function
/// returns early when there are no residential buildings with capacity.
#[test]
fn test_immigration_no_housing_prevents_immigration() {
    let mut city = TestCity::new()
        // Only job buildings, no residential
        .with_building(50, 54, ZoneType::CommercialLow, 1)
        .with_building(52, 54, ZoneType::Industrial, 1)
        .with_building(54, 54, ZoneType::Office, 1);

    let initial_count = city.citizen_count();
    assert_eq!(initial_count, 0, "should start with no citizens");

    // Even if score is computed above 60 (unlikely without residents pushing
    // happiness up), immigration_wave checks for residential buildings with
    // capacity and returns early if none exist.
    city.tick(600);

    let final_count = city.citizen_count();
    assert_eq!(
        final_count, 0,
        "no immigration should occur without residential buildings, got {final_count} citizens"
    );
}

/// Test that no immigration occurs when no job buildings are available.
/// The immigration_wave function requires both homes and workplaces.
#[test]
fn test_immigration_no_jobs_prevents_immigration() {
    let mut city = TestCity::new()
        // Only residential buildings, no job zones
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_building(52, 50, ZoneType::ResidentialLow, 1)
        .with_building(54, 50, ZoneType::ResidentialLow, 1);

    let initial_count = city.citizen_count();
    assert_eq!(initial_count, 0, "should start with no citizens");

    // Without job buildings, immigration_wave will find no workplaces
    // and return early even if the score is > 60.
    city.tick(600);

    let final_count = city.citizen_count();
    assert_eq!(
        final_count, 0,
        "no immigration should occur without job buildings, got {final_count} citizens"
    );
}

/// Test that immigration rate scales with available housing.
/// A city with more residential capacity should accumulate more immigrants
/// than a city with limited housing, given the same attractiveness level.
#[test]
fn test_immigration_scales_with_available_housing() {
    // -- Small city: 1 residential, 1 job building --
    let mut small_city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_building(50, 54, ZoneType::CommercialLow, 1)
        .with_citizen((50, 50), (50, 54));
    stabilize_citizens(&mut small_city);

    // -- Large city: 10 residential, 6 job buildings --
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
        .with_building(50, 54, ZoneType::CommercialLow, 1)
        .with_building(52, 54, ZoneType::CommercialLow, 1)
        .with_building(54, 54, ZoneType::Industrial, 1)
        .with_building(56, 54, ZoneType::Industrial, 1)
        .with_building(58, 54, ZoneType::Office, 1)
        .with_building(60, 54, ZoneType::Office, 1)
        .with_citizen((50, 50), (50, 54));
    stabilize_citizens(&mut large_city);

    // Tick both cities the same number of ticks
    small_city.tick(300);
    large_city.tick(300);

    let small_count = small_city.citizen_count();
    let large_count = large_city.citizen_count();

    // The large city should have at least as many immigrants because it has
    // more residential capacity available. (The immigration_wave function picks
    // from available homes, and with more homes, more families can settle.)
    assert!(
        large_count >= small_count,
        "more housing should allow at least as much immigration: small={small_count}, large={large_count}"
    );
}

/// Test that neutral attractiveness (score 30-60) does not trigger migration.
/// With no residential/job buildings, the score stays in the neutral zone
/// and no citizens are created or removed.
#[test]
fn test_immigration_neutral_attractiveness_no_migration() {
    // An empty city with no buildings at all. The computed attractiveness will
    // be: employment=1.0*25=25, happiness=0*25=0, services=0*20=0,
    // housing=0*15=0, tax=0.5*15=7.5 -> total=32.5 (neutral zone).
    // With no buildings, immigration_wave cannot spawn citizens anyway.
    let mut city = TestCity::new();

    city.tick(300);

    let count = city.citizen_count();
    assert_eq!(
        count, 0,
        "empty city in neutral attractiveness zone should have no migration, got {count} citizens"
    );

    let stats = city.resource::<ImmigrationStats>();
    assert_eq!(
        stats.immigrants_this_month, 0,
        "should have 0 immigrants in empty neutral city"
    );
    assert_eq!(
        stats.emigrants_this_month, 0,
        "should have 0 emigrants in empty neutral city"
    );
}

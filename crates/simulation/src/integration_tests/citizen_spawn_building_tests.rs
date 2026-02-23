//! Integration tests for TEST-015: Citizens Spawn in Completed Buildings.
//!
//! Verifies that when completed residential and commercial buildings exist
//! with sufficient demand and attractiveness, citizens are spawned via the
//! immigration system and have valid `HomeLocation` components.

use bevy::prelude::*;

use crate::buildings::{Building, UnderConstruction};
use crate::citizen::{Citizen, CitizenDetails, HomeLocation, WorkLocation};
use crate::grid::{RoadType, ZoneType};
use crate::immigration::CityAttractiveness;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::zones::ZoneDemand;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a standard city corridor with road, power, water, and both
/// residential and commercial zones side by side.
fn city_with_residential_and_commercial() -> TestCity {
    TestCity::new()
        .with_road(90, 100, 120, 100, RoadType::Local)
        .with_utility(90, 100, UtilityType::PowerPlant)
        .with_utility(91, 100, UtilityType::WaterTower)
        // Residential strip adjacent to road
        .with_zone_rect(93, 99, 105, 99, ZoneType::ResidentialLow)
        // Commercial strip adjacent to road on the other side
        .with_zone_rect(93, 101, 105, 101, ZoneType::CommercialLow)
}

/// Set high demand for all zone types so the building spawner activates.
fn set_high_demand(city: &mut TestCity) {
    let world = city.world_mut();
    let mut demand = world.resource_mut::<ZoneDemand>();
    demand.residential = 1.0;
    demand.commercial = 1.0;
    demand.industrial = 1.0;
    demand.office = 1.0;
}

/// Force city attractiveness above the immigration threshold (>60).
fn force_high_attractiveness(city: &mut TestCity) {
    let world = city.world_mut();
    let mut attr = world.resource_mut::<CityAttractiveness>();
    attr.overall_score = 85.0;
    attr.employment_factor = 1.0;
    attr.happiness_factor = 0.9;
    attr.services_factor = 0.5;
    attr.housing_factor = 0.8;
    attr.tax_factor = 0.5;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Test that citizens spawn in completed residential buildings after sufficient
/// simulation ticks, and that each citizen has a valid HomeLocation.
#[test]
fn test_citizens_spawn_in_completed_buildings_with_home_location() {
    let mut city = city_with_residential_and_commercial();
    set_high_demand(&mut city);

    // Phase 1: Let buildings spawn and construction complete (~150 ticks).
    // Building spawner triggers within a few ticks; construction takes ~100.
    city.tick(150);

    // Verify we have completed buildings of both types.
    let world = city.world_mut();
    let completed_residential: Vec<Entity> = world
        .query_filtered::<(Entity, &Building), Without<UnderConstruction>>()
        .iter(world)
        .filter(|(_, b)| b.zone_type == ZoneType::ResidentialLow && b.capacity > 0)
        .map(|(e, _)| e)
        .collect();
    let completed_commercial: Vec<Entity> = world
        .query_filtered::<(Entity, &Building), Without<UnderConstruction>>()
        .iter(world)
        .filter(|(_, b)| b.zone_type == ZoneType::CommercialLow && b.capacity > 0)
        .map(|(e, _)| e)
        .collect();

    assert!(
        !completed_residential.is_empty(),
        "Should have at least one completed ResidentialLow building with capacity > 0"
    );
    assert!(
        !completed_commercial.is_empty(),
        "Should have at least one completed CommercialLow building nearby"
    );

    // Phase 2: Force high attractiveness and run immigration waves.
    // Immigration fires every IMMIGRATION_INTERVAL ticks (100) when score > 60.
    // Re-apply high attractiveness before each wave since compute_attractiveness
    // may recalculate it.
    for _ in 0..5 {
        set_high_demand(&mut city);
        force_high_attractiveness(&mut city);
        city.tick(100);
    }

    // Verify citizens spawned.
    let citizen_count = city.citizen_count();
    assert!(
        citizen_count > 0,
        "After 50 ticks post-construction with high attractiveness, \
         citizen_count should be > 0, got {citizen_count}"
    );

    // Verify every citizen has a valid HomeLocation pointing to a real building.
    let world = city.world_mut();
    let citizens_with_home: Vec<(Entity, HomeLocation)> = world
        .query_filtered::<(Entity, &HomeLocation), With<Citizen>>()
        .iter(world)
        .map(|(e, h)| (e, h.clone()))
        .collect();

    assert!(
        !citizens_with_home.is_empty(),
        "Citizens should have HomeLocation components"
    );

    for (citizen_entity, home) in &citizens_with_home {
        // HomeLocation.building should reference an existing Building entity.
        let building = world.get::<Building>(home.building);
        assert!(
            building.is_some(),
            "Citizen {citizen_entity:?} HomeLocation.building {:?} should reference a valid Building",
            home.building
        );
        let building = building.unwrap();
        assert!(
            building.zone_type.is_residential() || building.zone_type.is_mixed_use(),
            "Home building should be residential or mixed-use, got {:?}",
            building.zone_type
        );
    }
}

/// Test that spawned citizens also receive a WorkLocation pointing to a
/// valid commercial/industrial/office building.
#[test]
fn test_citizens_spawn_with_valid_work_location() {
    let mut city = city_with_residential_and_commercial();
    set_high_demand(&mut city);

    // Let buildings spawn and complete construction.
    city.tick(150);

    // Drive immigration.
    for _ in 0..5 {
        set_high_demand(&mut city);
        force_high_attractiveness(&mut city);
        city.tick(100);
    }

    let citizen_count = city.citizen_count();
    assert!(
        citizen_count > 0,
        "Should have citizens after immigration waves, got {citizen_count}"
    );

    let world = city.world_mut();
    let citizens_with_work: Vec<(Entity, WorkLocation)> = world
        .query_filtered::<(Entity, &WorkLocation), With<Citizen>>()
        .iter(world)
        .map(|(e, w)| (e, w.clone()))
        .collect();

    assert!(
        !citizens_with_work.is_empty(),
        "Citizens should have WorkLocation components"
    );

    for (citizen_entity, work) in &citizens_with_work {
        let building = world.get::<Building>(work.building);
        assert!(
            building.is_some(),
            "Citizen {citizen_entity:?} WorkLocation.building {:?} should reference a valid Building",
            work.building
        );
        let building = building.unwrap();
        assert!(
            building.zone_type.is_job_zone(),
            "Work building should be a job zone, got {:?}",
            building.zone_type
        );
    }
}

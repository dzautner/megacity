//! Integration tests for SAVE-004: Serialize Citizen Health (Issue #699).
//!
//! Verifies that the `health` field on `CitizenDetails` roundtrips correctly
//! through serde serialization (the same path used by the save system).
//! Also verifies that the `with_citizen` TestCity builder spawns citizens
//! with a known health value.

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::movement::ActivityTimer;
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Test: Health roundtrips through CitizenDetails serde
// ---------------------------------------------------------------------------

/// A citizen with a non-default health value (e.g. 20.0, representing a sick
/// citizen) must retain that value after serialize -> deserialize, rather than
/// being overwritten with the default 80.0.
#[test]
fn test_citizen_health_roundtrip_preserves_low_health() {
    let details = CitizenDetails {
        age: 55,
        gender: Gender::Male,
        education: 1,
        happiness: 40.0,
        health: 20.0,
        salary: 2200.0,
        savings: 500.0,
    };

    let json = serde_json::to_string(&details).unwrap();
    let restored: CitizenDetails = serde_json::from_str(&json).unwrap();

    assert!(
        (restored.health - 20.0).abs() < f32::EPSILON,
        "Expected health 20.0 after roundtrip, got {}",
        restored.health
    );
}

// ---------------------------------------------------------------------------
// Test: Health roundtrips at various values
// ---------------------------------------------------------------------------

/// Verify that health roundtrips correctly at boundary and typical values:
/// 0.0, 1.0, 50.0, 80.0 (default), 99.0, 100.0.
#[test]
fn test_citizen_health_roundtrip_various_values() {
    let test_values = [0.0_f32, 1.0, 20.0, 50.0, 80.0, 99.0, 100.0];

    for &health_val in &test_values {
        let details = CitizenDetails {
            age: 30,
            gender: Gender::Female,
            education: 2,
            happiness: 60.0,
            health: health_val,
            salary: 3500.0,
            savings: 7000.0,
        };

        let json = serde_json::to_string(&details).unwrap();
        let restored: CitizenDetails = serde_json::from_str(&json).unwrap();

        assert!(
            (restored.health - health_val).abs() < f32::EPSILON,
            "Health roundtrip failed for value {}: got {}",
            health_val,
            restored.health
        );
    }
}

// ---------------------------------------------------------------------------
// Test: Health survives ECS -> serde -> ECS roundtrip in TestCity
// ---------------------------------------------------------------------------

/// Spawns a citizen with health=25.0 (critically ill) in a TestCity, reads it
/// back through ECS queries, serializes via serde, and verifies the value is
/// preserved exactly.
#[test]
fn test_citizen_health_ecs_serde_roundtrip_critically_ill() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    // Spawn a citizen with critically low health.
    let world = city.world_mut();
    let home_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(12, 11).building_id.unwrap()
    };
    let work_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(18, 11).building_id.unwrap()
    };

    world.spawn((
        Citizen,
        Position { x: 200.0, y: 180.0 },
        Velocity { x: 0.0, y: 0.0 },
        HomeLocation {
            grid_x: 12,
            grid_y: 11,
            building: home_entity,
        },
        WorkLocation {
            grid_x: 18,
            grid_y: 11,
            building: work_entity,
        },
        CitizenStateComp(CitizenState::AtHome),
        PathCache::new(Vec::new()),
        CitizenDetails {
            age: 70,
            gender: Gender::Male,
            education: 0,
            happiness: 30.0,
            health: 25.0,
            salary: 1500.0,
            savings: 200.0,
        },
        Personality::default(),
        Needs::default(),
        Family::default(),
        ActivityTimer::default(),
    ));

    // Read back from ECS and serialize.
    let world = city.world_mut();
    let mut query = world.query::<&CitizenDetails>();
    let details = query.iter(world).next().expect("should have 1 citizen");

    assert!(
        (details.health - 25.0).abs() < f32::EPSILON,
        "Health in ECS should be 25.0, got {}",
        details.health
    );

    // Roundtrip through serde (same path as save system).
    let json = serde_json::to_string(details).unwrap();
    let restored: CitizenDetails = serde_json::from_str(&json).unwrap();

    assert!(
        (restored.health - 25.0).abs() < f32::EPSILON,
        "Health after serde roundtrip should be 25.0, got {}",
        restored.health
    );
}

// ---------------------------------------------------------------------------
// Test: Multiple citizens with different health values
// ---------------------------------------------------------------------------

/// Spawns three citizens with different health values and verifies each value
/// is preserved through serde roundtrip.
#[test]
fn test_citizen_health_multiple_citizens_preserve_individual_values() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_building(24, 11, ZoneType::Industrial, 1);

    let world = city.world_mut();
    let home_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(12, 11).building_id.unwrap()
    };
    let work1_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(18, 11).building_id.unwrap()
    };
    let work2_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(24, 11).building_id.unwrap()
    };

    let health_values = [15.0_f32, 55.5, 98.0];

    for (i, &health_val) in health_values.iter().enumerate() {
        let work_entity = if i < 2 { work1_entity } else { work2_entity };
        let (hx, hy) = WorldGrid::grid_to_world(12, 11);
        world.spawn((
            Citizen,
            Position { x: hx, y: hy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: 12,
                grid_y: 11,
                building: home_entity,
            },
            WorkLocation {
                grid_x: if i < 2 { 18 } else { 24 },
                grid_y: 11,
                building: work_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30 + (i as u8) * 10,
                gender: Gender::Male,
                education: 2,
                happiness: 60.0,
                health: health_val,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality::default(),
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
        ));
    }

    // Read all citizens back and verify health values.
    let world = city.world_mut();
    let mut query = world.query::<&CitizenDetails>();
    let mut found_healths: Vec<f32> = query.iter(world).map(|d| d.health).collect();
    found_healths.sort_by(|a, b| a.partial_cmp(b).unwrap());

    assert_eq!(found_healths.len(), 3);
    assert!(
        (found_healths[0] - 15.0).abs() < f32::EPSILON,
        "First citizen health should be 15.0, got {}",
        found_healths[0]
    );
    assert!(
        (found_healths[1] - 55.5).abs() < f32::EPSILON,
        "Second citizen health should be 55.5, got {}",
        found_healths[1]
    );
    assert!(
        (found_healths[2] - 98.0).abs() < f32::EPSILON,
        "Third citizen health should be 98.0, got {}",
        found_healths[2]
    );

    // Roundtrip each through serde.
    for details in query.iter(world) {
        let json = serde_json::to_string(details).unwrap();
        let restored: CitizenDetails = serde_json::from_str(&json).unwrap();
        assert!(
            (restored.health - details.health).abs() < f32::EPSILON,
            "Health mismatch after roundtrip: expected {}, got {}",
            details.health,
            restored.health
        );
    }
}

// ---------------------------------------------------------------------------
// Test: with_citizen builder uses non-default health
// ---------------------------------------------------------------------------

/// Verify that the TestCity `with_citizen` builder spawns citizens with
/// health=90.0 (not the save-default 80.0), confirming the field is
/// explicitly set rather than relying on defaults.
#[test]
fn test_with_citizen_builder_sets_health_explicitly() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_citizen((12, 11), (18, 11));

    let world = city.world_mut();
    let mut query = world.query::<&CitizenDetails>();
    let details = query.iter(world).next().expect("should have 1 citizen");

    // with_citizen sets health to 90.0
    assert!(
        (details.health - 90.0).abs() < f32::EPSILON,
        "with_citizen should set health=90.0, got {}",
        details.health
    );
}

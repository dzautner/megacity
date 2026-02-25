//! SAVE-016: Citizen Needs Save/Load Round-Trip Tests (Issue #712)
//!
//! Verifies that citizen Needs (hunger, energy, social, fun, comfort) survive
//! a full serde roundtrip (the same path used by the save system). Also
//! verifies backward compatibility: old saves without needs fields default to
//! the Needs::default() values (80, 80, 70, 70, 60).

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::movement::ActivityTimer;
use crate::test_harness::TestCity;

/// Helper: assert all five need fields match expected values.
fn assert_needs_eq(actual: &Needs, expected: &Needs, ctx: &str) {
    assert!(
        (actual.hunger - expected.hunger).abs() < f32::EPSILON,
        "{ctx}: hunger expected {}, got {}",
        expected.hunger,
        actual.hunger
    );
    assert!(
        (actual.energy - expected.energy).abs() < f32::EPSILON,
        "{ctx}: energy expected {}, got {}",
        expected.energy,
        actual.energy
    );
    assert!(
        (actual.social - expected.social).abs() < f32::EPSILON,
        "{ctx}: social expected {}, got {}",
        expected.social,
        actual.social
    );
    assert!(
        (actual.fun - expected.fun).abs() < f32::EPSILON,
        "{ctx}: fun expected {}, got {}",
        expected.fun,
        actual.fun
    );
    assert!(
        (actual.comfort - expected.comfort).abs() < f32::EPSILON,
        "{ctx}: comfort expected {}, got {}",
        expected.comfort,
        actual.comfort
    );
}

/// Helper: serde roundtrip a Needs through JSON and assert equality.
fn roundtrip_needs(original: &Needs, ctx: &str) {
    let json = serde_json::to_string(original).unwrap();
    let restored: Needs = serde_json::from_str(&json).unwrap();
    assert_needs_eq(&restored, original, ctx);
}

// ---------------------------------------------------------------------------
// Test: Needs serde roundtrip preserves non-default values
// ---------------------------------------------------------------------------

#[test]
fn test_needs_serde_roundtrip_preserves_non_default_values() {
    let original = Needs {
        hunger: 15.0,
        energy: 30.0,
        social: 95.0,
        fun: 42.0,
        comfort: 88.0,
    };
    roundtrip_needs(&original, "non-default");
}

// ---------------------------------------------------------------------------
// Test: Needs roundtrip at boundary values
// ---------------------------------------------------------------------------

#[test]
fn test_needs_serde_roundtrip_boundary_values() {
    let test_cases = [
        Needs { hunger: 0.0, energy: 0.0, social: 0.0, fun: 0.0, comfort: 0.0 },
        Needs { hunger: 100.0, energy: 100.0, social: 100.0, fun: 100.0, comfort: 100.0 },
        Needs { hunger: 80.0, energy: 80.0, social: 70.0, fun: 70.0, comfort: 60.0 },
        Needs { hunger: 10.5, energy: 25.3, social: 55.7, fun: 82.1, comfort: 99.9 },
    ];

    for (i, needs) in test_cases.iter().enumerate() {
        roundtrip_needs(needs, &format!("boundary case {i}"));
    }
}

// ---------------------------------------------------------------------------
// Test: Backward compatibility — missing needs fields use Needs::default()
// ---------------------------------------------------------------------------

#[test]
fn test_needs_missing_fields_default_correctly() {
    #[derive(serde::Deserialize)]
    struct MiniSaveNeeds {
        #[serde(default = "default_hunger")]
        need_hunger: f32,
        #[serde(default = "default_energy")]
        need_energy: f32,
        #[serde(default = "default_social")]
        need_social: f32,
        #[serde(default = "default_fun")]
        need_fun: f32,
        #[serde(default = "default_comfort")]
        need_comfort: f32,
    }
    fn default_hunger() -> f32 { 80.0 }
    fn default_energy() -> f32 { 80.0 }
    fn default_social() -> f32 { 70.0 }
    fn default_fun() -> f32 { 70.0 }
    fn default_comfort() -> f32 { 60.0 }

    let restored: MiniSaveNeeds = serde_json::from_str(r#"{}"#).unwrap();
    let defaults = Needs::default();

    assert!((restored.need_hunger - defaults.hunger).abs() < f32::EPSILON);
    assert!((restored.need_energy - defaults.energy).abs() < f32::EPSILON);
    assert!((restored.need_social - defaults.social).abs() < f32::EPSILON);
    assert!((restored.need_fun - defaults.fun).abs() < f32::EPSILON);
    assert!((restored.need_comfort - defaults.comfort).abs() < f32::EPSILON);
}

// ---------------------------------------------------------------------------
// Test: Needs survive ECS -> serde -> ECS in TestCity
// ---------------------------------------------------------------------------

#[test]
fn test_needs_ecs_serde_roundtrip_non_default() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    let world = city.world_mut();
    let home_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(12, 11).building_id.unwrap()
    };
    let work_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(18, 11).building_id.unwrap()
    };

    let expected = Needs {
        hunger: 22.0,
        energy: 45.0,
        social: 88.0,
        fun: 12.0,
        comfort: 67.0,
    };

    world.spawn((
        Citizen,
        Position { x: 200.0, y: 180.0 },
        Velocity { x: 0.0, y: 0.0 },
        HomeLocation { grid_x: 12, grid_y: 11, building: home_entity },
        WorkLocation { grid_x: 18, grid_y: 11, building: work_entity },
        CitizenStateComp(CitizenState::AtHome),
        PathCache::new(Vec::new()),
        CitizenDetails {
            age: 35, gender: Gender::Female, education: 3,
            happiness: 70.0, health: 85.0, salary: 6000.0, savings: 12000.0,
        },
        Personality { ambition: 0.5, sociability: 0.5, materialism: 0.5, resilience: 0.5 },
        expected.clone(),
        Family::default(),
        ActivityTimer::default(),
    ));

    let world = city.world_mut();
    let mut query = world.query::<&Needs>();
    let needs = query.iter(world).next().expect("should have 1 citizen");
    assert_needs_eq(needs, &expected, "ECS read-back");
    roundtrip_needs(needs, "ECS serde roundtrip");
}

// ---------------------------------------------------------------------------
// Test: Multiple citizens with different needs all preserved
// ---------------------------------------------------------------------------

#[test]
fn test_needs_multiple_citizens_preserve_individual_values() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    let world = city.world_mut();
    let home_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(12, 11).building_id.unwrap()
    };
    let work_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(18, 11).building_id.unwrap()
    };

    let profiles = [
        Needs { hunger: 10.0, energy: 20.0, social: 30.0, fun: 40.0, comfort: 50.0 },
        Needs { hunger: 90.0, energy: 80.0, social: 70.0, fun: 60.0, comfort: 55.0 },
        Needs { hunger: 0.0, energy: 100.0, social: 50.0, fun: 0.0, comfort: 100.0 },
    ];

    for (i, needs) in profiles.iter().enumerate() {
        let (hx, hy) = WorldGrid::grid_to_world(12, 11);
        world.spawn((
            Citizen,
            Position { x: hx, y: hy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation { grid_x: 12, grid_y: 11, building: home_entity },
            WorkLocation { grid_x: 18, grid_y: 11, building: work_entity },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 25 + (i as u8) * 10, gender: Gender::Male, education: 2,
                happiness: 60.0, health: 85.0, salary: 3500.0, savings: 7000.0,
            },
            Personality { ambition: 0.5, sociability: 0.5, materialism: 0.5, resilience: 0.5 },
            needs.clone(),
            Family::default(),
            ActivityTimer::default(),
        ));
    }

    let world = city.world_mut();
    let mut query = world.query::<&Needs>();
    let all_needs: Vec<Needs> = query.iter(world).cloned().collect();
    assert_eq!(all_needs.len(), 3);

    for (i, needs) in all_needs.iter().enumerate() {
        roundtrip_needs(needs, &format!("citizen {i}"));
    }
}

// ---------------------------------------------------------------------------
// Test: Needs survive repeated save/load cycles (drift check)
// ---------------------------------------------------------------------------

#[test]
fn test_needs_no_drift_across_10_serde_cycles() {
    let original = Needs {
        hunger: 33.3,
        energy: 66.6,
        social: 11.1,
        fun: 77.7,
        comfort: 44.4,
    };

    let mut json = serde_json::to_string(&original).unwrap();
    for cycle in 0..10 {
        let n: Needs = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("cycle {cycle}: deserialize failed: {e}"));
        json = serde_json::to_string(&n).unwrap();
    }

    let final_n: Needs = serde_json::from_str(&json).unwrap();
    assert_needs_eq(&final_n, &original, "after 10 cycles");
}

// ---------------------------------------------------------------------------
// Test: with_citizen builder spawns citizens with default Needs
// ---------------------------------------------------------------------------

#[test]
fn test_with_citizen_builder_sets_default_needs() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_citizen((12, 11), (18, 11));

    let world = city.world_mut();
    let mut query = world.query::<&Needs>();
    let needs = query.iter(world).next().expect("should have 1 citizen");
    assert_needs_eq(needs, &Needs::default(), "with_citizen builder");
}

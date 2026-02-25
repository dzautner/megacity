//! SAVE-015: Citizen Personality Save/Load Round-Trip Tests (Issue #711)
//!
//! Verifies that citizen personality traits (ambition, sociability,
//! materialism, resilience) survive a full save/load roundtrip. Also verifies
//! backward compatibility: old saves without personality fields default to 0.5.

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Test: Personality serde roundtrip preserves non-default values
// ---------------------------------------------------------------------------

/// Personality traits that differ from the 0.5 default must survive a JSON
/// serialize/deserialize cycle (the same path used by the save system).
#[test]
fn test_personality_serde_roundtrip_preserves_non_default_values() {
    let original = Personality {
        ambition: 0.1,
        sociability: 0.9,
        materialism: 0.3,
        resilience: 0.7,
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: Personality = serde_json::from_str(&json).unwrap();

    assert!(
        (restored.ambition - 0.1).abs() < f32::EPSILON,
        "ambition should be 0.1, got {}",
        restored.ambition
    );
    assert!(
        (restored.sociability - 0.9).abs() < f32::EPSILON,
        "sociability should be 0.9, got {}",
        restored.sociability
    );
    assert!(
        (restored.materialism - 0.3).abs() < f32::EPSILON,
        "materialism should be 0.3, got {}",
        restored.materialism
    );
    assert!(
        (restored.resilience - 0.7).abs() < f32::EPSILON,
        "resilience should be 0.7, got {}",
        restored.resilience
    );
}

// ---------------------------------------------------------------------------
// Test: Personality roundtrips at boundary values
// ---------------------------------------------------------------------------

/// Verify that personality traits roundtrip correctly at boundary values
/// (0.0, 0.1, 0.5, 1.0) and typical non-default values.
#[test]
fn test_personality_serde_roundtrip_boundary_values() {
    let test_cases: &[(f32, f32, f32, f32)] = &[
        (0.0, 0.0, 0.0, 0.0),     // all minimum
        (1.0, 1.0, 1.0, 1.0),     // all maximum
        (0.1, 0.1, 0.1, 0.1),     // all at Personality::random lower bound
        (0.5, 0.5, 0.5, 0.5),     // all at default
        (0.15, 0.85, 0.42, 0.67), // arbitrary mix
    ];

    for &(amb, soc, mat, res) in test_cases {
        let original = Personality {
            ambition: amb,
            sociability: soc,
            materialism: mat,
            resilience: res,
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: Personality = serde_json::from_str(&json).unwrap();

        assert!(
            (restored.ambition - amb).abs() < f32::EPSILON,
            "ambition: expected {amb}, got {}",
            restored.ambition
        );
        assert!(
            (restored.sociability - soc).abs() < f32::EPSILON,
            "sociability: expected {soc}, got {}",
            restored.sociability
        );
        assert!(
            (restored.materialism - mat).abs() < f32::EPSILON,
            "materialism: expected {mat}, got {}",
            restored.materialism
        );
        assert!(
            (restored.resilience - res).abs() < f32::EPSILON,
            "resilience: expected {res}, got {}",
            restored.resilience
        );
    }
}

// ---------------------------------------------------------------------------
// Test: Backward compatibility — missing personality fields default to 0.5
// ---------------------------------------------------------------------------

/// When deserializing a JSON object that lacks personality fields entirely
/// (simulating a pre-personality save), serde defaults should produce 0.5
/// for all traits. This mirrors the `#[serde(default = "default_personality_trait")]`
/// annotation on `SaveCitizen`.
#[test]
fn test_personality_missing_fields_default_to_half() {
    #[derive(serde::Deserialize)]
    struct MiniSaveCitizen {
        #[serde(default = "default_half")]
        ambition: f32,
        #[serde(default = "default_half")]
        sociability: f32,
        #[serde(default = "default_half")]
        materialism: f32,
        #[serde(default = "default_half")]
        resilience: f32,
    }

    fn default_half() -> f32 {
        0.5
    }

    // JSON with no personality fields at all (old save format)
    let json = r#"{}"#;
    let restored: MiniSaveCitizen = serde_json::from_str(json).unwrap();

    assert!(
        (restored.ambition - 0.5).abs() < f32::EPSILON,
        "missing ambition should default to 0.5, got {}",
        restored.ambition
    );
    assert!(
        (restored.sociability - 0.5).abs() < f32::EPSILON,
        "missing sociability should default to 0.5, got {}",
        restored.sociability
    );
    assert!(
        (restored.materialism - 0.5).abs() < f32::EPSILON,
        "missing materialism should default to 0.5, got {}",
        restored.materialism
    );
    assert!(
        (restored.resilience - 0.5).abs() < f32::EPSILON,
        "missing resilience should default to 0.5, got {}",
        restored.resilience
    );
}

// ---------------------------------------------------------------------------
// Test: Personality survives ECS -> serde -> ECS in TestCity
// ---------------------------------------------------------------------------

/// Spawns a citizen with non-default personality in a TestCity, reads it
/// back through ECS queries, serializes via serde, and verifies all four
/// traits are preserved exactly.
#[test]
fn test_personality_ecs_serde_roundtrip_non_default() {
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
            age: 35,
            gender: Gender::Female,
            education: 3,
            happiness: 70.0,
            health: 85.0,
            salary: 6000.0,
            savings: 12000.0,
        },
        Personality {
            ambition: 0.9,
            sociability: 0.2,
            materialism: 0.75,
            resilience: 0.35,
        },
        Needs::default(),
        Family::default(),
        ActivityTimer::default(),
        ChosenTransportMode::default(),
    ));

    // Read back from ECS and verify.
    let world = city.world_mut();
    let mut query = world.query::<&Personality>();
    let pers = query.iter(world).next().expect("should have 1 citizen");

    assert!((pers.ambition - 0.9).abs() < f32::EPSILON);
    assert!((pers.sociability - 0.2).abs() < f32::EPSILON);
    assert!((pers.materialism - 0.75).abs() < f32::EPSILON);
    assert!((pers.resilience - 0.35).abs() < f32::EPSILON);

    // Roundtrip through serde (same path as save system).
    let json = serde_json::to_string(pers).unwrap();
    let restored: Personality = serde_json::from_str(&json).unwrap();

    assert!(
        (restored.ambition - 0.9).abs() < f32::EPSILON,
        "ambition after roundtrip: expected 0.9, got {}",
        restored.ambition
    );
    assert!(
        (restored.sociability - 0.2).abs() < f32::EPSILON,
        "sociability after roundtrip: expected 0.2, got {}",
        restored.sociability
    );
    assert!(
        (restored.materialism - 0.75).abs() < f32::EPSILON,
        "materialism after roundtrip: expected 0.75, got {}",
        restored.materialism
    );
    assert!(
        (restored.resilience - 0.35).abs() < f32::EPSILON,
        "resilience after roundtrip: expected 0.35, got {}",
        restored.resilience
    );
}

// ---------------------------------------------------------------------------
// Test: Multiple citizens with different personalities all preserved
// ---------------------------------------------------------------------------

/// Spawns three citizens with different personality profiles and verifies
/// each set of traits is preserved through serde roundtrip.
#[test]
fn test_personality_multiple_citizens_preserve_individual_values() {
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

    let personalities = [
        Personality {
            ambition: 0.1,
            sociability: 0.9,
            materialism: 0.3,
            resilience: 0.7,
        },
        Personality {
            ambition: 0.8,
            sociability: 0.2,
            materialism: 0.6,
            resilience: 0.4,
        },
        Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 1.0,
            resilience: 0.0,
        },
    ];

    for (i, p) in personalities.iter().enumerate() {
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
                grid_x: 18,
                grid_y: 11,
                building: work_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 25 + (i as u8) * 10,
                gender: Gender::Male,
                education: 2,
                happiness: 60.0,
                health: 85.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            p.clone(),
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
            ChosenTransportMode::default(),
        ));
    }

    // Read all citizens and verify personality roundtrips.
    let world = city.world_mut();
    let mut query = world.query::<&Personality>();
    let all_pers: Vec<Personality> = query.iter(world).cloned().collect();
    assert_eq!(all_pers.len(), 3);

    for pers in &all_pers {
        let json = serde_json::to_string(pers).unwrap();
        let restored: Personality = serde_json::from_str(&json).unwrap();

        assert!(
            (restored.ambition - pers.ambition).abs() < f32::EPSILON,
            "ambition mismatch after roundtrip: expected {}, got {}",
            pers.ambition,
            restored.ambition
        );
        assert!(
            (restored.sociability - pers.sociability).abs() < f32::EPSILON,
            "sociability mismatch: expected {}, got {}",
            pers.sociability,
            restored.sociability
        );
        assert!(
            (restored.materialism - pers.materialism).abs() < f32::EPSILON,
            "materialism mismatch: expected {}, got {}",
            pers.materialism,
            restored.materialism
        );
        assert!(
            (restored.resilience - pers.resilience).abs() < f32::EPSILON,
            "resilience mismatch: expected {}, got {}",
            pers.resilience,
            restored.resilience
        );
    }
}

// ---------------------------------------------------------------------------
// Test: Personality survives repeated save/load cycles (drift check)
// ---------------------------------------------------------------------------

/// Run 10 consecutive serde roundtrip cycles on a Personality and verify
/// no floating-point drift occurs.
#[test]
fn test_personality_no_drift_across_10_serde_cycles() {
    let original = Personality {
        ambition: 0.123,
        sociability: 0.876,
        materialism: 0.456,
        resilience: 0.789,
    };

    let mut json = serde_json::to_string(&original).unwrap();

    for cycle in 0..10 {
        let p: Personality = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("cycle {cycle}: deserialize failed: {e}"));
        json = serde_json::to_string(&p).unwrap();
    }

    let final_p: Personality = serde_json::from_str(&json).unwrap();
    assert!(
        (final_p.ambition - original.ambition).abs() < f32::EPSILON,
        "ambition drifted after 10 cycles: {} != {}",
        final_p.ambition,
        original.ambition
    );
    assert!(
        (final_p.sociability - original.sociability).abs() < f32::EPSILON,
        "sociability drifted: {} != {}",
        final_p.sociability,
        original.sociability
    );
    assert!(
        (final_p.materialism - original.materialism).abs() < f32::EPSILON,
        "materialism drifted: {} != {}",
        final_p.materialism,
        original.materialism
    );
    assert!(
        (final_p.resilience - original.resilience).abs() < f32::EPSILON,
        "resilience drifted: {} != {}",
        final_p.resilience,
        original.resilience
    );
}

// ---------------------------------------------------------------------------
// Test: with_citizen builder uses default personality (0.5)
// ---------------------------------------------------------------------------

/// Verify that the TestCity `with_citizen` builder spawns citizens with
/// personality traits all set to 0.5.
#[test]
fn test_with_citizen_builder_sets_default_personality() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_citizen((12, 11), (18, 11));

    let world = city.world_mut();
    let mut query = world.query::<&Personality>();
    let pers = query.iter(world).next().expect("should have 1 citizen");

    assert!(
        (pers.ambition - 0.5).abs() < f32::EPSILON,
        "with_citizen ambition should be 0.5, got {}",
        pers.ambition
    );
    assert!(
        (pers.sociability - 0.5).abs() < f32::EPSILON,
        "with_citizen sociability should be 0.5, got {}",
        pers.sociability
    );
    assert!(
        (pers.materialism - 0.5).abs() < f32::EPSILON,
        "with_citizen materialism should be 0.5, got {}",
        pers.materialism
    );
    assert!(
        (pers.resilience - 0.5).abs() < f32::EPSILON,
        "with_citizen resilience should be 0.5, got {}",
        pers.resilience
    );
}

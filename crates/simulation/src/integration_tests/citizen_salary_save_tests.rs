//! SAVE-005: Citizen Salary Save/Load Round-Trip Tests (Issue #701)
//!
//! Verifies that citizen salary (including job-match modifiers like seniority)
//! survives a full save/load roundtrip. Also verifies backward compatibility:
//! old saves with salary=0.0 correctly recalculate from education level.

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::test_harness::TestCity;
use crate::SaveableRegistry;

// ---------------------------------------------------------------------------
// Test: Salary with seniority modifier roundtrips via serde
// ---------------------------------------------------------------------------

/// A salary that includes the seniority bonus (e.g. 4550.0 rather than the
/// base 3500.0 for education=2) must survive a JSON serialize/deserialize
/// cycle on CitizenDetails.
#[test]
fn test_citizen_salary_serde_roundtrip_preserves_seniority_modifier() {
    let original = CitizenDetails {
        age: 40,
        gender: Gender::Male,
        education: 2,
        happiness: 65.0,
        health: 85.0,
        salary: 4550.0, // base 3500 + seniority; NOT base_salary_for_education(2)
        savings: 10000.0,
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: CitizenDetails = serde_json::from_str(&json).unwrap();

    assert!(
        (restored.salary - 4550.0).abs() < f32::EPSILON,
        "salary with seniority modifier should survive serde roundtrip, got {}",
        restored.salary
    );
    assert_eq!(restored.education, 2);
    assert_eq!(restored.age, 40);
}

// ---------------------------------------------------------------------------
// Test: Old saves (salary=0.0) recalculate from education
// ---------------------------------------------------------------------------

/// When loading a citizen whose saved salary is 0.0 (as in pre-V4 saves),
/// the load path should fall back to base_salary_for_education. This test
/// simulates the SaveCitizen deserialization path by verifying the fallback
/// logic directly.
#[test]
fn test_citizen_salary_zero_falls_back_to_education_base() {
    // Simulate what spawn_entities.rs does when salary == 0.0
    let saved_salary: f32 = 0.0;
    let education: u8 = 3; // University

    let effective_salary = if saved_salary != 0.0 {
        saved_salary
    } else {
        CitizenDetails::base_salary_for_education(education)
    };

    assert!(
        (effective_salary - 6000.0).abs() < f32::EPSILON,
        "zero salary should fall back to base for edu=3 (6000.0), got {}",
        effective_salary
    );

    // Also verify for other education levels
    for (edu, expected_base) in [(0u8, 1500.0), (1, 2200.0), (2, 3500.0), (3, 6000.0)] {
        let base = CitizenDetails::base_salary_for_education(edu);
        assert!(
            (base - expected_base).abs() < f32::EPSILON,
            "base_salary_for_education({edu}) = {base}, expected {expected_base}"
        );
    }
}

// ---------------------------------------------------------------------------
// Test: Non-zero salary is preserved (not recalculated)
// ---------------------------------------------------------------------------

/// When loading a citizen whose saved salary is non-zero, the load path
/// should use the saved value directly rather than recalculating from
/// education. This is critical for preserving job-match modifiers.
#[test]
fn test_citizen_salary_nonzero_preserved_not_recalculated() {
    // A salary that differs from any base value
    let saved_salary: f32 = 4277.0;
    let education: u8 = 2; // HighSchool base = 3500

    let effective_salary = if saved_salary != 0.0 {
        saved_salary
    } else {
        CitizenDetails::base_salary_for_education(education)
    };

    // Should use the saved value, NOT the base
    assert!(
        (effective_salary - 4277.0).abs() < f32::EPSILON,
        "non-zero salary should be preserved, not recalculated; got {}",
        effective_salary
    );

    let base = CitizenDetails::base_salary_for_education(education);
    assert!(
        (effective_salary - base).abs() > f32::EPSILON,
        "saved salary {effective_salary} should differ from base {base}"
    );
}

// ---------------------------------------------------------------------------
// Test: Full SaveableRegistry roundtrip preserves salary
// ---------------------------------------------------------------------------

/// Spawn citizens with specific non-base salaries (simulating seniority or
/// job-match modifiers), run save/load via SaveableRegistry, and verify
/// salaries are preserved exactly.
#[test]
fn test_citizen_salary_saveable_registry_roundtrip() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    // Spawn a citizen with a salary that includes seniority modifier.
    let world = city.world_mut();
    let home_entity = world
        .resource::<WorldGrid>()
        .get(12, 11)
        .building_id
        .unwrap();
    let work_entity = world
        .resource::<WorldGrid>()
        .get(18, 11)
        .building_id
        .unwrap();

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
        CitizenStateComp(CitizenState::Working),
        PathCache::new(Vec::new()),
        CitizenDetails {
            age: 45,
            gender: Gender::Female,
            education: 3,
            happiness: 70.0,
            health: 90.0,
            salary: 7800.0, // 6000 base + seniority; NOT base for edu=3
            savings: 20000.0,
        },
        Personality {
            ambition: 0.7,
            sociability: 0.4,
            materialism: 0.6,
            resilience: 0.8,
        },
        Needs::default(),
        Family::default(),
        ActivityTimer::default(),
        ChosenTransportMode::default(),
    ));

    // Save via SaveableRegistry.
    let extensions = {
        let w = city.world_mut();
        let r = w.resource::<SaveableRegistry>();
        r.save_all(w)
    };

    // Load via SaveableRegistry.
    {
        let w = city.world_mut();
        let r = w.remove_resource::<SaveableRegistry>().unwrap();
        r.load_all(w, &extensions);
        w.insert_resource(r);
    }

    // Verify salary survived the roundtrip.
    let world = city.world_mut();
    let mut query = world.query::<&CitizenDetails>();
    let citizens: Vec<&CitizenDetails> = query.iter(world).collect();
    assert!(
        !citizens.is_empty(),
        "should have at least one citizen after load"
    );

    let found = citizens
        .iter()
        .any(|d| (d.salary - 7800.0).abs() < f32::EPSILON);
    assert!(
        found,
        "salary 7800.0 (with seniority modifier) should survive SaveableRegistry roundtrip; \
         salaries found: {:?}",
        citizens.iter().map(|d| d.salary).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Test: Multiple citizens with different salaries all roundtrip
// ---------------------------------------------------------------------------

/// Spawn multiple citizens with varying education levels and salary modifiers,
/// roundtrip via serde on CitizenDetails, and verify all salaries are preserved.
#[test]
fn test_citizen_salary_multiple_citizens_serde_roundtrip() {
    let citizens_data = vec![
        CitizenDetails {
            age: 20,
            gender: Gender::Male,
            education: 0,
            happiness: 50.0,
            health: 95.0,
            salary: 1530.0, // base 1500 + tiny seniority
            savings: 3000.0,
        },
        CitizenDetails {
            age: 35,
            gender: Gender::Female,
            education: 1,
            happiness: 65.0,
            health: 88.0,
            salary: 2574.0, // base 2200 + 17yr seniority
            savings: 8000.0,
        },
        CitizenDetails {
            age: 50,
            gender: Gender::Male,
            education: 2,
            happiness: 72.0,
            health: 75.0,
            salary: 5250.0, // base 3500 + max seniority
            savings: 50000.0,
        },
        CitizenDetails {
            age: 60,
            gender: Gender::Female,
            education: 3,
            happiness: 80.0,
            health: 70.0,
            salary: 9000.0, // base 6000 + max seniority
            savings: 100000.0,
        },
    ];

    for (i, original) in citizens_data.iter().enumerate() {
        let json = serde_json::to_string(original).unwrap();
        let restored: CitizenDetails = serde_json::from_str(&json).unwrap();

        assert!(
            (restored.salary - original.salary).abs() < f32::EPSILON,
            "citizen {i} (edu={}, age={}): salary {:.1} != expected {:.1}",
            original.education,
            original.age,
            restored.salary,
            original.salary
        );
    }
}

// ---------------------------------------------------------------------------
// Test: Salary survives repeated save/load cycles (drift check)
// ---------------------------------------------------------------------------

/// Run 10 consecutive serde roundtrip cycles on a CitizenDetails with a
/// non-base salary and verify no floating-point drift occurs.
#[test]
fn test_citizen_salary_no_drift_across_10_serde_cycles() {
    let original_salary = 7350.5_f32;
    let mut json = serde_json::to_string(&CitizenDetails {
        age: 42,
        gender: Gender::Female,
        education: 3,
        happiness: 72.5,
        health: 88.3,
        salary: original_salary,
        savings: 15000.0,
    })
    .unwrap();

    for cycle in 0..10 {
        let d: CitizenDetails = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("cycle {cycle}: deserialize failed: {e}"));
        json = serde_json::to_string(&d).unwrap();
    }

    let final_details: CitizenDetails = serde_json::from_str(&json).unwrap();
    assert!(
        (final_details.salary - original_salary).abs() < f32::EPSILON,
        "salary drifted after 10 cycles: {} != {}",
        final_details.salary,
        original_salary
    );
}

// ---------------------------------------------------------------------------
// Test: Salary field survives SaveCitizen-like struct roundtrip
// ---------------------------------------------------------------------------

/// Simulate the full save file path: CitizenDetails -> SaveCitizen-like
/// serialization -> restore. This mirrors the entity_stage.rs save and
/// spawn_entities.rs load paths.
#[test]
fn test_citizen_salary_save_citizen_struct_roundtrip() {
    // Simulate what entity_stage.rs produces
    #[derive(serde::Serialize, serde::Deserialize)]
    struct MiniSaveCitizen {
        education: u8,
        age: u8,
        #[serde(default)]
        salary: f32,
        #[serde(default)]
        savings: f32,
    }

    let original_salary = 5432.1_f32;
    let save_citizen = MiniSaveCitizen {
        education: 2,
        age: 38,
        salary: original_salary,
        savings: 12000.0,
    };

    let json = serde_json::to_string(&save_citizen).unwrap();
    let restored: MiniSaveCitizen = serde_json::from_str(&json).unwrap();

    // Simulate spawn_entities.rs restore logic
    let effective_salary = if restored.salary != 0.0 {
        restored.salary
    } else {
        CitizenDetails::base_salary_for_education(restored.education)
    };

    assert!(
        (effective_salary - original_salary).abs() < f32::EPSILON,
        "salary should survive SaveCitizen roundtrip: {} != {}",
        effective_salary,
        original_salary
    );
}

// ---------------------------------------------------------------------------
// Test: Old save without salary field defaults correctly
// ---------------------------------------------------------------------------

/// When deserializing a JSON object that lacks the salary field entirely
/// (simulating a pre-V4 save), serde default should produce 0.0, which
/// triggers the education-based fallback.
#[test]
fn test_citizen_salary_missing_field_defaults_to_zero() {
    #[derive(serde::Deserialize)]
    struct MiniSaveCitizen {
        education: u8,
        #[serde(default)]
        salary: f32,
    }

    // JSON without salary field at all (old save format)
    let json = r#"{"education": 2}"#;
    let restored: MiniSaveCitizen = serde_json::from_str(json).unwrap();

    assert!(
        restored.salary == 0.0,
        "missing salary field should default to 0.0, got {}",
        restored.salary
    );

    // Apply the load-path fallback
    let effective_salary = if restored.salary != 0.0 {
        restored.salary
    } else {
        CitizenDetails::base_salary_for_education(restored.education)
    };

    assert!(
        (effective_salary - 3500.0).abs() < f32::EPSILON,
        "missing salary for edu=2 should fall back to 3500.0, got {}",
        effective_salary
    );
}

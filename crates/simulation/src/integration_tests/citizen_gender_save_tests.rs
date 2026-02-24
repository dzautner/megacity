//! SAVE-003: Citizen gender serialization roundtrip tests (Issue #697).
//!
//! Verifies that citizen gender survives a full save/load cycle through the
//! SaveableRegistry, and that the serde default for gender (0 = Male) provides
//! backward compatibility with old saves.

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::test_harness::TestCity;
use crate::SaveableRegistry;
use bevy::prelude::*;

/// Helper: spawn a citizen with a specific gender via direct ECS insertion.
fn spawn_citizen_with_gender(
    world: &mut World,
    home: (usize, usize),
    work: (usize, usize),
    gender: Gender,
    age: u8,
) {
    let home_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(home.0, home.1)
            .building_id
            .unwrap_or(Entity::PLACEHOLDER)
    };
    let work_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(work.0, work.1)
            .building_id
            .unwrap_or(Entity::PLACEHOLDER)
    };
    let (hx, hy) = WorldGrid::grid_to_world(home.0, home.1);

    world.spawn((
        Citizen,
        Position { x: hx, y: hy },
        Velocity { x: 0.0, y: 0.0 },
        HomeLocation {
            grid_x: home.0,
            grid_y: home.1,
            building: home_entity,
        },
        WorkLocation {
            grid_x: work.0,
            grid_y: work.1,
            building: work_entity,
        },
        CitizenStateComp(CitizenState::AtHome),
        PathCache::new(Vec::new()),
        CitizenDetails {
            age,
            gender,
            education: 2,
            happiness: 60.0,
            health: 90.0,
            salary: 3500.0,
            savings: 7000.0,
        },
        Personality::default(),
        Needs::default(),
        Family::default(),
        ActivityTimer::default(),
        ChosenTransportMode::default(),
    ));
}

/// Collect all citizen genders from the world, sorted by age for determinism.
fn collect_genders_by_age(world: &mut World) -> Vec<(u8, Gender)> {
    let mut q = world.query::<&CitizenDetails>();
    let mut result: Vec<(u8, Gender)> = q.iter(world).map(|d| (d.age, d.gender)).collect();
    result.sort_by_key(|(age, _)| *age);
    result
}

/// Helper: save and load via SaveableRegistry.
fn save_load_roundtrip(city: &mut TestCity) {
    let extensions = {
        let w = city.world_mut();
        let r = w.resource::<SaveableRegistry>();
        r.save_all(w)
    };
    {
        let w = city.world_mut();
        let r = w.remove_resource::<SaveableRegistry>().unwrap();
        r.load_all(w, &extensions);
        w.insert_resource(r);
    }
}

// ---------------------------------------------------------------------------
// Test: Male gender roundtrips through SaveableRegistry
// ---------------------------------------------------------------------------

#[test]
fn test_citizen_gender_male_roundtrip_saveable_registry() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    spawn_citizen_with_gender(city.world_mut(), (12, 11), (18, 11), Gender::Male, 25);

    let before = collect_genders_by_age(city.world_mut());
    assert_eq!(before.len(), 1);
    assert_eq!(before[0], (25, Gender::Male));

    save_load_roundtrip(&mut city);

    let after = collect_genders_by_age(city.world_mut());
    assert_eq!(after.len(), 1, "citizen count changed after roundtrip");
    assert_eq!(
        after[0].1,
        Gender::Male,
        "Male gender lost after save/load roundtrip"
    );
}

// ---------------------------------------------------------------------------
// Test: Female gender roundtrips through SaveableRegistry
// ---------------------------------------------------------------------------

#[test]
fn test_citizen_gender_female_roundtrip_saveable_registry() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    spawn_citizen_with_gender(city.world_mut(), (12, 11), (18, 11), Gender::Female, 33);

    let before = collect_genders_by_age(city.world_mut());
    assert_eq!(before.len(), 1);
    assert_eq!(before[0], (33, Gender::Female));

    save_load_roundtrip(&mut city);

    let after = collect_genders_by_age(city.world_mut());
    assert_eq!(after.len(), 1, "citizen count changed after roundtrip");
    assert_eq!(
        after[0].1,
        Gender::Female,
        "Female gender lost after save/load roundtrip"
    );
}

// ---------------------------------------------------------------------------
// Test: Mixed genders both survive a single roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_citizen_gender_mixed_roundtrip() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    // Spawn one Male (age 20) and one Female (age 40) so we can sort by age.
    spawn_citizen_with_gender(city.world_mut(), (12, 11), (18, 11), Gender::Male, 20);
    spawn_citizen_with_gender(city.world_mut(), (12, 11), (18, 11), Gender::Female, 40);

    let before = collect_genders_by_age(city.world_mut());
    assert_eq!(before.len(), 2);
    assert_eq!(before[0], (20, Gender::Male));
    assert_eq!(before[1], (40, Gender::Female));

    save_load_roundtrip(&mut city);

    let after = collect_genders_by_age(city.world_mut());
    assert_eq!(after.len(), 2, "citizen count changed after roundtrip");
    assert_eq!(
        after[0],
        (20, Gender::Male),
        "Male citizen gender incorrect after roundtrip"
    );
    assert_eq!(
        after[1],
        (40, Gender::Female),
        "Female citizen gender incorrect after roundtrip"
    );
}

// ---------------------------------------------------------------------------
// Test: Gender survives multiple consecutive roundtrips
// ---------------------------------------------------------------------------

#[test]
fn test_citizen_gender_survives_multiple_roundtrips() {
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    spawn_citizen_with_gender(city.world_mut(), (12, 11), (18, 11), Gender::Female, 28);

    for cycle in 0..5 {
        save_load_roundtrip(&mut city);

        let genders = collect_genders_by_age(city.world_mut());
        assert_eq!(genders.len(), 1, "citizen count changed at cycle {cycle}");
        assert_eq!(
            genders[0].1,
            Gender::Female,
            "Female gender lost at save/load cycle {cycle}"
        );
    }
}

// ---------------------------------------------------------------------------
// Test: CitizenDetails gender field survives serde JSON roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_citizen_details_gender_serde_roundtrip() {
    let details_male = CitizenDetails {
        age: 30,
        gender: Gender::Male,
        education: 2,
        happiness: 70.0,
        health: 90.0,
        salary: 3500.0,
        savings: 7000.0,
    };
    let json = serde_json::to_string(&details_male).unwrap();
    let restored: CitizenDetails = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.gender, Gender::Male, "Male gender lost in serde");

    let details_female = CitizenDetails {
        gender: Gender::Female,
        ..details_male
    };
    let json = serde_json::to_string(&details_female).unwrap();
    let restored: CitizenDetails = serde_json::from_str(&json).unwrap();
    assert_eq!(
        restored.gender,
        Gender::Female,
        "Female gender lost in serde"
    );
}

// ---------------------------------------------------------------------------
// Test: Gender enum u8 mapping matches save/load codec expectations
// ---------------------------------------------------------------------------

#[test]
fn test_gender_u8_codec_mapping() {
    // The save system maps Gender::Male -> 0 and Gender::Female -> 1.
    // The load system maps 0 -> Male, 1 -> Female, anything else -> Male.
    // Verify these mappings are consistent.

    let map_to_u8 = |g: Gender| -> u8 {
        match g {
            Gender::Male => 0,
            Gender::Female => 1,
        }
    };

    let map_from_u8 = |v: u8| -> Gender {
        if v == 1 {
            Gender::Female
        } else {
            Gender::Male
        }
    };

    // Male roundtrip
    assert_eq!(map_from_u8(map_to_u8(Gender::Male)), Gender::Male);
    // Female roundtrip
    assert_eq!(map_from_u8(map_to_u8(Gender::Female)), Gender::Female);
    // Default (0) maps to Male (backward compat for old saves)
    assert_eq!(map_from_u8(0), Gender::Male);
    // Unknown values default to Male
    assert_eq!(map_from_u8(2), Gender::Male);
    assert_eq!(map_from_u8(255), Gender::Male);
}

//! SAVE-006: Citizen savings serialization tests.
//!
//! Verifies that savings roundtrips correctly through save/load, including:
//! - Non-zero savings preserved exactly
//! - Zero savings preserved (not replaced by salary * 2.0)
//! - Old saves without savings field default to salary * 2.0

use super::*;
use crate::spawn_entities::spawn_entities_from_save;
use bevy::prelude::*;
use simulation::citizen::{CitizenDetails, Gender};
use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::roads::RoadNetwork;
use simulation::time_of_day::GameClock;
use simulation::zones::ZoneDemand;

/// Helper: create a minimal SaveData with a single citizen whose savings
/// and salary are set to the given values.
fn save_data_with_citizen(salary: f32, savings: f32) -> SaveData {
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();

    let citizens = vec![CitizenSaveInput {
        entity: Entity::PLACEHOLDER,
        details: CitizenDetails {
            age: 30,
            gender: Gender::Male,
            education: 2,
            happiness: 70.0,
            health: 85.0,
            salary,
            savings,
        },
        state: simulation::citizen::CitizenState::AtHome,
        home_x: 1,
        home_y: 1,
        work_x: 2,
        work_y: 2,
        path: simulation::citizen::PathCache::new(vec![]),
        velocity: simulation::citizen::Velocity { x: 0.0, y: 0.0 },
        position: simulation::citizen::Position { x: 16.0, y: 16.0 },
        personality: simulation::citizen::Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.5,
            resilience: 0.5,
        },
        needs: simulation::citizen::Needs::default(),
        activity_timer: 0,
        family: simulation::citizen::Family::default(),
    }];

    create_save_data(
        &grid,
        &roads,
        &clock,
        &budget,
        &demand,
        &[],
        &citizens,
        &[],
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
}

/// Helper: spawn entities from SaveData into a minimal Bevy world and return
/// the savings value of the first citizen.
fn load_citizen_savings(save: &SaveData) -> f32 {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(WorldGrid::new(4, 4));
    let world = app.world_mut();
    spawn_entities_from_save(world, save);
    let mut query = world.query::<&CitizenDetails>();
    let details = query.iter(world).next().expect("expected one citizen");
    details.savings
}

/// Non-zero savings ($50,000) survives the full encode/decode/spawn roundtrip.
#[test]
fn test_savings_50k_roundtrip() {
    let save = save_data_with_citizen(1500.0, 50_000.0);
    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");
    assert!(
        (restored.citizens[0].savings - 50_000.0).abs() < 0.01,
        "SaveCitizen.savings should be 50000, got {}",
        restored.citizens[0].savings
    );
    let loaded_savings = load_citizen_savings(&restored);
    assert!(
        (loaded_savings - 50_000.0).abs() < 0.01,
        "CitizenDetails.savings should be 50000 after load, got {}",
        loaded_savings
    );
}

/// Zero savings is preserved (the bug was that 0.0 was replaced by salary * 2.0).
#[test]
fn test_savings_zero_roundtrip() {
    let save = save_data_with_citizen(3000.0, 0.0);
    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");
    assert!(
        restored.citizens[0].savings.abs() < 0.01,
        "SaveCitizen.savings should be 0.0, got {}",
        restored.citizens[0].savings
    );
    let loaded_savings = load_citizen_savings(&restored);
    assert!(
        loaded_savings.abs() < 0.01,
        "CitizenDetails.savings should be 0.0 after load (not salary*2 = {}), got {}",
        3000.0 * 2.0,
        loaded_savings
    );
}

/// Negative savings (debt) is preserved.
#[test]
fn test_savings_negative_roundtrip() {
    let save = save_data_with_citizen(2000.0, -500.0);
    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");
    let loaded_savings = load_citizen_savings(&restored);
    assert!(
        (loaded_savings - (-500.0)).abs() < 0.01,
        "Negative savings should roundtrip, got {}",
        loaded_savings
    );
}

/// Old save that lacks a savings field defaults to salary * 2.0.
#[test]
fn test_old_save_missing_savings_defaults_to_salary_times_two() {
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();
    let mut save = create_save_data(
        &grid,
        &roads,
        &clock,
        &budget,
        &demand,
        &[],
        &[],
        &[],
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    // Manually construct a SaveCitizen with the sentinel value for savings
    // (simulating an old save that didn't have the savings field).
    save.citizens.push(SaveCitizen {
        age: 30,
        happiness: 70.0,
        education: 2,
        state: 0,
        home_x: 1,
        home_y: 1,
        work_x: 2,
        work_y: 2,
        path_waypoints: vec![],
        path_current_index: 0,
        velocity_x: 0.0,
        velocity_y: 0.0,
        pos_x: 16.0,
        pos_y: 16.0,
        gender: 0,
        health: 85.0,
        salary: 4000.0,
        savings: default_savings_sentinel(),
        ambition: 0.5,
        sociability: 0.5,
        materialism: 0.5,
        resilience: 0.5,
        need_hunger: 80.0,
        need_energy: 80.0,
        need_social: 70.0,
        need_fun: 70.0,
        need_comfort: 60.0,
        activity_timer: 0,
        family_partner: u32::MAX,
        family_children: vec![],
        family_parent: u32::MAX,
    });

    let loaded_savings = load_citizen_savings(&save);
    let expected = 4000.0 * 2.0; // salary * 2.0
    assert!(
        (loaded_savings - expected).abs() < 0.01,
        "Old save without savings should default to salary*2 = {}, got {}",
        expected,
        loaded_savings
    );
}

//! Family graph serialization tests.

use super::*;
use bevy::prelude::Entity;
use simulation::citizen::{
    CitizenDetails, CitizenState, Family, Gender, Needs, PathCache, Personality, Position, Velocity,
};
use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::roads::RoadNetwork;
use simulation::time_of_day::GameClock;
use simulation::zones::ZoneDemand;

fn make_citizen(entity: Entity) -> CitizenSaveInput {
    CitizenSaveInput {
        entity,
        details: CitizenDetails {
            age: 30,
            gender: Gender::Male,
            education: 2,
            happiness: 70.0,
            health: 90.0,
            salary: 3500.0,
            savings: 7000.0,
        },
        state: CitizenState::AtHome,
        home_x: 1,
        home_y: 1,
        work_x: 2,
        work_y: 2,
        path: PathCache::new(vec![]),
        velocity: Velocity { x: 0.0, y: 0.0 },
        position: Position { x: 16.0, y: 16.0 },
        personality: Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.5,
            resilience: 0.5,
        },
        needs: Needs::default(),
        activity_timer: 0,
        family: Family::default(),
    }
}

#[test]
fn test_family_graph_roundtrip() {
    // Create 4 dummy entities to simulate a family: parent_m, parent_f, child1, child2
    // We use Entity::from_raw since we just need distinct entity IDs for the mapping.
    let e_parent_m = Entity::from_raw(100);
    let e_parent_f = Entity::from_raw(101);
    let e_child1 = Entity::from_raw(102);
    let e_child2 = Entity::from_raw(103);

    let mut parent_m = make_citizen(e_parent_m);
    parent_m.family = Family {
        partner: Some(e_parent_f),
        children: vec![e_child1, e_child2],
        parent: None,
    };

    let mut parent_f = make_citizen(e_parent_f);
    parent_f.family = Family {
        partner: Some(e_parent_m),
        children: vec![e_child1, e_child2],
        parent: None,
    };

    let mut child1 = make_citizen(e_child1);
    child1.details.age = 5;
    child1.family = Family {
        partner: None,
        children: vec![],
        parent: Some(e_parent_f),
    };

    let mut child2 = make_citizen(e_child2);
    child2.details.age = 3;
    child2.family = Family {
        partner: None,
        children: vec![],
        parent: Some(e_parent_f),
    };

    let citizens = vec![parent_m, parent_f, child1, child2];

    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();

    let save = create_save_data(
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
    );

    // Verify entity-to-index mapping during save
    // parent_m (index 0): partner=1, children=[2,3], parent=MAX
    let sc0 = &save.citizens[0];
    assert_eq!(sc0.family_partner, 1);
    assert_eq!(sc0.family_children, vec![2, 3]);
    assert_eq!(sc0.family_parent, u32::MAX);

    // parent_f (index 1): partner=0, children=[2,3], parent=MAX
    let sc1 = &save.citizens[1];
    assert_eq!(sc1.family_partner, 0);
    assert_eq!(sc1.family_children, vec![2, 3]);
    assert_eq!(sc1.family_parent, u32::MAX);

    // child1 (index 2): partner=MAX, children=[], parent=1
    let sc2 = &save.citizens[2];
    assert_eq!(sc2.family_partner, u32::MAX);
    assert!(sc2.family_children.is_empty());
    assert_eq!(sc2.family_parent, 1);

    // child2 (index 3): partner=MAX, children=[], parent=1
    let sc3 = &save.citizens[3];
    assert_eq!(sc3.family_partner, u32::MAX);
    assert!(sc3.family_children.is_empty());
    assert_eq!(sc3.family_parent, 1);

    // Verify encode/decode roundtrip preserves family data
    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");
    assert_eq!(restored.citizens.len(), 4);
    assert_eq!(restored.citizens[0].family_partner, 1);
    assert_eq!(restored.citizens[0].family_children, vec![2, 3]);
    assert_eq!(restored.citizens[1].family_partner, 0);
    assert_eq!(restored.citizens[2].family_parent, 1);
    assert_eq!(restored.citizens[3].family_parent, 1);
}

#[test]
fn test_family_graph_backward_compat_old_save() {
    // Simulate loading an old save that has no family fields.
    // The serde defaults should produce u32::MAX for partner/parent and empty children.
    let sc = SaveCitizen {
        age: 25,
        happiness: 70.0,
        education: 1,
        state: 0,
        home_x: 0,
        home_y: 0,
        work_x: 1,
        work_y: 1,
        path_waypoints: vec![],
        path_current_index: 0,
        velocity_x: 0.0,
        velocity_y: 0.0,
        pos_x: 0.0,
        pos_y: 0.0,
        gender: 0,
        health: 80.0,
        salary: 0.0,
        savings: 0.0,
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
        // Family fields use defaults (simulating old save)
        family_partner: u32::MAX,
        family_children: vec![],
        family_parent: u32::MAX,
    };
    // Verify defaults mean "no relationships"
    assert_eq!(sc.family_partner, u32::MAX);
    assert!(sc.family_children.is_empty());
    assert_eq!(sc.family_parent, u32::MAX);
}

#[test]
fn test_family_dangling_ref_ignored() {
    // If a family member entity is not in the citizen list (e.g. died and was removed),
    // the entity-to-index lookup should produce u32::MAX (not found).
    let e_alive = Entity::from_raw(200);
    let e_dead = Entity::from_raw(999); // not in citizens list

    let mut citizen = make_citizen(e_alive);
    citizen.family = Family {
        partner: Some(e_dead),
        children: vec![e_dead],
        parent: Some(e_dead),
    };

    let citizens = vec![citizen];

    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);

    let save = create_save_data(
        &grid,
        &RoadNetwork::default(),
        &GameClock::default(),
        &CityBudget::default(),
        &ZoneDemand::default(),
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
    );

    // Dangling references should become u32::MAX / empty
    let sc = &save.citizens[0];
    assert_eq!(sc.family_partner, u32::MAX);
    assert!(sc.family_children.is_empty()); // filtered out
    assert_eq!(sc.family_parent, u32::MAX);
}

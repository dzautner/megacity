//! Citizen serialization tests: virtual population, path cache, velocity.

use super::*;
use bevy::prelude::Entity;
use simulation::citizen::CitizenState;
use simulation::citizen::{
    CitizenDetails, Family, Needs, PathCache, Personality, Position, Velocity,
};
use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::roads::RoadNetwork;
use simulation::time_of_day::GameClock;
use simulation::virtual_population::VirtualPopulation;
use simulation::zones::ZoneDemand;

#[test]
fn test_virtual_population_roundtrip() {
    let mut vp = VirtualPopulation::default();
    vp.add_virtual_citizen(0, 25, true, 75.0, 1000.0, 0.1);
    vp.add_virtual_citizen(0, 40, false, 50.0, 0.0, 0.0);
    vp.add_virtual_citizen(1, 60, true, 80.0, 1500.0, 0.12);

    let save = SaveVirtualPopulation {
        total_virtual: vp.total_virtual,
        virtual_employed: vp.virtual_employed,
        district_stats: vp
            .district_stats
            .iter()
            .map(|ds| SaveDistrictStats {
                population: ds.population,
                employed: ds.employed,
                avg_happiness: ds.avg_happiness,
                avg_age: ds.avg_age,
                age_brackets: ds.age_brackets,
                commuters_out: ds.commuters_out,
                tax_contribution: ds.tax_contribution,
                service_demand: ds.service_demand,
            })
            .collect(),
        max_real_citizens: vp.max_real_citizens,
    };

    let restored = restore_virtual_population(&save);
    assert_eq!(restored.total_virtual, 3);
    assert_eq!(restored.virtual_employed, 2);
    assert_eq!(restored.district_stats.len(), 2);
    assert_eq!(restored.district_stats[0].population, 2);
    assert_eq!(restored.district_stats[0].employed, 1);
    assert_eq!(restored.district_stats[1].population, 1);
    assert_eq!(restored.district_stats[1].employed, 1);
    assert_eq!(restored.max_real_citizens, vp.max_real_citizens);
}

#[test]
fn test_pathcache_velocity_citizen_roundtrip() {
    use simulation::roads::RoadNode;
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();
    let citizens = vec![
        CitizenSaveInput {
            details: CitizenDetails {
                age: 30,
                gender: simulation::citizen::Gender::Male,
                education: 2,
                happiness: 75.0,
                health: 90.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            state: CitizenState::CommutingToWork,
            home_x: 1,
            home_y: 1,
            work_x: 3,
            work_y: 3,
            path: PathCache {
                waypoints: vec![
                    RoadNode(1, 1),
                    RoadNode(2, 1),
                    RoadNode(2, 2),
                    RoadNode(3, 3),
                ],
                current_index: 1,
            },
            velocity: Velocity { x: 4.5, y: -2.3 },
            position: Position { x: 100.0, y: 200.0 },
            personality: Personality {
                ambition: 0.8,
                sociability: 0.6,
                materialism: 0.4,
                resilience: 0.9,
            },
            needs: Needs {
                hunger: 90.0,
                energy: 85.0,
                social: 60.0,
                fun: 55.0,
                comfort: 70.0,
            },
            activity_timer: 42,
            entity: Entity::PLACEHOLDER,
            family: Family::default(),
        },
        CitizenSaveInput {
            details: CitizenDetails {
                age: 45,
                gender: simulation::citizen::Gender::Female,
                education: 1,
                happiness: 60.0,
                health: 80.0,
                salary: 2200.0,
                savings: 4400.0,
            },
            state: CitizenState::AtHome,
            home_x: 2,
            home_y: 2,
            work_x: 3,
            work_y: 2,
            path: PathCache {
                waypoints: vec![],
                current_index: 0,
            },
            velocity: Velocity { x: 0.0, y: 0.0 },
            position: Position { x: 50.0, y: 75.0 },
            personality: Personality {
                ambition: 0.3,
                sociability: 0.7,
                materialism: 0.5,
                resilience: 0.2,
            },
            needs: Needs {
                hunger: 70.0,
                energy: 65.0,
                social: 80.0,
                fun: 75.0,
                comfort: 50.0,
            },
            activity_timer: 0,
            entity: Entity::PLACEHOLDER,
            family: Family::default(),
        },
    ];
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
    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");
    assert_eq!(restored.citizens.len(), 2);
    // First citizen: active path with waypoints
    let c0 = &restored.citizens[0];
    assert_eq!(c0.path_waypoints, vec![(1, 1), (2, 1), (2, 2), (3, 3)]);
    assert_eq!(c0.path_current_index, 1);
    assert!((c0.velocity_x - 4.5).abs() < 0.001);
    assert!((c0.velocity_y - (-2.3)).abs() < 0.001);
    assert!((c0.pos_x - 100.0).abs() < 0.001);
    assert!((c0.pos_y - 200.0).abs() < 0.001);
    assert_eq!(c0.state, 1); // CommutingToWork
                             // V4 fields: gender, health, salary, savings, personality, needs
    assert_eq!(c0.gender, 0); // Male
    assert!((c0.health - 90.0).abs() < 0.001);
    assert!((c0.salary - 3500.0).abs() < 0.001);
    assert!((c0.savings - 7000.0).abs() < 0.001);
    assert!((c0.ambition - 0.8).abs() < 0.001);
    assert!((c0.sociability - 0.6).abs() < 0.001);
    assert!((c0.materialism - 0.4).abs() < 0.001);
    assert!((c0.resilience - 0.9).abs() < 0.001);
    assert!((c0.need_hunger - 90.0).abs() < 0.001);
    assert!((c0.need_energy - 85.0).abs() < 0.001);
    assert!((c0.need_social - 60.0).abs() < 0.001);
    assert!((c0.need_fun - 55.0).abs() < 0.001);
    assert!((c0.need_comfort - 70.0).abs() < 0.001);
    assert_eq!(c0.activity_timer, 42);
    // V32 fields: family graph (default = no relationships)
    assert_eq!(c0.family_partner, u32::MAX);
    assert!(c0.family_children.is_empty());
    assert_eq!(c0.family_parent, u32::MAX);
    // Second citizen: idle, empty path
    let c1 = &restored.citizens[1];
    assert!(c1.path_waypoints.is_empty());
    assert_eq!(c1.path_current_index, 0);
    assert!((c1.velocity_x).abs() < 0.001);
    assert!((c1.velocity_y).abs() < 0.001);
    assert!((c1.pos_x - 50.0).abs() < 0.001);
    assert!((c1.pos_y - 75.0).abs() < 0.001);
    assert_eq!(c1.state, 0); // AtHome
                             // V4 fields for second citizen
    assert_eq!(c1.gender, 1); // Female
    assert!((c1.health - 80.0).abs() < 0.001);
    assert!((c1.salary - 2200.0).abs() < 0.001);
    assert!((c1.savings - 4400.0).abs() < 0.001);
    assert!((c1.ambition - 0.3).abs() < 0.001);
    assert!((c1.sociability - 0.7).abs() < 0.001);
    assert!((c1.materialism - 0.5).abs() < 0.001);
    assert!((c1.resilience - 0.2).abs() < 0.001);
    assert!((c1.need_hunger - 70.0).abs() < 0.001);
    assert!((c1.need_energy - 65.0).abs() < 0.001);
    assert!((c1.need_social - 80.0).abs() < 0.001);
    assert!((c1.need_fun - 75.0).abs() < 0.001);
    assert!((c1.need_comfort - 50.0).abs() < 0.001);
    assert_eq!(c1.activity_timer, 0);
}

#[test]
fn test_pathcache_velocity_v2_backward_compat() {
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
    // Simulate an old save citizen with default V3 fields
    save.citizens.push(SaveCitizen {
        age: 25,
        happiness: 70.0,
        education: 1,
        state: 1, // CommutingToWork
        home_x: 1,
        home_y: 1,
        work_x: 3,
        work_y: 3,
        path_waypoints: vec![],
        path_current_index: 0,
        velocity_x: 0.0,
        velocity_y: 0.0,
        pos_x: 0.0,
        pos_y: 0.0,
        // V4 fields: use defaults to simulate old save
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
        family_partner: u32::MAX,
        family_children: vec![],
        family_parent: u32::MAX,
    });
    save.version = 2;
    let old = migrate_save(&mut save).expect("migration should succeed");
    assert_eq!(old, 2);
    assert_eq!(save.version, CURRENT_SAVE_VERSION);
    let c = &save.citizens[0];
    assert!(c.path_waypoints.is_empty());
    assert_eq!(c.path_current_index, 0);
    assert!((c.velocity_x).abs() < 0.001);
    assert!((c.velocity_y).abs() < 0.001);
}

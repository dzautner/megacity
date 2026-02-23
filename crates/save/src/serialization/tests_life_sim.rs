//! Life simulation timer serialization and migration v3 tests.

use super::*;

use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::life_simulation::LifeSimTimer;
use simulation::roads::RoadNetwork;
use simulation::time_of_day::GameClock;
use simulation::zones::ZoneDemand;

#[test]
fn test_life_sim_timer_roundtrip() {
    let timer = LifeSimTimer {
        needs_tick: 7,
        life_event_tick: 123,
        salary_tick: 9999,
        education_tick: 500,
        job_seek_tick: 42,
        personality_tick: 1234,
        health_tick: 777,
    };

    let save = SaveLifeSimTimer {
        needs_tick: timer.needs_tick,
        life_event_tick: timer.life_event_tick,
        salary_tick: timer.salary_tick,
        education_tick: timer.education_tick,
        job_seek_tick: timer.job_seek_tick,
        personality_tick: timer.personality_tick,
        health_tick: timer.health_tick,
    };

    let restored = restore_life_sim_timer(&save);
    assert_eq!(restored.needs_tick, 7);
    assert_eq!(restored.life_event_tick, 123);
    assert_eq!(restored.salary_tick, 9999);
    assert_eq!(restored.education_tick, 500);
    assert_eq!(restored.job_seek_tick, 42);
    assert_eq!(restored.personality_tick, 1234);
    assert_eq!(restored.health_tick, 777);
}

#[test]
fn test_life_sim_timer_full_roundtrip() {
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();

    let life_sim_timer = LifeSimTimer {
        needs_tick: 5,
        life_event_tick: 300,
        salary_tick: 20000,
        education_tick: 700,
        job_seek_tick: 100,
        personality_tick: 1500,
        health_tick: 900,
    };

    let save = create_save_data(
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
        Some(&life_sim_timer),
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

    let rlst = restored
        .life_sim_timer
        .as_ref()
        .expect("life_sim_timer present");
    assert_eq!(rlst.needs_tick, 5);
    assert_eq!(rlst.life_event_tick, 300);
    assert_eq!(rlst.salary_tick, 20000);
    assert_eq!(rlst.education_tick, 700);
    assert_eq!(rlst.job_seek_tick, 100);
    assert_eq!(rlst.personality_tick, 1500);
    assert_eq!(rlst.health_tick, 900);
}

#[test]
fn test_life_sim_timer_backward_compat() {
    // Saves without life_sim_timer should have it as None
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

    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");
    assert!(restored.life_sim_timer.is_none());
}

#[test]
fn test_migrate_from_v3() {
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
    save.version = 3;

    let old = migrate_save(&mut save).expect("migration should succeed");
    assert_eq!(old, 3);
    assert_eq!(save.version, CURRENT_SAVE_VERSION);
}

//! Vacancy rates and extension map roundtrip/backward compatibility tests.

use super::*;

use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::roads::RoadNetwork;
use simulation::time_of_day::GameClock;
use simulation::zones::ZoneDemand;

#[test]
fn test_vacancy_rates_roundtrip() {
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand {
        residential: 0.7,
        commercial: 0.5,
        industrial: 0.3,
        office: 0.2,
        vacancy_residential: 0.06,
        vacancy_commercial: 0.12,
        vacancy_industrial: 0.08,
        vacancy_office: 0.10,
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

    assert!((restored.demand.residential - 0.7).abs() < 0.001);
    assert!((restored.demand.commercial - 0.5).abs() < 0.001);
    assert!((restored.demand.industrial - 0.3).abs() < 0.001);
    assert!((restored.demand.office - 0.2).abs() < 0.001);
    assert!((restored.demand.vacancy_residential - 0.06).abs() < 0.001);
    assert!((restored.demand.vacancy_commercial - 0.12).abs() < 0.001);
    assert!((restored.demand.vacancy_industrial - 0.08).abs() < 0.001);
    assert!((restored.demand.vacancy_office - 0.10).abs() < 0.001);
}

#[test]
fn test_extension_map_roundtrip() {
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

    // Simulate extension map entries as if populated by SaveableRegistry
    save.extensions
        .insert("test_feature_a".to_string(), vec![1, 2, 3, 4]);
    save.extensions
        .insert("test_feature_b".to_string(), vec![10, 20]);

    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");

    assert_eq!(restored.extensions.len(), 2);
    assert_eq!(
        restored.extensions.get("test_feature_a"),
        Some(&vec![1, 2, 3, 4])
    );
    assert_eq!(
        restored.extensions.get("test_feature_b"),
        Some(&vec![10, 20])
    );
}

#[test]
fn test_extension_map_backward_compat() {
    // Saves without extensions field should have empty map
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
    assert!(restored.extensions.is_empty());
}

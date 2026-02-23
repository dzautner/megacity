//! Water source, degree days, and migration v4/v5 tests.

use super::*;

use simulation::degree_days::DegreeDays;
use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::roads::RoadNetwork;
use simulation::time_of_day::GameClock;
use simulation::water_sources::{WaterSource, WaterSourceType};
use simulation::zones::ZoneDemand;

#[test]
fn test_water_source_type_roundtrip() {
    let types = [
        WaterSourceType::Well,
        WaterSourceType::SurfaceIntake,
        WaterSourceType::Reservoir,
        WaterSourceType::Desalination,
    ];
    for wt in &types {
        let encoded = water_source_type_to_u8(*wt);
        let decoded = u8_to_water_source_type(encoded).expect("valid water source type");
        assert_eq!(*wt, decoded);
    }
    assert!(u8_to_water_source_type(255).is_none());
}

#[test]
fn test_water_source_save_roundtrip() {
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();

    let water_sources = vec![
        WaterSource {
            source_type: WaterSourceType::Well,
            capacity_mgd: 0.5,
            quality: 0.7,
            operating_cost: 15.0,
            grid_x: 2,
            grid_y: 2,
            stored_gallons: 0.0,
            storage_capacity: 0.0,
        },
        WaterSource {
            source_type: WaterSourceType::Reservoir,
            capacity_mgd: 20.0,
            quality: 0.8,
            operating_cost: 200.0,
            grid_x: 1,
            grid_y: 1,
            stored_gallons: 1_800_000_000.0,
            storage_capacity: 1_800_000_000.0,
        },
    ];

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
        Some(&water_sources),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
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

    let rws = restored
        .water_sources
        .as_ref()
        .expect("water_sources present");
    assert_eq!(rws.len(), 2);

    let w0 = &rws[0];
    assert_eq!(
        u8_to_water_source_type(w0.source_type),
        Some(WaterSourceType::Well)
    );
    assert!((w0.capacity_mgd - 0.5).abs() < 0.001);
    assert!((w0.quality - 0.7).abs() < 0.001);
    assert_eq!(w0.grid_x, 2);
    assert_eq!(w0.grid_y, 2);

    let w1 = &rws[1];
    assert_eq!(
        u8_to_water_source_type(w1.source_type),
        Some(WaterSourceType::Reservoir)
    );
    assert!((w1.capacity_mgd - 20.0).abs() < 0.001);
    assert!(w1.stored_gallons > 0.0);
}

#[test]
fn test_water_source_restore() {
    let save = SaveWaterSource {
        source_type: water_source_type_to_u8(WaterSourceType::Desalination),
        grid_x: 5,
        grid_y: 5,
        capacity_mgd: 10.0,
        quality: 0.95,
        operating_cost: 500.0,
        stored_gallons: 0.0,
        storage_capacity: 0.0,
    };

    let ws = restore_water_source(&save).expect("valid water source");
    assert_eq!(ws.source_type, WaterSourceType::Desalination);
    assert!((ws.capacity_mgd - 10.0).abs() < 0.001);
    assert!((ws.quality - 0.95).abs() < 0.001);
    assert_eq!(ws.grid_x, 5);
    assert_eq!(ws.grid_y, 5);
}

#[test]
fn test_degree_days_roundtrip() {
    let dd = DegreeDays {
        daily_hdd: 15.5,
        daily_cdd: 0.0,
        monthly_hdd: [
            10.0, 20.0, 15.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 12.0, 25.0,
        ],
        monthly_cdd: [
            0.0, 0.0, 0.0, 0.0, 5.0, 15.0, 20.0, 18.0, 10.0, 0.0, 0.0, 0.0,
        ],
        annual_hdd: 92.5,
        annual_cdd: 68.0,
        last_update_day: 150,
    };

    let save = SaveDegreeDays {
        daily_hdd: dd.daily_hdd,
        daily_cdd: dd.daily_cdd,
        monthly_hdd: dd.monthly_hdd,
        monthly_cdd: dd.monthly_cdd,
        annual_hdd: dd.annual_hdd,
        annual_cdd: dd.annual_cdd,
        last_update_day: dd.last_update_day,
    };

    let restored = restore_degree_days(&save);
    assert!((restored.daily_hdd - 15.5).abs() < 0.001);
    assert!(restored.daily_cdd.abs() < 0.001);
    assert!((restored.monthly_hdd[0] - 10.0).abs() < 0.001);
    assert!((restored.monthly_cdd[6] - 20.0).abs() < 0.001);
    assert!((restored.annual_hdd - 92.5).abs() < 0.001);
    assert!((restored.annual_cdd - 68.0).abs() < 0.001);
    assert_eq!(restored.last_update_day, 150);
}

#[test]
fn test_water_source_backward_compat() {
    // Saves without water_sources should have it as None
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
    assert!(restored.water_sources.is_none());
    assert!(restored.degree_days.is_none());
}

#[test]
fn test_migrate_from_v4() {
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
    save.version = 4;

    let old = migrate_save(&mut save).expect("migration should succeed");
    assert_eq!(old, 4);
    assert_eq!(save.version, CURRENT_SAVE_VERSION);
    // Vacancy fields should default to 0.0 for a migrated v4 save.
    assert!((save.demand.vacancy_residential).abs() < 0.001);
    assert!((save.demand.vacancy_commercial).abs() < 0.001);
    assert!((save.demand.vacancy_industrial).abs() < 0.001);
    assert!((save.demand.vacancy_office).abs() < 0.001);
}

#[test]
fn test_migrate_from_v5() {
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
    save.version = 5;

    let old = migrate_save(&mut save).expect("migration should succeed");
    assert_eq!(old, 5);
    assert_eq!(save.version, CURRENT_SAVE_VERSION);
}

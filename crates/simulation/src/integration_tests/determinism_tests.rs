//! STAB-04: Determinism property test — same seed produces same state.
//!
//! Runs an identical simulation sequence twice and asserts that key state
//! (treasury, population, happiness, RNG position, building count, grid
//! cells with buildings) is bit-for-bit identical after 600 ticks.

use crate::grid::{RoadType, ZoneType};
use crate::sim_rng::SimRng;
use crate::stats::CityStats;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

/// Snapshot of key simulation state captured after running ticks.
#[derive(Debug, PartialEq)]
struct SimSnapshot {
    treasury: u64,        // f64 bits for exact comparison
    monthly_income: u64,  // f64 bits
    monthly_expenses: u64, // f64 bits
    population: u32,
    average_happiness: u32, // f32 bits
    citizen_count: usize,
    building_count: usize,
    rng_word_pos: u128,
    rng_seed: [u8; 32],
    cells_with_buildings: usize,
    road_cell_count: usize,
    residential_buildings: u32,
    commercial_buildings: u32,
    industrial_buildings: u32,
}

/// Build a city with roads, zones, utilities, buildings, and citizens,
/// advance the simulation, and return a snapshot of key state.
fn run_simulation() -> SimSnapshot {
    let mut city = TestCity::new()
        // Main east-west road
        .with_road(10, 20, 60, 20, RoadType::Avenue)
        // North-south cross-road
        .with_road(30, 10, 30, 40, RoadType::Local)
        // Second cross-road
        .with_road(50, 10, 50, 40, RoadType::Local)
        // Power plant and water tower
        .with_utility(8, 20, UtilityType::PowerPlant)
        .with_utility(62, 20, UtilityType::WaterTower)
        // Residential zone block (near the avenue, east side)
        .with_zone_rect(31, 21, 38, 25, ZoneType::ResidentialLow)
        // Commercial zone block (west side of cross-road)
        .with_zone_rect(22, 21, 29, 25, ZoneType::CommercialLow)
        // Industrial zone block (south of avenue)
        .with_zone_rect(31, 15, 38, 19, ZoneType::Industrial)
        // Seed buildings so citizens have homes and workplaces
        .with_building(32, 22, ZoneType::ResidentialLow, 2)
        .with_building(34, 22, ZoneType::ResidentialLow, 2)
        .with_building(24, 22, ZoneType::CommercialLow, 2)
        .with_building(26, 22, ZoneType::CommercialLow, 2)
        .with_building(32, 16, ZoneType::Industrial, 1)
        // Spawn citizens
        .with_citizen((32, 22), (24, 22))
        .with_citizen((32, 22), (26, 22))
        .with_citizen((34, 22), (24, 22))
        .with_citizen((34, 22), (32, 16))
        .with_citizen((34, 22), (26, 22))
        // Give starting budget
        .with_budget(75_000.0);

    // Run 6 slow cycles (~600 ticks) to exercise economy, building spawner,
    // population growth, happiness, and citizen movement.
    city.tick_slow_cycles(6);

    // Capture state snapshot
    let budget = city.budget().clone();
    let stats = city.resource::<CityStats>().clone();
    let rng = city.resource::<SimRng>();
    let rng_word_pos = rng.0.get_word_pos();
    let rng_seed = rng.0.get_seed();

    let citizen_count = city.citizen_count();
    let building_count = city.building_count();
    let cells_with_buildings = city.cells_with_buildings();
    let road_cell_count = city.road_cell_count();

    SimSnapshot {
        treasury: budget.treasury.to_bits(),
        monthly_income: budget.monthly_income.to_bits(),
        monthly_expenses: budget.monthly_expenses.to_bits(),
        population: stats.population,
        average_happiness: stats.average_happiness.to_bits(),
        citizen_count,
        building_count,
        rng_word_pos,
        rng_seed,
        cells_with_buildings,
        road_cell_count,
        residential_buildings: stats.residential_buildings,
        commercial_buildings: stats.commercial_buildings,
        industrial_buildings: stats.industrial_buildings,
    }
}

#[test]
fn test_simulation_determinism_same_seed_same_state() {
    let snapshot_a = run_simulation();
    let snapshot_b = run_simulation();

    assert_eq!(
        snapshot_a.treasury, snapshot_b.treasury,
        "Treasury diverged: run1={} run2={}",
        f64::from_bits(snapshot_a.treasury),
        f64::from_bits(snapshot_b.treasury),
    );
    assert_eq!(
        snapshot_a.monthly_income, snapshot_b.monthly_income,
        "Monthly income diverged: run1={} run2={}",
        f64::from_bits(snapshot_a.monthly_income),
        f64::from_bits(snapshot_b.monthly_income),
    );
    assert_eq!(
        snapshot_a.monthly_expenses, snapshot_b.monthly_expenses,
        "Monthly expenses diverged: run1={} run2={}",
        f64::from_bits(snapshot_a.monthly_expenses),
        f64::from_bits(snapshot_b.monthly_expenses),
    );
    assert_eq!(
        snapshot_a.population, snapshot_b.population,
        "Population diverged: run1={} run2={}",
        snapshot_a.population,
        snapshot_b.population,
    );
    assert_eq!(
        snapshot_a.average_happiness, snapshot_b.average_happiness,
        "Average happiness diverged: run1={} run2={}",
        f32::from_bits(snapshot_a.average_happiness),
        f32::from_bits(snapshot_b.average_happiness),
    );
    assert_eq!(
        snapshot_a.citizen_count, snapshot_b.citizen_count,
        "Citizen count diverged: run1={} run2={}",
        snapshot_a.citizen_count,
        snapshot_b.citizen_count,
    );
    assert_eq!(
        snapshot_a.building_count, snapshot_b.building_count,
        "Building count diverged: run1={} run2={}",
        snapshot_a.building_count,
        snapshot_b.building_count,
    );
    assert_eq!(
        snapshot_a.rng_word_pos, snapshot_b.rng_word_pos,
        "RNG word position diverged: run1={} run2={}",
        snapshot_a.rng_word_pos,
        snapshot_b.rng_word_pos,
    );
    assert_eq!(
        snapshot_a.rng_seed, snapshot_b.rng_seed,
        "RNG seed diverged",
    );
    assert_eq!(
        snapshot_a.cells_with_buildings, snapshot_b.cells_with_buildings,
        "Cells with buildings diverged: run1={} run2={}",
        snapshot_a.cells_with_buildings,
        snapshot_b.cells_with_buildings,
    );
    assert_eq!(
        snapshot_a.road_cell_count, snapshot_b.road_cell_count,
        "Road cell count diverged: run1={} run2={}",
        snapshot_a.road_cell_count,
        snapshot_b.road_cell_count,
    );
    assert_eq!(
        snapshot_a.residential_buildings, snapshot_b.residential_buildings,
        "Residential building count diverged: run1={} run2={}",
        snapshot_a.residential_buildings,
        snapshot_b.residential_buildings,
    );
    assert_eq!(
        snapshot_a.commercial_buildings, snapshot_b.commercial_buildings,
        "Commercial building count diverged: run1={} run2={}",
        snapshot_a.commercial_buildings,
        snapshot_b.commercial_buildings,
    );
    assert_eq!(
        snapshot_a.industrial_buildings, snapshot_b.industrial_buildings,
        "Industrial building count diverged: run1={} run2={}",
        snapshot_a.industrial_buildings,
        snapshot_b.industrial_buildings,
    );

    // Final comprehensive check (also covers any future fields)
    assert_eq!(
        snapshot_a, snapshot_b,
        "Full simulation snapshots diverged — the simulation is not deterministic!"
    );
}

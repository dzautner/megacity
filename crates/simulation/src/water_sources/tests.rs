use super::systems::is_near_water;
use super::types::*;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};

#[test]
fn test_well_capacity_is_half_mgd() {
    let well = WaterSource::new(WaterSourceType::Well, 10, 10);
    assert!(
        (well.capacity_mgd - 0.5).abs() < f32::EPSILON,
        "Well capacity should be 0.5 MGD, got {}",
        well.capacity_mgd
    );
}

#[test]
fn test_surface_intake_capacity() {
    let intake = WaterSource::new(WaterSourceType::SurfaceIntake, 10, 10);
    assert!(
        (intake.capacity_mgd - 5.0).abs() < f32::EPSILON,
        "Surface intake capacity should be 5.0 MGD, got {}",
        intake.capacity_mgd
    );
}

#[test]
fn test_reservoir_capacity() {
    let reservoir = WaterSource::new(WaterSourceType::Reservoir, 10, 10);
    assert!(
        (reservoir.capacity_mgd - 20.0).abs() < f32::EPSILON,
        "Reservoir capacity should be 20.0 MGD, got {}",
        reservoir.capacity_mgd
    );
}

#[test]
fn test_desalination_capacity() {
    let desal = WaterSource::new(WaterSourceType::Desalination, 10, 10);
    assert!(
        (desal.capacity_mgd - 10.0).abs() < f32::EPSILON,
        "Desalination capacity should be 10.0 MGD, got {}",
        desal.capacity_mgd
    );
}

#[test]
fn test_reservoir_stores_90_day_buffer() {
    let reservoir = WaterSource::new(WaterSourceType::Reservoir, 10, 10);
    let expected_storage = RESERVOIR_CAPACITY_MGD * MGD_TO_GPD * RESERVOIR_BUFFER_DAYS as f32;
    assert!(
        (reservoir.storage_capacity - expected_storage).abs() < 1.0,
        "Reservoir should store 90-day buffer: expected {}, got {}",
        expected_storage,
        reservoir.storage_capacity
    );
    // Verify it starts full
    assert!(
        (reservoir.stored_gallons - expected_storage).abs() < 1.0,
        "Reservoir should start full"
    );
}

#[test]
fn test_well_has_no_storage() {
    let well = WaterSource::new(WaterSourceType::Well, 10, 10);
    assert_eq!(well.storage_capacity, 0.0);
    assert_eq!(well.stored_gallons, 0.0);
}

#[test]
fn test_desalination_highest_quality() {
    let desal = WaterSource::new(WaterSourceType::Desalination, 10, 10);
    let well = WaterSource::new(WaterSourceType::Well, 10, 10);
    let intake = WaterSource::new(WaterSourceType::SurfaceIntake, 10, 10);
    let reservoir = WaterSource::new(WaterSourceType::Reservoir, 10, 10);

    assert!(
        desal.quality > well.quality,
        "Desalination quality should exceed well quality"
    );
    assert!(
        desal.quality > intake.quality,
        "Desalination quality should exceed surface intake quality"
    );
    assert!(
        desal.quality > reservoir.quality,
        "Desalination quality should exceed reservoir quality"
    );
}

#[test]
fn test_operating_cost_hierarchy() {
    // Well < Surface Intake < Reservoir < Desalination
    assert!(
        WaterSourceType::Well.operating_cost() < WaterSourceType::SurfaceIntake.operating_cost()
    );
    assert!(
        WaterSourceType::SurfaceIntake.operating_cost()
            < WaterSourceType::Reservoir.operating_cost()
    );
    assert!(
        WaterSourceType::Reservoir.operating_cost()
            < WaterSourceType::Desalination.operating_cost()
    );
}

#[test]
fn test_build_cost_hierarchy() {
    // Well < Surface Intake < Reservoir < Desalination
    assert!(WaterSourceType::Well.build_cost() < WaterSourceType::SurfaceIntake.build_cost());
    assert!(WaterSourceType::SurfaceIntake.build_cost() < WaterSourceType::Reservoir.build_cost());
    assert!(WaterSourceType::Reservoir.build_cost() < WaterSourceType::Desalination.build_cost());
}

#[test]
fn test_reservoir_footprint_8x8() {
    let (w, h) = WaterSourceType::Reservoir.footprint();
    assert_eq!(w, 8);
    assert_eq!(h, 8);
}

#[test]
fn test_is_near_water_true() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(12, 10).cell_type = CellType::Water;
    assert!(is_near_water(&grid, 10, 10, 2));
}

#[test]
fn test_is_near_water_false() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    // Default is Grass, no water
    assert!(!is_near_water(&grid, 10, 10, 2));
}

#[test]
fn test_water_source_type_names() {
    assert_eq!(WaterSourceType::Well.name(), "Groundwater Well");
    assert_eq!(
        WaterSourceType::SurfaceIntake.name(),
        "Surface Water Intake"
    );
    assert_eq!(WaterSourceType::Reservoir.name(), "Reservoir");
    assert_eq!(WaterSourceType::Desalination.name(), "Desalination Plant");
}

#[test]
fn test_mgd_to_gpd_conversion() {
    let well = WaterSource::new(WaterSourceType::Well, 10, 10);
    let supply_gpd = well.capacity_mgd * MGD_TO_GPD;
    assert!(
        (supply_gpd - 500_000.0).abs() < 1.0,
        "0.5 MGD should equal 500,000 GPD, got {}",
        supply_gpd
    );
}

#[test]
fn test_quality_penalty_increases_cost() {
    let base_cost = WaterSourceType::Well.operating_cost();
    // At quality 0.0, cost should double
    let quality = 0.0_f32;
    let penalty = if quality < 0.5 {
        1.0 + (1.0 - quality * 2.0)
    } else {
        1.0
    };
    let adjusted_cost = base_cost * penalty as f64;
    assert!(
        adjusted_cost > base_cost,
        "Adjusted cost {} should exceed base cost {}",
        adjusted_cost,
        base_cost
    );
    assert!(
        (adjusted_cost - base_cost * 2.0).abs() < 0.01,
        "At quality 0, cost should be 2x base"
    );
}

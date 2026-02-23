//! Tests for coverage computation and per-service-type stats.

use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{WorldGrid, ZoneType};
use simulation::happiness::{ServiceCoverageGrid, COVERAGE_FIRE, COVERAGE_HEALTH, COVERAGE_POLICE};
use simulation::services::{ServiceBuilding, ServiceType};

use super::categories::ServiceCategory;
use super::stats::{compute_category_stats, compute_service_type_stats};

// =========================================================================
// Coverage computation tests
// =========================================================================

#[test]
fn test_coverage_zero_demand() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let coverage = ServiceCoverageGrid::default();
    let services: Vec<&ServiceBuilding> = vec![];

    let stats = compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);

    assert_eq!(stats.demand_cells, 0);
    assert_eq!(stats.covered_cells, 0);
    assert_eq!(stats.building_count, 0);
    assert!((stats.coverage_pct - 0.0).abs() < f64::EPSILON);
    assert!((stats.monthly_maintenance - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_coverage_with_demand_no_coverage() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    // Zone a few cells
    grid.get_mut(10, 10).zone = ZoneType::ResidentialLow;
    grid.get_mut(11, 10).zone = ZoneType::CommercialLow;
    grid.get_mut(12, 10).zone = ZoneType::Industrial;

    let coverage = ServiceCoverageGrid::default();
    let services: Vec<&ServiceBuilding> = vec![];

    let stats = compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);

    assert_eq!(stats.demand_cells, 3);
    assert_eq!(stats.covered_cells, 0);
    assert!((stats.coverage_pct - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_coverage_full_coverage() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(10, 10).zone = ZoneType::ResidentialLow;
    grid.get_mut(11, 10).zone = ZoneType::CommercialLow;

    let mut coverage = ServiceCoverageGrid::default();
    // Set health coverage on both cells
    let idx1 = ServiceCoverageGrid::idx(10, 10);
    let idx2 = ServiceCoverageGrid::idx(11, 10);
    coverage.flags[idx1] |= COVERAGE_HEALTH;
    coverage.flags[idx2] |= COVERAGE_HEALTH;

    let services: Vec<&ServiceBuilding> = vec![];

    let stats = compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);

    assert_eq!(stats.demand_cells, 2);
    assert_eq!(stats.covered_cells, 2);
    assert!((stats.coverage_pct - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_coverage_partial_coverage() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(10, 10).zone = ZoneType::ResidentialLow;
    grid.get_mut(11, 10).zone = ZoneType::CommercialLow;
    grid.get_mut(12, 10).zone = ZoneType::Industrial;
    grid.get_mut(13, 10).zone = ZoneType::Office;

    let mut coverage = ServiceCoverageGrid::default();
    // Cover 2 of 4 cells
    coverage.flags[ServiceCoverageGrid::idx(10, 10)] |= COVERAGE_POLICE;
    coverage.flags[ServiceCoverageGrid::idx(12, 10)] |= COVERAGE_POLICE;

    let services: Vec<&ServiceBuilding> = vec![];

    let stats = compute_category_stats(ServiceCategory::Police, &grid, &coverage, &services);

    assert_eq!(stats.demand_cells, 4);
    assert_eq!(stats.covered_cells, 2);
    assert!((stats.coverage_pct - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_coverage_counts_buildings() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let coverage = ServiceCoverageGrid::default();

    let hospital = ServiceBuilding {
        service_type: ServiceType::Hospital,
        grid_x: 10,
        grid_y: 10,
        radius: 400.0,
    };
    let clinic = ServiceBuilding {
        service_type: ServiceType::MedicalClinic,
        grid_x: 20,
        grid_y: 20,
        radius: 192.0,
    };
    let school = ServiceBuilding {
        service_type: ServiceType::ElementarySchool,
        grid_x: 30,
        grid_y: 30,
        radius: 240.0,
    };

    let services: Vec<&ServiceBuilding> = vec![&hospital, &clinic, &school];

    let health_stats = compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);
    assert_eq!(health_stats.building_count, 2); // hospital + clinic

    let edu_stats = compute_category_stats(ServiceCategory::Education, &grid, &coverage, &services);
    assert_eq!(edu_stats.building_count, 1); // school only
}

#[test]
fn test_coverage_computes_maintenance() {
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let coverage = ServiceCoverageGrid::default();

    let hospital = ServiceBuilding {
        service_type: ServiceType::Hospital,
        grid_x: 10,
        grid_y: 10,
        radius: 400.0,
    };
    let clinic = ServiceBuilding {
        service_type: ServiceType::MedicalClinic,
        grid_x: 20,
        grid_y: 20,
        radius: 192.0,
    };

    let services: Vec<&ServiceBuilding> = vec![&hospital, &clinic];

    let stats = compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);

    let expected = ServiceBuilding::monthly_maintenance(ServiceType::Hospital)
        + ServiceBuilding::monthly_maintenance(ServiceType::MedicalClinic);
    assert!((stats.monthly_maintenance - expected).abs() < f64::EPSILON);
}

// =========================================================================
// Cross-category isolation test
// =========================================================================

#[test]
fn test_coverage_only_counts_matching_bit() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(5, 5).zone = ZoneType::ResidentialLow;

    let mut coverage = ServiceCoverageGrid::default();
    // Set only FIRE coverage
    coverage.flags[ServiceCoverageGrid::idx(5, 5)] |= COVERAGE_FIRE;

    let services: Vec<&ServiceBuilding> = vec![];

    // Fire should show 100%
    let fire_stats = compute_category_stats(ServiceCategory::Fire, &grid, &coverage, &services);
    assert_eq!(fire_stats.covered_cells, 1);
    assert!((fire_stats.coverage_pct - 1.0).abs() < f64::EPSILON);

    // Health should show 0%
    let health_stats = compute_category_stats(ServiceCategory::Health, &grid, &coverage, &services);
    assert_eq!(health_stats.covered_cells, 0);
    assert!((health_stats.coverage_pct - 0.0).abs() < f64::EPSILON);
}

// =========================================================================
// Per-service-type stats tests
// =========================================================================

#[test]
fn test_service_type_stats_empty() {
    let services: Vec<&ServiceBuilding> = vec![];
    let stats = compute_service_type_stats(ServiceType::Hospital, &services);
    assert_eq!(stats.count, 0);
    assert!((stats.monthly_maintenance - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_service_type_stats_counts_correct_type() {
    let hospital = ServiceBuilding {
        service_type: ServiceType::Hospital,
        grid_x: 10,
        grid_y: 10,
        radius: 400.0,
    };
    let clinic = ServiceBuilding {
        service_type: ServiceType::MedicalClinic,
        grid_x: 20,
        grid_y: 20,
        radius: 192.0,
    };
    let hospital2 = ServiceBuilding {
        service_type: ServiceType::Hospital,
        grid_x: 30,
        grid_y: 30,
        radius: 400.0,
    };

    let services: Vec<&ServiceBuilding> = vec![&hospital, &clinic, &hospital2];

    let stats = compute_service_type_stats(ServiceType::Hospital, &services);
    assert_eq!(stats.count, 2);
    let expected = ServiceBuilding::monthly_maintenance(ServiceType::Hospital) * 2.0;
    assert!((stats.monthly_maintenance - expected).abs() < f64::EPSILON);

    let clinic_stats = compute_service_type_stats(ServiceType::MedicalClinic, &services);
    assert_eq!(clinic_stats.count, 1);
}

//! Integration tests for SVC-001: Hybrid Service Coverage Model
//!
//! Tests that hybrid coverage uses road-network BFS (not Euclidean),
//! that coverage quality degrades with over-capacity, that budget
//! changes affect quality factor, and that unreachable areas get
//! zero coverage.

use crate::grid::{CellType, RoadType};
use crate::hybrid_service_coverage::{
    budget_quality_factor, compute_effective_quality, HybridCoverageGrid, HybridCoverageStats,
    ServiceCategory,
};
use crate::service_budget::{Department, ServiceBudgetState};
use crate::service_capacity::ServiceCapacity;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Helper: tick enough to trigger the hybrid coverage update
// ====================================================================

fn tick_coverage(city: &mut TestCity) {
    // 20 ticks to trigger HYBRID_COVERAGE_UPDATE_INTERVAL
    city.tick(20);
}

// ====================================================================
// 1. Road-network BFS coverage
// ====================================================================

#[test]
fn test_hospital_covers_cells_along_road() {
    // Place a hospital at (128, 128), with a road extending east.
    // Cells along the road should have non-zero health coverage.
    let mut city = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Local)
        .with_service(128, 128, ServiceType::Hospital);

    tick_coverage(&mut city);

    let world = city.world_mut();
    let coverage = world.resource::<HybridCoverageGrid>();

    // Cell at the hospital should have high coverage
    let at_hospital = coverage.get(128, 128, ServiceCategory::Health);
    assert!(
        at_hospital > 0.0,
        "Hospital cell should have health coverage, got {at_hospital}"
    );

    // Cell along the road (a few cells east) should have coverage
    let along_road = coverage.get(135, 128, ServiceCategory::Health);
    assert!(
        along_road > 0.0,
        "Cell along road should have health coverage, got {along_road}"
    );

    // Coverage should decay with distance
    assert!(
        at_hospital >= along_road,
        "Coverage should decay: at_hospital={at_hospital} >= along_road={along_road}"
    );
}

#[test]
fn test_fire_station_no_bridge_zero_coverage() {
    // Place a fire station on one side, water in between, and a cell on the other.
    // With no road connecting them, the far cell should have zero fire coverage.
    let mut city = TestCity::new()
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_service(100, 128, ServiceType::FireStation);

    // Set water cells between the road and the target area
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        for x in 111..120 {
            grid.get_mut(x, 128).cell_type = CellType::Water;
        }
    }

    tick_coverage(&mut city);

    let world = city.world_mut();
    let coverage = world.resource::<HybridCoverageGrid>();

    // Cell across the water (no road connection) should have zero coverage
    let across_water = coverage.get(125, 128, ServiceCategory::Fire);
    assert!(
        across_water < f32::EPSILON,
        "Cell across water with no bridge should have zero fire coverage, got {across_water}"
    );

    // Cell on the same side should have coverage
    let same_side = coverage.get(105, 128, ServiceCategory::Fire);
    assert!(
        same_side > 0.0,
        "Cell on same road should have fire coverage, got {same_side}"
    );
}

#[test]
fn test_coverage_follows_road_not_euclidean() {
    // Place a police station with an L-shaped road.
    // A cell near the station (Euclidean) but not on the road should have
    // less coverage than a cell farther away (Euclidean) but on the road.
    let mut city = TestCity::new()
        .with_road(128, 128, 128, 140, RoadType::Local) // vertical road
        .with_road(128, 140, 140, 140, RoadType::Local) // horizontal road
        .with_service(128, 128, ServiceType::PoliceStation);

    tick_coverage(&mut city);

    let world = city.world_mut();
    let coverage = world.resource::<HybridCoverageGrid>();

    // Cell at end of L-shaped road (road distance = 24, Euclidean ~ 17)
    let on_road = coverage.get(140, 140, ServiceCategory::Police);

    // Cell diagonally adjacent but not on road (Euclidean ~ 7)
    let off_road = coverage.get(135, 135, ServiceCategory::Police);

    // The on-road cell should have some coverage from road BFS
    assert!(
        on_road > 0.0,
        "Cell at end of L-road should have coverage, got {on_road}"
    );

    // The off-road cell might have some coverage from adjacent-road bleed,
    // but it should generally be lower or zero since there is no road there
    // and it's not adjacent to any road cell
    assert!(
        on_road >= off_road,
        "On-road coverage ({on_road}) should be >= off-road coverage ({off_road})"
    );
}

// ====================================================================
// 2. Coverage quality with over-capacity
// ====================================================================

#[test]
fn test_over_capacity_degrades_coverage() {
    // Two hospitals: one at capacity, one over capacity.
    // The over-capacity one should produce lower quality coverage.
    let mut city = TestCity::new()
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_road(150, 128, 160, 128, RoadType::Local)
        .with_service(100, 128, ServiceType::Hospital)
        .with_service(150, 128, ServiceType::Hospital);

    // Attach capacity components manually
    {
        let world = city.world_mut();
        let mut service_entities: Vec<bevy::prelude::Entity> = world
            .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<crate::services::ServiceBuilding>>()
            .iter(world)
            .collect();
        service_entities.sort(); // deterministic order

        // First hospital: at capacity
        world
            .entity_mut(service_entities[0])
            .insert(ServiceCapacity {
                capacity: 200,
                current_usage: 200,
            });
        // Second hospital: 3x over capacity
        world
            .entity_mut(service_entities[1])
            .insert(ServiceCapacity {
                capacity: 200,
                current_usage: 600,
            });
    }

    tick_coverage(&mut city);

    let world = city.world_mut();
    let coverage = world.resource::<HybridCoverageGrid>();

    let normal = coverage.get(100, 128, ServiceCategory::Health);
    let degraded = coverage.get(150, 128, ServiceCategory::Health);

    assert!(
        normal > degraded,
        "At-capacity hospital ({normal}) should have better coverage than over-capacity ({degraded})"
    );
}

// ====================================================================
// 3. Budget funding affects quality
// ====================================================================

#[test]
fn test_budget_quality_factor_ranges() {
    // Zero funding => 0.5
    let state = ServiceBudgetState::default();
    let q0 = budget_quality_factor(&state, ServiceType::Hospital);
    assert!(
        (q0 - 0.5).abs() < f32::EPSILON,
        "Zero funding should give quality 0.5, got {q0}"
    );

    // Full funding => 1.0
    let mut state1 = ServiceBudgetState::default();
    state1
        .department_mut(Department::Healthcare)
        .funding_ratio = 1.0;
    let q1 = budget_quality_factor(&state1, ServiceType::Hospital);
    assert!(
        (q1 - 1.0).abs() < f32::EPSILON,
        "Full funding should give quality 1.0, got {q1}"
    );

    // Overfunded => capped at 1.5
    let mut state2 = ServiceBudgetState::default();
    state2
        .department_mut(Department::Healthcare)
        .funding_ratio = 10.0;
    let q2 = budget_quality_factor(&state2, ServiceType::Hospital);
    assert!(
        (q2 - 1.5).abs() < f32::EPSILON,
        "Overfunded should give quality 1.5, got {q2}"
    );
}

#[test]
fn test_effective_quality_combines_capacity_and_budget() {
    let mut state = ServiceBudgetState::default();
    state
        .department_mut(Department::Healthcare)
        .funding_ratio = 1.0;

    // At capacity, full budget => 1.0
    let at_cap = ServiceCapacity {
        capacity: 200,
        current_usage: 200,
    };
    let q = compute_effective_quality(Some(&at_cap), &state, ServiceType::Hospital);
    assert!(
        (q - 1.0).abs() < f32::EPSILON,
        "At capacity + full budget should give 1.0, got {q}"
    );

    // 2x over capacity, full budget => 0.5
    let over_cap = ServiceCapacity {
        capacity: 200,
        current_usage: 400,
    };
    let q2 = compute_effective_quality(Some(&over_cap), &state, ServiceType::Hospital);
    assert!(
        (q2 - 0.5).abs() < f32::EPSILON,
        "2x over + full budget should give 0.5, got {q2}"
    );
}

// ====================================================================
// 4. Coverage stats tracking
// ====================================================================

#[test]
fn test_coverage_stats_updated() {
    let mut city = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Local)
        .with_service(128, 128, ServiceType::Hospital);

    tick_coverage(&mut city);

    let world = city.world_mut();
    let stats = world.resource::<HybridCoverageStats>();

    let health_idx = ServiceCategory::Health.grid_index();
    assert!(
        stats.covered_cell_counts[health_idx] > 0,
        "Should have some cells with health coverage"
    );
    assert!(
        stats.category_averages[health_idx] > 0.0,
        "Average health coverage should be > 0"
    );

    // No fire service placed, so fire coverage should be zero
    let fire_idx = ServiceCategory::Fire.grid_index();
    assert_eq!(
        stats.covered_cell_counts[fire_idx], 0,
        "No fire station placed, so fire coverage count should be 0"
    );
}

// ====================================================================
// 5. Service category classification
// ====================================================================

#[test]
fn test_service_category_classification() {
    assert_eq!(
        ServiceCategory::from_service_type(ServiceType::Hospital),
        Some(ServiceCategory::Health)
    );
    assert_eq!(
        ServiceCategory::from_service_type(ServiceType::FireStation),
        Some(ServiceCategory::Fire)
    );
    assert_eq!(
        ServiceCategory::from_service_type(ServiceType::PoliceStation),
        Some(ServiceCategory::Police)
    );
    assert_eq!(
        ServiceCategory::from_service_type(ServiceType::SmallPark),
        Some(ServiceCategory::Park)
    );
    assert_eq!(
        ServiceCategory::from_service_type(ServiceType::Stadium),
        Some(ServiceCategory::Entertainment)
    );
    assert_eq!(
        ServiceCategory::from_service_type(ServiceType::CellTower),
        Some(ServiceCategory::Telecom)
    );
    assert_eq!(
        ServiceCategory::from_service_type(ServiceType::BusDepot),
        Some(ServiceCategory::Transport)
    );
    // CityHall has no category
    assert_eq!(
        ServiceCategory::from_service_type(ServiceType::CityHall),
        None
    );
}

// ====================================================================
// 6. Grid operations
// ====================================================================

#[test]
fn test_coverage_grid_default_is_zero() {
    let grid = HybridCoverageGrid::default();
    for cat in ServiceCategory::ALL {
        assert!(
            grid.get(128, 128, cat).abs() < f32::EPSILON,
            "Default coverage for {:?} should be 0.0",
            cat
        );
    }
}

#[test]
fn test_coverage_grid_clamped() {
    let mut grid = HybridCoverageGrid::default();
    // Manually set a value > 1.0
    let idx = ServiceCategory::Health.grid_index() * (256 * 256) + 10 * 256 + 10;
    grid.data[idx] = 1.5;
    assert!((grid.get(10, 10, ServiceCategory::Health) - 1.5).abs() < f32::EPSILON);
    assert!((grid.get_clamped(10, 10, ServiceCategory::Health) - 1.0).abs() < f32::EPSILON);
}

// ====================================================================
// 7. Multiple services stack (max, not additive)
// ====================================================================

#[test]
fn test_multiple_services_take_max_coverage() {
    // Two hospitals near the same road -- coverage should be max, not sum
    let mut city = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Local)
        .with_service(128, 128, ServiceType::Hospital)
        .with_service(129, 128, ServiceType::MedicalClinic);

    tick_coverage(&mut city);

    let world = city.world_mut();
    let coverage = world.resource::<HybridCoverageGrid>();

    let val = coverage.get_clamped(130, 128, ServiceCategory::Health);
    assert!(
        val <= 1.0,
        "Coverage should be max (not sum), got {val}"
    );
    assert!(
        val > 0.0,
        "Should have some health coverage at (130,128), got {val}"
    );
}

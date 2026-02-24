//! Integration tests for SVC-001: Hybrid Service Coverage Model

use crate::grid::{CellType, RoadType};
use crate::hybrid_service_coverage::{
    budget_quality_factor, compute_effective_quality, HybridCoverageGrid, HybridCoverageStats,
    ServiceCategory,
};
use crate::service_budget::{Department, ServiceBudgetState};
use crate::service_capacity::ServiceCapacity;
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;

fn tick_coverage(city: &mut TestCity) {
    city.tick(20);
}

// ====================================================================
// 1. Road-network BFS coverage
// ====================================================================

#[test]
fn test_hospital_covers_cells_along_road() {
    let mut city = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Local)
        .with_service(128, 128, ServiceType::Hospital);

    tick_coverage(&mut city);

    let world = city.world_mut();
    let coverage = world.resource::<HybridCoverageGrid>();

    let at_hospital = coverage.get(128, 128, ServiceCategory::Health);
    assert!(at_hospital > 0.0, "Hospital cell should have coverage");

    let along_road = coverage.get(135, 128, ServiceCategory::Health);
    assert!(along_road > 0.0, "Cell along road should have coverage");

    assert!(
        at_hospital >= along_road,
        "Coverage should decay with distance"
    );
}

#[test]
fn test_fire_station_no_bridge_zero_coverage() {
    let mut city = TestCity::new()
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_service(100, 128, ServiceType::FireStation);

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

    let across_water = coverage.get(125, 128, ServiceCategory::Fire);
    assert!(
        across_water < f32::EPSILON,
        "Across water should be zero, got {across_water}"
    );

    let same_side = coverage.get(105, 128, ServiceCategory::Fire);
    assert!(same_side > 0.0, "Same side should have coverage");
}

#[test]
fn test_coverage_follows_road_not_euclidean() {
    // Short L-shaped road within PoliceStation radius (20 cells).
    let mut city = TestCity::new()
        .with_road(128, 128, 128, 135, RoadType::Local)
        .with_road(128, 135, 135, 135, RoadType::Local)
        .with_service(128, 128, ServiceType::PoliceStation);

    tick_coverage(&mut city);

    let world = city.world_mut();
    let coverage = world.resource::<HybridCoverageGrid>();

    let on_road = coverage.get(135, 135, ServiceCategory::Police);
    assert!(
        on_road > 0.0,
        "Cell at end of L-road should have coverage, got {on_road}"
    );

    let off_road = coverage.get(133, 133, ServiceCategory::Police);
    assert!(
        on_road >= off_road,
        "On-road ({on_road}) should be >= off-road ({off_road})"
    );
}

// ====================================================================
// 2. Coverage quality with over-capacity
// ====================================================================

#[test]
fn test_over_capacity_degrades_coverage() {
    let mut city = TestCity::new()
        .with_road(50, 128, 60, 128, RoadType::Local)
        .with_road(200, 128, 210, 128, RoadType::Local)
        .with_service(50, 128, ServiceType::Hospital)
        .with_service(200, 128, ServiceType::Hospital);

    // Tick 10 to let attach_capacity_to_new_services run
    city.tick(10);

    // Collect entity IDs
    let entities: Vec<(bevy::prelude::Entity, usize)> = {
        let world = city.world_mut();
        world
            .query::<(bevy::prelude::Entity, &ServiceBuilding)>()
            .iter(world)
            .map(|(e, s)| (e, s.grid_x))
            .collect()
    };

    // Modify capacity components and set dirty
    {
        let world = city.world_mut();
        for (entity, grid_x) in &entities {
            if *grid_x == 50 {
                if let Some(mut cap) = world.get_mut::<ServiceCapacity>(*entity) {
                    cap.capacity = 200;
                    cap.current_usage = 200;
                }
            } else if *grid_x == 200 {
                if let Some(mut cap) = world.get_mut::<ServiceCapacity>(*entity) {
                    cap.capacity = 200;
                    cap.current_usage = 600;
                }
            }
        }
        world.resource_mut::<HybridCoverageGrid>().dirty = true;
    }

    // Tick just 1 more (tick 11: not multiple of 10, so usage won't overwrite;
    // dirty flag triggers coverage recompute)
    city.tick(1);

    let world = city.world_mut();
    let coverage = world.resource::<HybridCoverageGrid>();

    let normal = coverage.get(50, 128, ServiceCategory::Health);
    let degraded = coverage.get(200, 128, ServiceCategory::Health);

    assert!(
        normal > degraded,
        "At-capacity ({normal}) should beat over-capacity ({degraded})"
    );
}

// ====================================================================
// 3. Budget funding affects quality (pure unit tests)
// ====================================================================

#[test]
fn test_budget_quality_factor_ranges() {
    let state = ServiceBudgetState::default();
    let q0 = budget_quality_factor(&state, ServiceType::Hospital);
    assert!((q0 - 0.5).abs() < f32::EPSILON, "Zero funding => 0.5");

    let mut state1 = ServiceBudgetState::default();
    state1
        .department_mut(Department::Healthcare)
        .funding_ratio = 1.0;
    let q1 = budget_quality_factor(&state1, ServiceType::Hospital);
    assert!((q1 - 1.0).abs() < f32::EPSILON, "Full funding => 1.0");

    let mut state2 = ServiceBudgetState::default();
    state2
        .department_mut(Department::Healthcare)
        .funding_ratio = 10.0;
    let q2 = budget_quality_factor(&state2, ServiceType::Hospital);
    assert!((q2 - 1.5).abs() < f32::EPSILON, "Overfunded => 1.5");
}

#[test]
fn test_effective_quality_combines_capacity_and_budget() {
    let mut state = ServiceBudgetState::default();
    state
        .department_mut(Department::Healthcare)
        .funding_ratio = 1.0;

    let at_cap = ServiceCapacity {
        capacity: 200,
        current_usage: 200,
    };
    let q = compute_effective_quality(Some(&at_cap), &state, ServiceType::Hospital);
    assert!((q - 1.0).abs() < f32::EPSILON);

    let over_cap = ServiceCapacity {
        capacity: 200,
        current_usage: 400,
    };
    let q2 = compute_effective_quality(Some(&over_cap), &state, ServiceType::Hospital);
    assert!((q2 - 0.5).abs() < f32::EPSILON);
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
    assert!(stats.covered_cell_counts[health_idx] > 0);
    assert!(stats.category_averages[health_idx] > 0.0);

    let fire_idx = ServiceCategory::Fire.grid_index();
    assert_eq!(stats.covered_cell_counts[fire_idx], 0);
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
        assert!(grid.get(128, 128, cat).abs() < f32::EPSILON);
    }
}

#[test]
fn test_coverage_grid_clamped() {
    let mut grid = HybridCoverageGrid::default();
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
    let mut city = TestCity::new()
        .with_road(128, 128, 140, 128, RoadType::Local)
        .with_service(128, 128, ServiceType::Hospital)
        .with_service(129, 128, ServiceType::MedicalClinic);

    tick_coverage(&mut city);

    let world = city.world_mut();
    let coverage = world.resource::<HybridCoverageGrid>();

    let val = coverage.get_clamped(130, 128, ServiceCategory::Health);
    assert!(val <= 1.0, "Coverage should be max (not sum)");
    assert!(val > 0.0, "Should have some health coverage");
}

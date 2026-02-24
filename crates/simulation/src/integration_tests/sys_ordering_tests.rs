//! TEST-017: Integration Tests for System Ordering Dependencies
//!
//! Verifies that system ordering constraints are correct:
//! - Service coverage updates before happiness reads coverage (chained)
//! - Traffic system produces density data consumed by happiness
//! - Hospital placement correctly populates the ServiceCoverageGrid
//!
//! Key system ordering (all in `SimulationSet::Simulation`):
//!   - `update_service_coverage` → `update_happiness` (explicitly chained)
//!   - `update_traffic_density` runs every 5 ticks, clears + repopulates
//!   - `update_happiness` reads `TrafficGrid` and `ServiceCoverageGrid`
//!
//! The service coverage → happiness chain is testable end-to-end because
//! `update_service_coverage` uses `Added<ServiceBuilding>` change detection,
//! so spawning a hospital right before a happiness tick proves the chain.
//!
//! The traffic → happiness dependency is verified by confirming that
//! `TrafficGrid` density data is accessible and that the `congestion_level`
//! method produces the values used by the happiness formula.

use crate::citizen::{CitizenDetails, Needs};
use crate::grid::ZoneType;
use crate::happiness::{ServiceCoverageGrid, COVERAGE_HEALTH};
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;
use crate::traffic::TrafficGrid;
use crate::utilities::UtilityType;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Ticks between happiness recalculations.
const HAPPINESS_TICKS: u32 = crate::happiness::HAPPINESS_UPDATE_INTERVAL as u32;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Query the happiness of the first citizen found.
fn first_citizen_happiness(city: &mut TestCity) -> f32 {
    let world = city.world_mut();
    world
        .query::<&CitizenDetails>()
        .iter(world)
        .next()
        .expect("expected at least one citizen")
        .happiness
}

/// Set needs and health on all citizens to stable values.
fn stabilize_needs(city: &mut TestCity) {
    let world = city.world_mut();
    let mut q = world.query::<(&mut Needs, &mut CitizenDetails)>();
    for (mut needs, mut details) in q.iter_mut(world) {
        needs.hunger = 80.0;
        needs.energy = 80.0;
        needs.social = 80.0;
        needs.fun = 80.0;
        needs.comfort = 80.0;
        details.health = 90.0;
    }
}

/// Build a city with an unemployed citizen and utilities (power + water).
/// Happiness is above 0 (has utilities) but below 100 (unemployed, no
/// services), leaving room for measuring positive and negative deltas.
fn city_with_unemployed_citizen(home: (usize, usize)) -> TestCity {
    TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower)
}

// ====================================================================
// 1. TrafficGrid resource exists and congestion math is correct
// ====================================================================

/// Verify the TrafficGrid resource is initialized in the ECS world.
#[test]
fn test_traffic_grid_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<TrafficGrid>();
}

/// Verify that congestion_level returns expected values from density.
#[test]
fn test_traffic_grid_congestion_level_in_ecs() {
    let mut city = TestCity::new();

    // Set density and verify congestion_level returns expected value.
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(50, 50, 10); // congestion_level = 10/20 = 0.5
    }

    let traffic = city.resource::<TrafficGrid>();
    let level = traffic.congestion_level(50, 50);
    assert!(
        (level - 0.5).abs() < 0.01,
        "congestion_level should be 0.5 for density 10, got {level}"
    );
}

/// Verify TrafficGrid starts at zero density (no initial congestion).
#[test]
fn test_traffic_grid_starts_at_zero() {
    let city = TestCity::new();
    let traffic = city.resource::<TrafficGrid>();

    // Check several random positions — all should be zero.
    for (x, y) in [(50, 50), (100, 100), (200, 200), (0, 0)] {
        let density = traffic.get(x, y);
        assert_eq!(
            density, 0,
            "TrafficGrid should start at zero density at ({x},{y}), got {density}"
        );
    }
}

/// Verify that after ticking, the traffic system clears density for
/// cells with no commuting citizens (confirming the system runs).
#[test]
fn test_traffic_system_clears_density_on_tick() {
    let mut city = TestCity::new();

    // Inject density manually.
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(50, 50, 42);
    }

    // After 5 ticks (traffic update interval), density should be cleared
    // because there are no commuting citizens.
    city.tick(5);

    let traffic = city.resource::<TrafficGrid>();
    let density = traffic.get(50, 50);
    assert_eq!(
        density, 0,
        "Traffic system should clear density when no citizens are commuting, got {density}"
    );
}

// ====================================================================
// 2. Hospital placement -> coverage grid has health flag
// ====================================================================

/// Verify that placing a hospital populates the ServiceCoverageGrid
/// with the COVERAGE_HEALTH flag at the hospital's position.
#[test]
fn test_hospital_placement_sets_health_coverage_flag() {
    let pos = (128, 128);
    let mut city = TestCity::new().with_service(pos.0, pos.1, ServiceType::Hospital);

    city.tick_slow_cycle();

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(pos.0, pos.1);
    assert!(
        cov.flags[idx] & COVERAGE_HEALTH != 0,
        "Hospital at ({},{}) should set COVERAGE_HEALTH flag",
        pos.0,
        pos.1
    );
}

/// Verify health coverage extends to cells within the hospital's radius.
#[test]
fn test_hospital_coverage_extends_to_nearby_cells() {
    let pos = (128, 128);
    let mut city = TestCity::new().with_service(pos.0, pos.1, ServiceType::Hospital);

    city.tick_slow_cycle();

    let cov = city.resource::<ServiceCoverageGrid>();

    // Hospital radius = 25 * CELL_SIZE = 400.0 -> 25 cells
    // 10 cells away = 160.0 < 400.0 => covered.
    let idx_near = ServiceCoverageGrid::idx(138, 128);
    assert!(
        cov.flags[idx_near] & COVERAGE_HEALTH != 0,
        "Cell 10 cells away from hospital should have health coverage"
    );

    // 24 cells away = 384.0 < 400.0 => still covered.
    let idx_edge = ServiceCoverageGrid::idx(152, 128);
    assert!(
        cov.flags[idx_edge] & COVERAGE_HEALTH != 0,
        "Cell 24 cells away from hospital should have health coverage"
    );
}

/// Verify that cells outside the hospital's radius do NOT have coverage.
#[test]
fn test_hospital_coverage_absent_outside_radius() {
    let pos = (128, 128);
    let mut city = TestCity::new().with_service(pos.0, pos.1, ServiceType::Hospital);

    city.tick_slow_cycle();

    let cov = city.resource::<ServiceCoverageGrid>();

    // 26 cells = 416.0 > 400.0 => outside.
    let idx_outside = ServiceCoverageGrid::idx(154, 128);
    assert!(
        cov.flags[idx_outside] & COVERAGE_HEALTH == 0,
        "Cell 26 cells beyond hospital should NOT have health coverage"
    );
}

// ====================================================================
// 3. Service coverage available to happiness system same tick
// ====================================================================

/// When a hospital is placed before ticking, the happiness system should
/// reflect the health coverage bonus without requiring an extra tick.
/// This verifies update_service_coverage runs before update_happiness
/// within the same FixedUpdate pass (they are chained in HappinessPlugin).
#[test]
fn test_service_coverage_available_to_happiness_same_tick() {
    let home = (100, 100);

    // City WITHOUT hospital — baseline happiness.
    let mut city_no_hosp = city_with_unemployed_citizen(home);
    city_no_hosp.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_no_hosp);
    city_no_hosp.tick(1);
    let happiness_no_hospital = first_citizen_happiness(&mut city_no_hosp);

    // City WITH hospital placed before any ticks.
    let mut city_with_hosp = city_with_unemployed_citizen(home)
        .with_service(home.0, home.1, ServiceType::Hospital);
    city_with_hosp.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_with_hosp);
    city_with_hosp.tick(1);
    let happiness_with_hospital = first_citizen_happiness(&mut city_with_hosp);

    assert!(
        happiness_no_hospital > 0.0,
        "Baseline happiness should be positive. Got {happiness_no_hospital}"
    );
    assert!(
        happiness_with_hospital > happiness_no_hospital,
        "Hospital coverage should boost happiness on the same tick. \
         Without={happiness_no_hospital}, With={happiness_with_hospital}"
    );
}

/// Verify that dynamically spawning a hospital mid-simulation makes its
/// coverage bonus appear at the next happiness tick.
#[test]
fn test_dynamic_hospital_spawn_reflected_in_happiness() {
    let home = (100, 100);
    let mut city = city_with_unemployed_citizen(home);

    // Get baseline happiness (no hospital).
    city.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city);
    city.tick(1);
    let baseline = first_citizen_happiness(&mut city);

    // Dynamically spawn a hospital at the citizen's home.
    {
        let radius = ServiceBuilding::coverage_radius(ServiceType::Hospital);
        city.world_mut().spawn(ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: home.0,
            grid_y: home.1,
            radius,
        });
    }

    // Advance to the next happiness tick.
    city.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city);
    city.tick(1);
    let with_hospital = first_citizen_happiness(&mut city);

    assert!(
        baseline > 0.0,
        "Baseline should be positive. Got {baseline}"
    );
    assert!(
        with_hospital > baseline,
        "Dynamically spawned hospital should increase happiness. \
         Baseline={baseline}, With hospital={with_hospital}"
    );
}

/// Verify the chained ordering: update_service_coverage -> update_happiness.
/// We spawn a hospital exactly one tick before happiness fires and confirm
/// the coverage bonus is reflected in that same happiness calculation.
#[test]
fn test_coverage_and_happiness_chained_within_single_tick() {
    let home = (100, 100);
    let mut city = city_with_unemployed_citizen(home);

    // Run to tick HAPPINESS_TICKS-1.
    city.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city);
    let baseline = first_citizen_happiness(&mut city);

    // Spawn hospital right before the happiness tick.
    {
        let radius = ServiceBuilding::coverage_radius(ServiceType::Hospital);
        city.world_mut().spawn(ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: home.0,
            grid_y: home.1,
            radius,
        });
    }

    // Tick once — update_service_coverage detects Added<ServiceBuilding>
    // and computes coverage, THEN update_happiness reads it.
    stabilize_needs(&mut city);
    city.tick(1);
    let with_coverage = first_citizen_happiness(&mut city);

    // Verify the coverage grid was updated.
    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(home.0, home.1);
    assert!(
        cov.flags[idx] & COVERAGE_HEALTH != 0,
        "Coverage grid should have health flag after single tick with new hospital"
    );

    assert!(
        with_coverage > baseline,
        "Happiness should reflect hospital coverage within the same tick. \
         Baseline={baseline}, With coverage={with_coverage}"
    );
}

// ====================================================================
// 4. Multiple service types reflected in happiness same tick
// ====================================================================

/// Verify that placing multiple service buildings (hospital + police + park)
/// all contribute to happiness within the same tick window, confirming
/// that service coverage for all types propagates before happiness reads.
#[test]
fn test_multiple_services_reflected_in_happiness_same_tick() {
    let home = (100, 100);

    // Baseline: no services.
    let mut city_none = city_with_unemployed_citizen(home);
    city_none.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_none);
    city_none.tick(1);
    let happiness_none = first_citizen_happiness(&mut city_none);

    // With just hospital.
    let mut city_hosp = city_with_unemployed_citizen(home)
        .with_service(home.0, home.1, ServiceType::Hospital);
    city_hosp.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_hosp);
    city_hosp.tick(1);
    let happiness_hosp = first_citizen_happiness(&mut city_hosp);

    // With multiple services.
    let mut city_all = city_with_unemployed_citizen(home)
        .with_service(home.0, home.1, ServiceType::Hospital)
        .with_service(home.0, home.1, ServiceType::PoliceStation)
        .with_service(home.0, home.1, ServiceType::SmallPark);
    city_all.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_all);
    city_all.tick(1);
    let happiness_all = first_citizen_happiness(&mut city_all);

    assert!(
        happiness_none > 0.0,
        "Baseline should be positive. Got {happiness_none}"
    );
    assert!(
        happiness_hosp > happiness_none,
        "Hospital should boost happiness. \
         None={happiness_none}, Hospital={happiness_hosp}"
    );
    assert!(
        happiness_all > happiness_hosp,
        "Multiple services should provide more than hospital alone. \
         Hospital={happiness_hosp}, All={happiness_all}"
    );
}

// ====================================================================
// 5. Coverage grid tracks service changes across ticks
// ====================================================================

/// Verify that the coverage grid is recalculated when new services are
/// added on different ticks, not just on the initial tick.
#[test]
fn test_coverage_grid_updates_on_subsequent_service_additions() {
    let pos = (128, 128);
    let mut city = TestCity::new();

    // Initially no coverage.
    city.tick_slow_cycle();
    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(pos.0, pos.1);
    assert_eq!(
        cov.flags[idx] & COVERAGE_HEALTH,
        0,
        "No coverage before any service is placed"
    );

    // Add a hospital dynamically.
    {
        let radius = ServiceBuilding::coverage_radius(ServiceType::Hospital);
        city.world_mut().spawn(ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: pos.0,
            grid_y: pos.1,
            radius,
        });
    }

    // After ticking, coverage should appear.
    city.tick_slow_cycle();
    let cov = city.resource::<ServiceCoverageGrid>();
    assert!(
        cov.flags[idx] & COVERAGE_HEALTH != 0,
        "Health coverage should appear after dynamically adding hospital"
    );
}

/// Verify that adding a service on tick N makes coverage available to
/// the happiness system on tick N (same-tick availability).
#[test]
fn test_same_tick_service_coverage_not_delayed() {
    let home = (100, 100);
    let mut city = city_with_unemployed_citizen(home);

    // Run 2 full happiness cycles to ensure everything is settled.
    city.tick(HAPPINESS_TICKS * 2);
    stabilize_needs(&mut city);

    // Record happiness before adding hospital.
    city.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city);
    city.tick(1);
    let before = first_citizen_happiness(&mut city);

    // Add hospital and immediately run another happiness cycle.
    {
        let radius = ServiceBuilding::coverage_radius(ServiceType::Hospital);
        city.world_mut().spawn(ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: home.0,
            grid_y: home.1,
            radius,
        });
    }
    city.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city);
    city.tick(1);
    let after = first_citizen_happiness(&mut city);

    assert!(
        after > before,
        "Service coverage should be available without delay. \
         Before={before}, After={after}"
    );
}

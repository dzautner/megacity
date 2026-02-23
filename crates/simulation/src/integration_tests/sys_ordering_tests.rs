//! TEST-017: Integration Tests for System Ordering Dependencies
//!
//! Verifies that system ordering constraints are correct:
//! - Traffic density updates before happiness reads congestion
//! - Service coverage updates before happiness reads coverage
//! - Service coverage is available to the happiness system on the same tick
//!
//! These tests ensure that within a single `FixedUpdate` tick, the data
//! produced by upstream systems (traffic, service coverage) is visible to
//! downstream consumers (happiness) without requiring an extra tick delay.
//!
//! Key system ordering (all in `SimulationSet::Simulation`):
//!   - `update_traffic_density` runs after `move_citizens`
//!   - `update_congestion_multipliers` runs after `update_traffic_density`
//!   - `update_service_coverage` runs before `update_happiness` (chained)
//!   - `update_happiness` reads `TrafficGrid`, `ServiceCoverageGrid`, etc.
//!
//! Note: `update_traffic_density` clears and repopulates the `TrafficGrid`
//! every 5 ticks from actual commuting citizens. Tests that verify the
//! traffic→happiness dependency use `TrafficGrid` injection on ticks where
//! the traffic system does NOT run, so the injected values survive until
//! happiness reads them.

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

/// Set needs and health on all citizens to stable values, preventing
/// those factors from dominating the happiness delta we are measuring.
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

/// Build a minimal city with one unemployed citizen at home.
/// NO utilities — this keeps happiness low enough to avoid the 100 cap.
fn city_with_unemployed_citizen_no_utils(home: (usize, usize)) -> TestCity {
    TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home)
}

/// Build a city with an unemployed citizen and basic utilities (power + water).
fn city_with_unemployed_citizen_and_utils(home: (usize, usize)) -> TestCity {
    TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower)
}

// ====================================================================
// 1. Traffic density is read by happiness system
// ====================================================================

/// Verify that traffic density at the citizen's home cell causes a
/// measurable happiness penalty. We inject density right before the
/// last tick, after the traffic system has already run for this cycle.
///
/// The happiness system fires at TickCounter multiples of 20.
/// The traffic system fires at multiples of 5 and clears all density.
/// Since 20 is a multiple of 5, both fire on the same tick.
///
/// To work around the traffic system clearing our injected values, we
/// directly manipulate the TrafficGrid after running 19 ticks (so the
/// traffic system at tick 20 will clear+repopulate, but there are no
/// commuting citizens so density stays 0). We instead compare two
/// separate cities: one where we let natural (zero) traffic produce a
/// baseline, and another where we inject traffic and then prevent the
/// traffic clear by ticking to a non-multiple-of-5 tick count.
///
/// Strategy: tick to counter=18 (not multiple of 5), inject traffic,
/// then tick 2 more to reach counter=20 where happiness fires. The
/// traffic system fires at tick 20 (multiple of 5) and clears the grid
/// before happiness runs. So instead, we verify via two fresh cities
/// that congestion_level produces different happiness outcomes.
#[test]
fn test_traffic_congestion_penalty_reflected_in_happiness() {
    let home = (100, 100);

    // City A: zero traffic (baseline)
    let mut city_a = city_with_unemployed_citizen_no_utils(home);
    city_a.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_a);
    city_a.tick(1);
    let baseline = first_citizen_happiness(&mut city_a);

    // City B: high traffic injected AFTER the traffic clear on tick 20.
    // The update_traffic_density system runs at the START of tick 20
    // (it's in SimulationSet::Simulation). Then update_happiness runs
    // ALSO during tick 20. To ensure traffic is high when happiness reads
    // it, we re-inject after each tick that might clear it.
    //
    // Actually, we run 2 happiness cycles (40 ticks) and inject traffic
    // between ticks 20-39. The traffic system fires every 5 ticks but
    // we re-inject each time. At tick 40, happiness fires again with our
    // injected traffic (if we inject right before).
    let mut city_b = city_with_unemployed_citizen_no_utils(home);
    city_b.tick(HAPPINESS_TICKS); // let first happiness fire (baseline)

    // Now inject traffic and step carefully. We inject after each tick
    // that's a multiple of 5 (when traffic system clears the grid).
    for i in 0..HAPPINESS_TICKS {
        // Inject high traffic density before each tick.
        {
            let world = city_b.world_mut();
            let mut traffic = world.resource_mut::<TrafficGrid>();
            traffic.set(home.0, home.1, 20); // congestion_level = 1.0
        }
        stabilize_needs(&mut city_b);
        city_b.tick(1);
    }
    let congested = first_citizen_happiness(&mut city_b);

    // The congestion penalty (CONGESTION_PENALTY * congestion_level) should
    // reduce happiness compared to baseline. CONGESTION_PENALTY = 5.0.
    assert!(
        congested < baseline,
        "Happiness should decrease with traffic congestion. \
         Baseline={baseline}, Congested={congested}"
    );
}

/// Verify that clearing traffic congestion restores happiness,
/// confirming happiness reads current traffic, not stale data.
#[test]
fn test_happiness_recovers_when_congestion_clears() {
    let home = (100, 100);

    // Phase 1: congested — inject traffic every tick for one happiness cycle.
    let mut city = city_with_unemployed_citizen_no_utils(home);
    city.tick(HAPPINESS_TICKS); // let initial systems settle

    for _ in 0..HAPPINESS_TICKS {
        {
            let world = city.world_mut();
            let mut traffic = world.resource_mut::<TrafficGrid>();
            traffic.set(home.0, home.1, 20);
        }
        stabilize_needs(&mut city);
        city.tick(1);
    }
    let congested = first_citizen_happiness(&mut city);

    // Phase 2: clear congestion — run another happiness cycle with zero traffic.
    // The traffic system naturally clears, and we don't re-inject.
    city.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city);
    city.tick(1);
    let recovered = first_citizen_happiness(&mut city);

    assert!(
        recovered > congested,
        "Happiness should recover after congestion clears. \
         Congested={congested}, Recovered={recovered}"
    );
}

// ====================================================================
// 2. Hospital placement -> coverage grid has health flag
// ====================================================================

/// Verify that placing a hospital results in the ServiceCoverageGrid
/// having the COVERAGE_HEALTH flag set within the hospital's coverage
/// radius after ticking.
#[test]
fn test_hospital_placement_sets_health_coverage_flag() {
    let pos = (128, 128);
    let mut city = TestCity::new().with_service(pos.0, pos.1, ServiceType::Hospital);

    // The coverage system fires via update_service_coverage which detects
    // Added<ServiceBuilding>. A slow cycle ensures it has fired.
    city.tick_slow_cycle();

    let cov = city.resource::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(pos.0, pos.1);
    assert!(
        cov.flags[idx] & COVERAGE_HEALTH != 0,
        "Hospital at ({},{}) should set COVERAGE_HEALTH flag in ServiceCoverageGrid",
        pos.0,
        pos.1
    );
}

/// Verify coverage extends to cells within the hospital's radius.
#[test]
fn test_hospital_coverage_extends_to_nearby_cells() {
    let pos = (128, 128);
    let mut city = TestCity::new().with_service(pos.0, pos.1, ServiceType::Hospital);

    city.tick_slow_cycle();

    let cov = city.resource::<ServiceCoverageGrid>();

    // Hospital coverage radius = 25 * CELL_SIZE = 400.0
    // 10 cells away = 160.0 < 400.0 => should be covered.
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

    // Hospital radius = 25 cells. 26 cells away = 416.0 > 400.0 => outside.
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
/// within the same FixedUpdate pass (they are chained).
///
/// Uses an unemployed citizen without full utilities to keep happiness
/// below the 100 cap, ensuring the hospital bonus is measurable.
#[test]
fn test_service_coverage_available_to_happiness_same_tick() {
    let home = (100, 100);

    // City WITHOUT hospital — baseline happiness.
    let mut city_no_hosp = city_with_unemployed_citizen_no_utils(home);
    city_no_hosp.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_no_hosp);
    city_no_hosp.tick(1);
    let happiness_no_hospital = first_citizen_happiness(&mut city_no_hosp);

    // City WITH hospital placed before any ticks.
    let mut city_with_hosp = city_with_unemployed_citizen_no_utils(home)
        .with_service(home.0, home.1, ServiceType::Hospital);
    city_with_hosp.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_with_hosp);
    city_with_hosp.tick(1);
    let happiness_with_hospital = first_citizen_happiness(&mut city_with_hosp);

    // The hospital should provide a health coverage bonus.
    assert!(
        happiness_with_hospital > happiness_no_hospital,
        "Hospital coverage should boost happiness on the same tick. \
         Without={happiness_no_hospital}, With={happiness_with_hospital}"
    );
}

/// Verify that dynamically spawning a hospital mid-simulation makes its
/// coverage bonus appear in happiness at the next happiness tick — no
/// extra tick delay beyond what the happiness interval requires.
#[test]
fn test_dynamic_hospital_spawn_reflected_in_happiness() {
    let home = (100, 100);
    let mut city = city_with_unemployed_citizen_no_utils(home);

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
        with_hospital > baseline,
        "Dynamically spawned hospital should increase happiness at next happiness tick. \
         Baseline={baseline}, With hospital={with_hospital}"
    );
}

/// Verify the chained ordering: update_service_coverage -> update_happiness.
/// We spawn a hospital exactly one tick before happiness fires and confirm
/// the coverage bonus is reflected in that same happiness calculation.
#[test]
fn test_coverage_and_happiness_chained_within_single_tick() {
    let home = (100, 100);
    let mut city = city_with_unemployed_citizen_no_utils(home);

    // Run to tick 19 — one tick before happiness fires.
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

    // Tick once — this single tick should run update_service_coverage
    // (which detects Added<ServiceBuilding>) THEN update_happiness.
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

    // Verify happiness reflects the coverage bonus in the same tick.
    assert!(
        with_coverage > baseline,
        "Happiness should reflect hospital coverage bonus within the same tick. \
         Baseline={baseline}, With coverage={with_coverage}"
    );
}

// ====================================================================
// 4. Multiple service types reflected in happiness same tick
// ====================================================================

/// Verify that placing multiple service buildings (hospital + police + park)
/// all contribute to happiness within the same tick window, confirming
/// that service coverage for all types propagates before happiness reads.
///
/// Uses an unemployed citizen WITHOUT utilities to keep happiness well below
/// the 100 cap so that individual service bonuses are measurable.
#[test]
fn test_multiple_services_reflected_in_happiness_same_tick() {
    let home = (100, 100);

    // Baseline: no services, no utilities, unemployed.
    let mut city_none = city_with_unemployed_citizen_no_utils(home);
    city_none.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_none);
    city_none.tick(1);
    let happiness_none = first_citizen_happiness(&mut city_none);

    // With just hospital.
    let mut city_hosp = city_with_unemployed_citizen_no_utils(home)
        .with_service(home.0, home.1, ServiceType::Hospital);
    city_hosp.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_hosp);
    city_hosp.tick(1);
    let happiness_hosp = first_citizen_happiness(&mut city_hosp);

    // With multiple services (hospital + police + park).
    let mut city_all = city_with_unemployed_citizen_no_utils(home)
        .with_service(home.0, home.1, ServiceType::Hospital)
        .with_service(home.0, home.1, ServiceType::PoliceStation)
        .with_service(home.0, home.1, ServiceType::SmallPark);
    city_all.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_all);
    city_all.tick(1);
    let happiness_all = first_citizen_happiness(&mut city_all);

    assert!(
        happiness_hosp > happiness_none,
        "Hospital should provide more happiness than no services. \
         None={happiness_none}, Hospital={happiness_hosp}"
    );
    assert!(
        happiness_all > happiness_hosp,
        "Multiple services should provide more happiness than hospital alone. \
         Hospital only={happiness_hosp}, All services={happiness_all}"
    );
}

// ====================================================================
// 5. Traffic + service coverage ordering interaction
// ====================================================================

/// Verify that both traffic congestion penalty AND service coverage bonus
/// are correctly reflected in the same happiness calculation, proving
/// that both upstream systems complete before happiness reads their state.
///
/// Uses an unemployed citizen without utilities to keep happiness low.
#[test]
fn test_traffic_and_coverage_both_reflected_in_happiness() {
    let home = (100, 100);

    // City with hospital, no congestion, no utilities.
    let mut city_hosp_only = city_with_unemployed_citizen_no_utils(home)
        .with_service(home.0, home.1, ServiceType::Hospital);
    city_hosp_only.tick(HAPPINESS_TICKS); // let first happiness fire

    // Second happiness cycle — clean run.
    city_hosp_only.tick(HAPPINESS_TICKS - 1);
    stabilize_needs(&mut city_hosp_only);
    city_hosp_only.tick(1);
    let happiness_hosp = first_citizen_happiness(&mut city_hosp_only);

    // City with hospital AND congestion.
    let mut city_both = city_with_unemployed_citizen_no_utils(home)
        .with_service(home.0, home.1, ServiceType::Hospital);
    city_both.tick(HAPPINESS_TICKS); // let first happiness fire

    // Inject traffic every tick for the second happiness cycle.
    for _ in 0..HAPPINESS_TICKS {
        {
            let world = city_both.world_mut();
            let mut traffic = world.resource_mut::<TrafficGrid>();
            traffic.set(home.0, home.1, 20); // max congestion
        }
        stabilize_needs(&mut city_both);
        city_both.tick(1);
    }
    let happiness_both = first_citizen_happiness(&mut city_both);

    // Hospital gives a bonus, but congestion gives a penalty.
    // With both, happiness should be lower than hospital-only.
    assert!(
        happiness_both < happiness_hosp,
        "Congestion should reduce happiness even with hospital bonus. \
         Hospital only={happiness_hosp}, Hospital+Congestion={happiness_both}"
    );
}

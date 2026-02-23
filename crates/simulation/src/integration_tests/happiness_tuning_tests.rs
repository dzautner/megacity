//! Integration tests for happiness formula tuning (issue #552).
//!
//! Tests that diminishing returns, critical thresholds, weather factor,
//! and wealth satisfaction factor work correctly in the full simulation.
//!
//! The happiness system fires every HAPPINESS_UPDATE_INTERVAL ticks (20).

use crate::citizen::{CitizenDetails, Needs};
use crate::grid::ZoneType;
use crate::happiness::HAPPINESS_UPDATE_INTERVAL;
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Ticks needed for the happiness system to fire.
const HAPPINESS_TICKS: u32 = HAPPINESS_UPDATE_INTERVAL as u32;

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

/// Set needs and health on all citizens.
fn set_needs_and_health(city: &mut TestCity, need_val: f32, health_val: f32) {
    let world = city.world_mut();
    let mut q = world.query::<(&mut Needs, &mut CitizenDetails)>();
    for (mut needs, mut details) in q.iter_mut(world) {
        needs.hunger = need_val;
        needs.energy = need_val;
        needs.social = need_val;
        needs.fun = need_val;
        needs.comfort = need_val;
        details.health = health_val;
    }
}

/// Set savings on all citizens.
fn set_savings(city: &mut TestCity, savings: f32) {
    let world = city.world_mut();
    let mut q = world.query::<&mut CitizenDetails>();
    for mut details in q.iter_mut(world) {
        details.savings = savings;
    }
}

/// Spawn a service building at a grid location.
fn spawn_service(city: &mut TestCity, gx: usize, gy: usize, service_type: ServiceType) {
    let radius = ServiceBuilding::coverage_radius(service_type);
    city.world_mut().spawn(ServiceBuilding {
        service_type,
        grid_x: gx,
        grid_y: gy,
        radius,
    });
}

/// Create a city with citizen + utilities.
fn city_with_utilities(home: (usize, usize), work: (usize, usize)) -> TestCity {
    TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower)
}

/// Advance to just before happiness fires, inject stable state, then tick once.
fn tick_with_stable_needs(city: &mut TestCity) {
    city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(city, 80.0, 90.0);
    set_savings(city, 5000.0);
    city.tick(1);
}

// ====================================================================
// 1. Diminishing returns: first service matters most
// ====================================================================

#[test]
fn test_happiness_tuning_first_service_matters_most() {
    let home = (100, 100);
    let work = (102, 100);

    // City with NO services
    let mut city_no_svc = city_with_utilities(home, work);
    tick_with_stable_needs(&mut city_no_svc);
    let h_no_svc = first_citizen_happiness(&mut city_no_svc);

    // City with ONE service (hospital)
    let mut city_one_svc = city_with_utilities(home, work);
    spawn_service(&mut city_one_svc, home.0, home.1, ServiceType::Hospital);
    tick_with_stable_needs(&mut city_one_svc);
    let h_one_svc = first_citizen_happiness(&mut city_one_svc);

    // City with ALL services
    let mut city_all_svc = city_with_utilities(home, work);
    spawn_service(&mut city_all_svc, home.0, home.1, ServiceType::Hospital);
    spawn_service(
        &mut city_all_svc,
        home.0,
        home.1,
        ServiceType::ElementarySchool,
    );
    spawn_service(
        &mut city_all_svc,
        home.0,
        home.1,
        ServiceType::PoliceStation,
    );
    spawn_service(&mut city_all_svc, home.0, home.1, ServiceType::SmallPark);
    spawn_service(&mut city_all_svc, home.0, home.1, ServiceType::Stadium);
    spawn_service(&mut city_all_svc, home.0, home.1, ServiceType::CellTower);
    spawn_service(&mut city_all_svc, home.0, home.1, ServiceType::BusDepot);
    spawn_service(&mut city_all_svc, home.0, home.1, ServiceType::FireStation);
    tick_with_stable_needs(&mut city_all_svc);
    let h_all_svc = first_citizen_happiness(&mut city_all_svc);

    // First service should add more than each subsequent one on average
    let gain_first = h_one_svc - h_no_svc;
    let gain_rest = h_all_svc - h_one_svc;
    let avg_gain_per_subsequent = gain_rest / 7.0; // 7 more services added

    assert!(
        gain_first > avg_gain_per_subsequent,
        "First service gain ({:.2}) should exceed avg subsequent gain ({:.2})",
        gain_first,
        avg_gain_per_subsequent
    );
}

// ====================================================================
// 2. Critical threshold: no water causes severe penalty
// ====================================================================

#[test]
fn test_happiness_tuning_no_water_critical_penalty() {
    let home = (100, 100);
    let work = (102, 100);

    // City WITH water
    let mut city_water = city_with_utilities(home, work);
    tick_with_stable_needs(&mut city_water);
    let h_water = first_citizen_happiness(&mut city_water);

    // City WITHOUT water (only power)
    let mut city_no_water = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant);
    tick_with_stable_needs(&mut city_no_water);
    let h_no_water = first_citizen_happiness(&mut city_no_water);

    // No water should cause a large happiness drop (> 30 points)
    let drop = h_water - h_no_water;
    assert!(
        drop > 30.0,
        "No water should cause > 30 point drop, got {:.2} (with={:.2}, without={:.2})",
        drop,
        h_water,
        h_no_water
    );
}

// ====================================================================
// 3. Critical threshold: no power causes severe penalty
// ====================================================================

#[test]
fn test_happiness_tuning_no_power_critical_penalty() {
    let home = (100, 100);
    let work = (102, 100);

    // City WITH power
    let mut city_power = city_with_utilities(home, work);
    tick_with_stable_needs(&mut city_power);
    let h_power = first_citizen_happiness(&mut city_power);

    // City WITHOUT power (only water)
    let mut city_no_power = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower);
    tick_with_stable_needs(&mut city_no_power);
    let h_no_power = first_citizen_happiness(&mut city_no_power);

    // No power should cause a significant drop (> 25 points)
    let drop = h_power - h_no_power;
    assert!(
        drop > 25.0,
        "No power should cause > 25 point drop, got {:.2} (with={:.2}, without={:.2})",
        drop,
        h_power,
        h_no_power
    );
}

// ====================================================================
// 4. Wealth satisfaction: savings affect happiness
// ====================================================================

#[test]
fn test_happiness_tuning_wealth_savings_positive() {
    let home = (100, 100);
    let work = (102, 100);

    // City with zero savings
    let mut city_poor = city_with_utilities(home, work);
    city_poor.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_poor, 80.0, 90.0);
    set_savings(&mut city_poor, 0.0);
    city_poor.tick(1);
    let h_poor = first_citizen_happiness(&mut city_poor);

    // City with comfortable savings
    let mut city_rich = city_with_utilities(home, work);
    city_rich.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_rich, 80.0, 90.0);
    set_savings(&mut city_rich, 10_000.0);
    city_rich.tick(1);
    let h_rich = first_citizen_happiness(&mut city_rich);

    assert!(
        h_rich > h_poor,
        "Wealthy citizen ({:.2}) should be happier than poor ({:.2})",
        h_rich,
        h_poor
    );
    // Gap should be at least 8 (poverty penalty + wealth bonus)
    let gap = h_rich - h_poor;
    assert!(
        gap > 8.0,
        "Wealth gap should be > 8 happiness points, got {:.2}",
        gap
    );
}

// ====================================================================
// 5. Critical health threshold
// ====================================================================

#[test]
fn test_happiness_tuning_critical_health_penalty() {
    let home = (100, 100);
    let work = (102, 100);

    // Citizen with normal health
    let mut city_healthy = city_with_utilities(home, work);
    city_healthy.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_healthy, 80.0, 90.0);
    set_savings(&mut city_healthy, 5000.0);
    city_healthy.tick(1);
    let h_healthy = first_citizen_happiness(&mut city_healthy);

    // Citizen with critically low health
    let mut city_sick = city_with_utilities(home, work);
    city_sick.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_sick, 80.0, 15.0);
    set_savings(&mut city_sick, 5000.0);
    city_sick.tick(1);
    let h_sick = first_citizen_happiness(&mut city_sick);

    let drop = h_healthy - h_sick;
    assert!(
        drop > 20.0,
        "Critical health should cause > 20 point drop, got {:.2} (healthy={:.2}, sick={:.2})",
        drop,
        h_healthy,
        h_sick
    );
}

// ====================================================================
// 6. Update interval is faster (20 ticks)
// ====================================================================

#[test]
fn test_happiness_tuning_update_interval_is_20() {
    assert_eq!(
        HAPPINESS_UPDATE_INTERVAL, 20u64,
        "Happiness should update every 20 ticks"
    );
}

// ====================================================================
// 7. All positive factors still yield high happiness
// ====================================================================

#[test]
fn test_happiness_tuning_all_positive_high() {
    let home = (100, 100);
    let work = (102, 100);

    let mut city = city_with_utilities(home, work);
    spawn_service(&mut city, home.0, home.1, ServiceType::Hospital);
    spawn_service(&mut city, home.0, home.1, ServiceType::ElementarySchool);
    spawn_service(&mut city, home.0, home.1, ServiceType::PoliceStation);
    spawn_service(&mut city, home.0, home.1, ServiceType::SmallPark);
    spawn_service(&mut city, home.0, home.1, ServiceType::Stadium);
    spawn_service(&mut city, home.0, home.1, ServiceType::CellTower);
    spawn_service(&mut city, home.0, home.1, ServiceType::BusDepot);

    city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city, 100.0, 100.0);
    set_savings(&mut city, 20_000.0);
    city.tick(1);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        happiness >= 75.0,
        "Expected high happiness (>=75) with all positive factors, got {:.2}",
        happiness
    );
}

// ====================================================================
// 8. All negative factors still yield low happiness
// ====================================================================

#[test]
fn test_happiness_tuning_all_negative_low() {
    let home = (100, 100);

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home);

    // No utilities, max pollution/crime/noise/garbage, high taxes
    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<crate::pollution::PollutionGrid>();
        pollution.set(home.0, home.1, 255);
        let mut crime = world.resource_mut::<crate::crime::CrimeGrid>();
        crime.set(home.0, home.1, 255);
        let mut noise = world.resource_mut::<crate::noise::NoisePollutionGrid>();
        noise.set(home.0, home.1, 255);
        let mut garbage = world.resource_mut::<crate::garbage::GarbageGrid>();
        garbage.set(home.0, home.1, 255);
        let mut budget = world.resource_mut::<crate::economy::CityBudget>();
        budget.tax_rate = 0.30;
    }

    city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city, 0.0, 10.0);
    set_savings(&mut city, 0.0);
    city.tick(1);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        happiness <= 10.0,
        "Expected very low happiness (<=10) with all negative factors, got {:.2}",
        happiness
    );
}

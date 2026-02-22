//! Integration tests for the happiness formula (TEST-002).
//!
//! Tests that each factor in the happiness calculation contributes correctly,
//! that the output is clamped to [0.0, 100.0], and that extreme inputs are
//! handled gracefully.
//!
//! The happiness system (`update_happiness`) runs in FixedUpdate when
//! `TickCounter.is_multiple_of(10)` (i.e., every 10th FixedUpdate tick).
//! Various simulation sub-systems (utility propagation, service coverage,
//! traffic density, needs decay) also run during FixedUpdate and can
//! overwrite manually-set state. The tests use a "late inject" pattern:
//!   1. `tick(9)` — let initialization and intermediate systems settle
//!   2. Inject test-specific state (coverage flags, needs, etc.)
//!   3. `tick(1)` — advance to tick 10 where happiness fires
//!
//! Injecting at tick 9 (rather than tick 1) minimizes the window for
//! other systems (update_needs, etc.) to overwrite the injected values.

use crate::citizen::{CitizenDetails, HomeLocation, Needs};
use crate::grid::ZoneType;
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Total ticks needed for the happiness system to fire (counter=10).
const HAPPINESS_TICKS: u32 = 10;

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

/// Collect happiness values for all citizens.
fn all_citizen_happiness(city: &mut TestCity) -> Vec<f32> {
    let world = city.world_mut();
    world
        .query::<&CitizenDetails>()
        .iter(world)
        .map(|d| d.happiness)
        .collect()
}

/// Spawn a service building at a grid location. The coverage system
/// (`update_service_coverage`) will detect it via `Added<ServiceBuilding>`
/// and naturally compute coverage flags, which survives change detection.
fn spawn_service(city: &mut TestCity, gx: usize, gy: usize, service_type: ServiceType) {
    let radius = ServiceBuilding::coverage_radius(service_type);
    city.world_mut().spawn(ServiceBuilding {
        service_type,
        grid_x: gx,
        grid_y: gy,
        radius,
    });
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

/// Create a city with a citizen at home, work, with power and water utilities
/// placed as direct 4-neighbors of home (BFS can reach through grass 1-hop).
fn city_with_utilities(home: (usize, usize), work: (usize, usize)) -> TestCity {
    TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower)
}

/// Create a city with an unemployed citizen and utilities.
fn city_unemployed_with_utilities(home: (usize, usize)) -> TestCity {
    TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower)
}

/// Get happiness of a citizen at a specific home grid location.
fn citizen_happiness_at(city: &mut TestCity, gx: usize, gy: usize) -> f32 {
    let world = city.world_mut();
    world
        .query::<(&CitizenDetails, &HomeLocation)>()
        .iter(world)
        .find(|(_, home)| home.grid_x == gx && home.grid_y == gy)
        .expect("expected a citizen at the given location")
        .0
        .happiness
}

/// Create a city with two citizens at different home locations, both with utilities.
/// Used for same-world comparison tests that eliminate cross-world Bevy change
/// detection noise (~1.8 point drift between separate worlds).
/// Work is at the midpoint so both citizens have equal commute distance.
fn two_citizen_city() -> (TestCity, (usize, usize), (usize, usize)) {
    let home_a = (100, 100);
    let home_b = (130, 130);
    let work = (115, 115); // equidistant from both homes (~21.2 cells each)
    let city = TestCity::new()
        .with_building(home_a.0, home_a.1, ZoneType::ResidentialLow, 1)
        .with_building(home_b.0, home_b.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home_a, work)
        .with_citizen(home_b, work)
        .with_utility(home_a.0, home_a.1 + 1, UtilityType::PowerPlant)
        .with_utility(home_a.0, home_a.1 - 1, UtilityType::WaterTower)
        .with_utility(home_b.0, home_b.1 + 1, UtilityType::PowerPlant)
        .with_utility(home_b.0, home_b.1 - 1, UtilityType::WaterTower);
    (city, home_a, home_b)
}

/// Advance to tick 9, then inject stable needs/health, then tick once more
/// so happiness fires at counter=10 with our injected values still fresh.
/// Injecting at tick 9 instead of tick 1 minimizes the window for other
/// systems (update_needs, etc.) to overwrite injected state.
fn tick_with_stable_needs(city: &mut TestCity) {
    city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(city, 80.0, 90.0);
    city.tick(1);
}

// ====================================================================
// 1. All positive factors -> high happiness
// ====================================================================

#[test]
fn test_happiness_all_positive_factors_yields_high_happiness() {
    let home = (100, 100);
    let work = (102, 100); // very short commute (distance 2)

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower);

    // Spawn service buildings for full coverage
    spawn_service(&mut city, home.0, home.1, ServiceType::Hospital);
    spawn_service(&mut city, home.0, home.1, ServiceType::ElementarySchool);
    spawn_service(&mut city, home.0, home.1, ServiceType::PoliceStation);
    spawn_service(&mut city, home.0, home.1, ServiceType::SmallPark);
    spawn_service(&mut city, home.0, home.1, ServiceType::Stadium);
    spawn_service(&mut city, home.0, home.1, ServiceType::CellTower);
    spawn_service(&mut city, home.0, home.1, ServiceType::BusDepot);

    // Max needs and health
    set_needs_and_health(&mut city, 100.0, 100.0);

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        happiness >= 80.0,
        "Expected high happiness (>=80) with all positive factors, got {happiness}"
    );
}

// ====================================================================
// 2. All negative factors -> low happiness
// ====================================================================

#[test]
fn test_happiness_all_negative_factors_yields_low_happiness() {
    let home = (100, 100);

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home);

    // No utilities → no power/water
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

    set_needs_and_health(&mut city, 0.0, 10.0);

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        happiness <= 10.0,
        "Expected low happiness (<=10) with all negative factors, got {happiness}"
    );
}

// ====================================================================
// 3. Individual factor tests
// ====================================================================

#[test]
fn test_happiness_employment_bonus() {
    let home = (100, 100);
    let work = (120, 100);

    let mut employed_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut employed_city);

    let mut unemployed_city = city_unemployed_with_utilities(home);
    tick_with_stable_needs(&mut unemployed_city);

    let h_emp = first_citizen_happiness(&mut employed_city);
    let h_unemp = first_citizen_happiness(&mut unemployed_city);

    assert!(
        h_emp > h_unemp,
        "Employed citizen should be happier ({h_emp}) than unemployed ({h_unemp})"
    );
}

#[test]
fn test_happiness_short_commute_bonus() {
    let home = (100, 100);
    let work_near = (105, 100); // distance 5 < 20
    let work_far = (130, 100); // distance 30 >= 20

    let mut near_city = city_with_utilities(home, work_near);
    tick_with_stable_needs(&mut near_city);

    let mut far_city = city_with_utilities(home, work_far);
    tick_with_stable_needs(&mut far_city);

    let h_near = first_citizen_happiness(&mut near_city);
    let h_far = first_citizen_happiness(&mut far_city);

    assert!(
        h_near > h_far,
        "Short commute citizen should be happier ({h_near}) than long commute ({h_far})"
    );
}

#[test]
fn test_happiness_power_bonus_and_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    // With power and water
    let mut powered_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut powered_city);

    // Without power (only water, placed as 4-neighbor)
    let mut unpowered_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower);
    tick_with_stable_needs(&mut unpowered_city);

    let h_powered = first_citizen_happiness(&mut powered_city);
    let h_unpowered = first_citizen_happiness(&mut unpowered_city);

    // Power bonus (+5) vs no power penalty (-25) = 30 point swing
    let delta = h_powered - h_unpowered;
    assert!(
        delta > 20.0,
        "Power should create a large happiness delta (expected >20, got {delta})"
    );
}

#[test]
fn test_happiness_water_bonus_and_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    // With power and water
    let mut watered_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut watered_city);

    // Without water (only power, placed as 4-neighbor)
    let mut dry_city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant);
    tick_with_stable_needs(&mut dry_city);

    let h_watered = first_citizen_happiness(&mut watered_city);
    let h_dry = first_citizen_happiness(&mut dry_city);

    // Water bonus (+5) vs no water penalty (-20) = 25 point swing
    let delta = h_watered - h_dry;
    assert!(
        delta > 15.0,
        "Water should create a large happiness delta (expected >15, got {delta})"
    );
}

#[test]
fn test_happiness_health_service_coverage_bonus() {
    let (mut city, home_a, home_b) = two_citizen_city();
    spawn_service(&mut city, home_a.0, home_a.1, ServiceType::Hospital);
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(HAPPINESS_TICKS);

    let h_covered = citizen_happiness_at(&mut city, home_a.0, home_a.1);
    let h_uncovered = citizen_happiness_at(&mut city, home_b.0, home_b.1);

    assert!(
        h_covered > h_uncovered,
        "Health coverage should increase happiness ({h_covered} vs {h_uncovered})"
    );
}

#[test]
fn test_happiness_park_coverage_bonus() {
    let (mut city, home_a, home_b) = two_citizen_city();
    spawn_service(&mut city, home_a.0, home_a.1, ServiceType::SmallPark);
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(HAPPINESS_TICKS);

    let h_park = citizen_happiness_at(&mut city, home_a.0, home_a.1);
    let h_no_park = citizen_happiness_at(&mut city, home_b.0, home_b.1);

    assert!(
        h_park > h_no_park,
        "Park coverage should increase happiness ({h_park} vs {h_no_park})"
    );
}

#[test]
fn test_happiness_pollution_penalty() {
    let (mut city, home_a, home_b) = two_citizen_city();
    city.tick(HAPPINESS_TICKS - 1);
    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<crate::pollution::PollutionGrid>();
        pollution.set(home_a.0, home_a.1, 200);
    }
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(1);

    let h_clean = citizen_happiness_at(&mut city, home_b.0, home_b.1);
    let h_polluted = citizen_happiness_at(&mut city, home_a.0, home_a.1);

    assert!(
        h_clean > h_polluted,
        "Pollution should decrease happiness ({h_clean} vs {h_polluted})"
    );
}

#[test]
fn test_happiness_crime_penalty() {
    let (mut city, home_a, home_b) = two_citizen_city();
    city.tick(HAPPINESS_TICKS - 1);
    {
        let world = city.world_mut();
        let mut crime = world.resource_mut::<crate::crime::CrimeGrid>();
        crime.set(home_a.0, home_a.1, 200);
    }
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(1);

    let h_safe = citizen_happiness_at(&mut city, home_b.0, home_b.1);
    let h_crime = citizen_happiness_at(&mut city, home_a.0, home_a.1);

    assert!(
        h_safe > h_crime,
        "Crime should decrease happiness ({h_safe} vs {h_crime})"
    );
}

#[test]
fn test_happiness_noise_penalty() {
    let (mut city, home_a, home_b) = two_citizen_city();
    city.tick(HAPPINESS_TICKS - 1);
    {
        let world = city.world_mut();
        let mut noise = world.resource_mut::<crate::noise::NoisePollutionGrid>();
        noise.set(home_a.0, home_a.1, 200);
    }
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(1);

    let h_quiet = citizen_happiness_at(&mut city, home_b.0, home_b.1);
    let h_noisy = citizen_happiness_at(&mut city, home_a.0, home_a.1);

    assert!(
        h_quiet > h_noisy,
        "Noise should decrease happiness ({h_quiet} vs {h_noisy})"
    );
}

#[test]
fn test_happiness_high_tax_penalty() {
    // Tax rate is a global resource, so we can't differentiate per-citizen in the same world.
    // Use a large tax difference (0.10 vs 0.30) which creates a 10+ point swing,
    // well above the ~1.8 point cross-world noise.
    let home = (100, 100);
    let work = (120, 100);

    let mut low_tax_city = city_with_utilities(home, work);
    {
        let world = low_tax_city.world_mut();
        let mut budget = world.resource_mut::<crate::economy::CityBudget>();
        budget.tax_rate = 0.10;
    }
    tick_with_stable_needs(&mut low_tax_city);

    let mut high_tax_city = city_with_utilities(home, work);
    {
        let world = high_tax_city.world_mut();
        let mut budget = world.resource_mut::<crate::economy::CityBudget>();
        budget.tax_rate = 0.30;
    }
    tick_with_stable_needs(&mut high_tax_city);

    let h_low = first_citizen_happiness(&mut low_tax_city);
    let h_high = first_citizen_happiness(&mut high_tax_city);

    assert!(
        h_low > h_high,
        "High taxes should decrease happiness ({h_low} vs {h_high})"
    );
}

#[test]
fn test_happiness_education_service_coverage() {
    let (mut city, home_a, home_b) = two_citizen_city();
    spawn_service(&mut city, home_a.0, home_a.1, ServiceType::ElementarySchool);
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(HAPPINESS_TICKS);

    let h_edu = citizen_happiness_at(&mut city, home_a.0, home_a.1);
    let h_no_edu = citizen_happiness_at(&mut city, home_b.0, home_b.1);

    assert!(
        h_edu > h_no_edu,
        "Education coverage should increase happiness ({h_edu} vs {h_no_edu})"
    );
}

#[test]
fn test_happiness_police_coverage() {
    let (mut city, home_a, home_b) = two_citizen_city();
    spawn_service(&mut city, home_a.0, home_a.1, ServiceType::PoliceStation);
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(HAPPINESS_TICKS);

    let h_police = citizen_happiness_at(&mut city, home_a.0, home_a.1);
    let h_no_police = citizen_happiness_at(&mut city, home_b.0, home_b.1);

    assert!(
        h_police > h_no_police,
        "Police coverage should increase happiness ({h_police} vs {h_no_police})"
    );
}

#[test]
fn test_happiness_entertainment_coverage() {
    let (mut city, home_a, home_b) = two_citizen_city();
    spawn_service(&mut city, home_a.0, home_a.1, ServiceType::Stadium);
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(HAPPINESS_TICKS);

    let h_ent = citizen_happiness_at(&mut city, home_a.0, home_a.1);
    let h_no_ent = citizen_happiness_at(&mut city, home_b.0, home_b.1);

    assert!(
        h_ent > h_no_ent,
        "Entertainment coverage should increase happiness ({h_ent} vs {h_no_ent})"
    );
}

#[test]
fn test_happiness_telecom_coverage() {
    let (mut city, home_a, home_b) = two_citizen_city();
    spawn_service(&mut city, home_a.0, home_a.1, ServiceType::CellTower);
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(HAPPINESS_TICKS);

    let h_telecom = citizen_happiness_at(&mut city, home_a.0, home_a.1);
    let h_no_telecom = citizen_happiness_at(&mut city, home_b.0, home_b.1);

    assert!(
        h_telecom > h_no_telecom,
        "Telecom coverage should increase happiness ({h_telecom} vs {h_no_telecom})"
    );
}

#[test]
fn test_happiness_transport_coverage() {
    let (mut city, home_a, home_b) = two_citizen_city();
    spawn_service(&mut city, home_a.0, home_a.1, ServiceType::BusDepot);
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(HAPPINESS_TICKS);

    let h_transport = citizen_happiness_at(&mut city, home_a.0, home_a.1);
    let h_no_transport = citizen_happiness_at(&mut city, home_b.0, home_b.1);

    assert!(
        h_transport > h_no_transport,
        "Transport coverage should increase happiness ({h_transport} vs {h_no_transport})"
    );
}

#[test]
fn test_happiness_land_value_bonus() {
    let (mut city, home_a, home_b) = two_citizen_city();
    // Make citizens HighIncome (education=3) so land_value weight = 2.0 instead of 0.3.
    // This gives a (200/50)*2.0 = 8.0 point bonus, well above positional noise (~3.8 pts).
    {
        let world = city.world_mut();
        let mut q = world.query::<&mut CitizenDetails>();
        for mut details in q.iter_mut(world) {
            details.education = 3;
        }
    }
    city.tick(HAPPINESS_TICKS - 1);
    {
        let world = city.world_mut();
        let mut land_value = world.resource_mut::<crate::land_value::LandValueGrid>();
        land_value.set(home_a.0, home_a.1, 200);
    }
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(1);

    let h_high = citizen_happiness_at(&mut city, home_a.0, home_a.1);
    let h_low = citizen_happiness_at(&mut city, home_b.0, home_b.1);

    assert!(
        h_high > h_low,
        "High land value should increase happiness ({h_high} vs {h_low})"
    );
}

#[test]
fn test_happiness_garbage_penalty_threshold() {
    let (mut city, home_a, home_b) = two_citizen_city();
    city.tick(HAPPINESS_TICKS - 1);
    {
        let world = city.world_mut();
        let mut garbage = world.resource_mut::<crate::garbage::GarbageGrid>();
        garbage.set(home_a.0, home_a.1, 50);
    }
    set_needs_and_health(&mut city, 80.0, 90.0);
    city.tick(1);

    let h_low = citizen_happiness_at(&mut city, home_b.0, home_b.1);
    let h_high = citizen_happiness_at(&mut city, home_a.0, home_a.1);

    assert!(
        h_low > h_high,
        "High garbage (above threshold 10) should reduce happiness ({h_low} vs {h_high})"
    );
}

#[test]
fn test_happiness_low_health_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    let mut healthy_city = city_with_utilities(home, work);
    healthy_city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut healthy_city, 80.0, 90.0);
    healthy_city.tick(1);

    let mut sick_city = city_with_utilities(home, work);
    sick_city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut sick_city, 80.0, 10.0);
    sick_city.tick(1);

    let h_healthy = first_citizen_happiness(&mut healthy_city);
    let h_sick = first_citizen_happiness(&mut sick_city);

    assert!(
        h_healthy > h_sick,
        "Healthy citizens should be happier ({h_healthy}) than sick ones ({h_sick})"
    );
}

#[test]
fn test_happiness_needs_satisfaction_impact() {
    let home = (100, 100);
    let work = (120, 100);

    let mut satisfied_city = city_with_utilities(home, work);
    satisfied_city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut satisfied_city, 100.0, 90.0);
    satisfied_city.tick(1);

    let mut unsatisfied_city = city_with_utilities(home, work);
    unsatisfied_city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut unsatisfied_city, 10.0, 90.0);
    unsatisfied_city.tick(1);

    let h_sat = first_citizen_happiness(&mut satisfied_city);
    let h_unsat = first_citizen_happiness(&mut unsatisfied_city);

    assert!(
        h_sat > h_unsat,
        "Satisfied citizens should be happier ({h_sat}) than unsatisfied ({h_unsat})"
    );
}

// ====================================================================
// 4. Output clamped to [0.0, 100.0]
// ====================================================================

#[test]
fn test_happiness_clamped_at_zero() {
    let home = (100, 100);

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen(home);

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
        budget.tax_rate = 0.50;
    }
    set_needs_and_health(&mut city, 0.0, 0.0);

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    assert_eq!(
        happiness, 0.0,
        "With extreme negative factors, happiness should be clamped to exactly 0.0, got {happiness}"
    );
}

#[test]
fn test_happiness_clamped_at_hundred() {
    // Verify that with all max bonuses, happiness is clamped and does not
    // exceed 100.0. The raw formula can produce values > 100.
    let home = (100, 100);
    let work = (101, 100);

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower);

    // Max land value
    {
        let world = city.world_mut();
        let mut land_value = world.resource_mut::<crate::land_value::LandValueGrid>();
        land_value.set(home.0, home.1, 255);
    }

    // Spawn service buildings for full coverage
    spawn_service(&mut city, home.0, home.1, ServiceType::Hospital);
    spawn_service(&mut city, home.0, home.1, ServiceType::ElementarySchool);
    spawn_service(&mut city, home.0, home.1, ServiceType::PoliceStation);
    spawn_service(&mut city, home.0, home.1, ServiceType::SmallPark);
    spawn_service(&mut city, home.0, home.1, ServiceType::Stadium);
    spawn_service(&mut city, home.0, home.1, ServiceType::CellTower);
    spawn_service(&mut city, home.0, home.1, ServiceType::BusDepot);

    // Max needs and health
    set_needs_and_health(&mut city, 100.0, 100.0);

    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        happiness <= 100.0,
        "Happiness should never exceed 100, got {happiness}"
    );
    // With all bonuses the raw value exceeds 100, so clamp should cap it
    assert!(
        happiness >= 95.0,
        "With all max bonuses, happiness should be near maximum (>=95), got {happiness}"
    );
}

// ====================================================================
// 5. Extreme values
// ====================================================================

#[test]
fn test_happiness_extreme_pollution_255() {
    let home = (100, 100);
    let work = (120, 100);

    let mut city = city_with_utilities(home, work);
    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<crate::pollution::PollutionGrid>();
        pollution.set(home.0, home.1, 255);
    }
    tick_with_stable_needs(&mut city);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        (0.0..=100.0).contains(&happiness),
        "Happiness should be in [0, 100] even with max pollution, got {happiness}"
    );
}

#[test]
fn test_happiness_extreme_crime_255() {
    let home = (100, 100);
    let work = (120, 100);

    let mut city = city_with_utilities(home, work);
    {
        let world = city.world_mut();
        let mut crime = world.resource_mut::<crate::crime::CrimeGrid>();
        crime.set(home.0, home.1, 255);
    }
    tick_with_stable_needs(&mut city);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        (0.0..=100.0).contains(&happiness),
        "Happiness should be in [0, 100] even with max crime, got {happiness}"
    );
}

#[test]
fn test_happiness_extreme_all_services_max_land_value() {
    let home = (100, 100);
    let work = (101, 100);

    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower);

    {
        let world = city.world_mut();
        let mut land_value = world.resource_mut::<crate::land_value::LandValueGrid>();
        land_value.set(home.0, home.1, 255);
    }

    // Spawn all service buildings for full coverage
    spawn_service(&mut city, home.0, home.1, ServiceType::Hospital);
    spawn_service(&mut city, home.0, home.1, ServiceType::ElementarySchool);
    spawn_service(&mut city, home.0, home.1, ServiceType::PoliceStation);
    spawn_service(&mut city, home.0, home.1, ServiceType::SmallPark);
    spawn_service(&mut city, home.0, home.1, ServiceType::Stadium);
    spawn_service(&mut city, home.0, home.1, ServiceType::CellTower);
    spawn_service(&mut city, home.0, home.1, ServiceType::BusDepot);
    spawn_service(&mut city, home.0, home.1, ServiceType::FireStation);

    set_needs_and_health(&mut city, 100.0, 100.0);
    city.tick(HAPPINESS_TICKS);

    let happiness = first_citizen_happiness(&mut city);
    assert!(
        happiness <= 100.0,
        "Even with all services and max land value, happiness must be <= 100.0, got {happiness}"
    );
    assert!(
        happiness >= 90.0,
        "With all positive factors, happiness should be very high (>=90), got {happiness}"
    );
}

#[test]
fn test_happiness_multiple_citizens_different_conditions() {
    let home_a = (100, 100);
    let home_b = (150, 150);
    let work_a = (102, 100);
    let work_b = (170, 150);

    let mut city = TestCity::new()
        .with_building(home_a.0, home_a.1, ZoneType::ResidentialLow, 1)
        .with_building(home_b.0, home_b.1, ZoneType::ResidentialLow, 1)
        .with_building(work_a.0, work_a.1, ZoneType::CommercialLow, 1)
        .with_building(work_b.0, work_b.1, ZoneType::CommercialLow, 1)
        .with_citizen(home_a, work_a)
        .with_citizen(home_b, work_b)
        .with_utility(home_a.0, home_a.1 + 1, UtilityType::PowerPlant)
        .with_utility(home_a.0, home_a.1 - 1, UtilityType::WaterTower);

    // Add pollution and crime at citizen B's home
    {
        let world = city.world_mut();
        let mut pollution = world.resource_mut::<crate::pollution::PollutionGrid>();
        pollution.set(home_b.0, home_b.1, 200);
        let mut crime = world.resource_mut::<crate::crime::CrimeGrid>();
        crime.set(home_b.0, home_b.1, 200);
    }
    tick_with_stable_needs(&mut city);

    let happinesses = all_citizen_happiness(&mut city);
    assert!(
        happinesses.len() >= 2,
        "Should have at least 2 citizens, got {}",
        happinesses.len()
    );

    for h in &happinesses {
        assert!(
            (0.0..=100.0).contains(h),
            "All happiness values must be in [0, 100], got {h}"
        );
    }
}

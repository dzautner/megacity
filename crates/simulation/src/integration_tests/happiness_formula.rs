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

use crate::citizen::{CitizenDetails, Needs};
use crate::grid::ZoneType;
use crate::happiness::ServiceCoverageGrid;
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

/// Set coverage flags on the ServiceCoverageGrid for a cell.
/// Also marks the coverage grid as not-dirty to prevent `update_service_coverage`
/// from recomputing (and clearing) our injected flags during the next tick.
fn set_coverage_flags(city: &mut TestCity, x: usize, y: usize, flags: u8) {
    let world = city.world_mut();
    let idx = ServiceCoverageGrid::idx(x, y);
    let mut coverage = world.resource_mut::<ServiceCoverageGrid>();
    coverage.flags[idx] |= flags;
    coverage.dirty = false;
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

    // Run 1 tick so initialization systems run
    city.tick(1);

    // Inject full service coverage
    set_coverage_flags(
        &mut city,
        home.0,
        home.1,
        crate::happiness::COVERAGE_HEALTH
            | crate::happiness::COVERAGE_EDUCATION
            | crate::happiness::COVERAGE_POLICE
            | crate::happiness::COVERAGE_PARK
            | crate::happiness::COVERAGE_ENTERTAINMENT
            | crate::happiness::COVERAGE_TELECOM
            | crate::happiness::COVERAGE_TRANSPORT,
    );

    // Max needs and health
    set_needs_and_health(&mut city, 100.0, 100.0);

    city.tick(HAPPINESS_TICKS - 1);

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
    let home = (100, 100);
    let work = (120, 100);

    let mut covered_city = city_with_utilities(home, work);
    covered_city.tick(HAPPINESS_TICKS - 1);
    set_coverage_flags(
        &mut covered_city,
        home.0,
        home.1,
        crate::happiness::COVERAGE_HEALTH,
    );
    set_needs_and_health(&mut covered_city, 80.0, 90.0);
    covered_city.tick(1);

    let mut uncovered_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut uncovered_city);

    let h_covered = first_citizen_happiness(&mut covered_city);
    let h_uncovered = first_citizen_happiness(&mut uncovered_city);

    assert!(
        h_covered > h_uncovered,
        "Health coverage should increase happiness ({h_covered} vs {h_uncovered})"
    );
}

#[test]
fn test_happiness_park_coverage_bonus() {
    let home = (100, 100);
    let work = (120, 100);

    let mut park_city = city_with_utilities(home, work);
    park_city.tick(HAPPINESS_TICKS - 1);
    set_coverage_flags(
        &mut park_city,
        home.0,
        home.1,
        crate::happiness::COVERAGE_PARK,
    );
    set_needs_and_health(&mut park_city, 80.0, 90.0);
    park_city.tick(1);

    let mut no_park_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut no_park_city);

    let h_park = first_citizen_happiness(&mut park_city);
    let h_no_park = first_citizen_happiness(&mut no_park_city);

    assert!(
        h_park > h_no_park,
        "Park coverage should increase happiness ({h_park} vs {h_no_park})"
    );
}

#[test]
fn test_happiness_pollution_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    let mut clean_city = city_with_utilities(home, work);
    clean_city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut clean_city, 80.0, 90.0);
    clean_city.tick(1);

    let mut polluted_city = city_with_utilities(home, work);
    polluted_city.tick(HAPPINESS_TICKS - 1);
    {
        let world = polluted_city.world_mut();
        let mut pollution = world.resource_mut::<crate::pollution::PollutionGrid>();
        pollution.set(home.0, home.1, 200);
    }
    set_needs_and_health(&mut polluted_city, 80.0, 90.0);
    polluted_city.tick(1);

    let h_clean = first_citizen_happiness(&mut clean_city);
    let h_polluted = first_citizen_happiness(&mut polluted_city);

    assert!(
        h_clean > h_polluted,
        "Pollution should decrease happiness ({h_clean} vs {h_polluted})"
    );
}

#[test]
fn test_happiness_crime_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    let mut safe_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut safe_city);

    let mut crime_city = city_with_utilities(home, work);
    {
        let world = crime_city.world_mut();
        let mut crime = world.resource_mut::<crate::crime::CrimeGrid>();
        crime.set(home.0, home.1, 200);
    }
    tick_with_stable_needs(&mut crime_city);

    let h_safe = first_citizen_happiness(&mut safe_city);
    let h_crime = first_citizen_happiness(&mut crime_city);

    assert!(
        h_safe > h_crime,
        "Crime should decrease happiness ({h_safe} vs {h_crime})"
    );
}

#[test]
fn test_happiness_noise_penalty() {
    let home = (100, 100);
    let work = (120, 100);

    let mut quiet_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut quiet_city);

    let mut noisy_city = city_with_utilities(home, work);
    {
        let world = noisy_city.world_mut();
        let mut noise = world.resource_mut::<crate::noise::NoisePollutionGrid>();
        noise.set(home.0, home.1, 200);
    }
    tick_with_stable_needs(&mut noisy_city);

    let h_quiet = first_citizen_happiness(&mut quiet_city);
    let h_noisy = first_citizen_happiness(&mut noisy_city);

    assert!(
        h_quiet > h_noisy,
        "Noise should decrease happiness ({h_quiet} vs {h_noisy})"
    );
}

#[test]
fn test_happiness_high_tax_penalty() {
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
    let home = (100, 100);
    let work = (120, 100);

    let mut edu_city = city_with_utilities(home, work);
    edu_city.tick(HAPPINESS_TICKS - 1);
    set_coverage_flags(
        &mut edu_city,
        home.0,
        home.1,
        crate::happiness::COVERAGE_EDUCATION,
    );
    set_needs_and_health(&mut edu_city, 80.0, 90.0);
    edu_city.tick(1);

    let mut no_edu_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut no_edu_city);

    let h_edu = first_citizen_happiness(&mut edu_city);
    let h_no_edu = first_citizen_happiness(&mut no_edu_city);

    assert!(
        h_edu > h_no_edu,
        "Education coverage should increase happiness ({h_edu} vs {h_no_edu})"
    );
}

#[test]
fn test_happiness_police_coverage() {
    let home = (100, 100);
    let work = (120, 100);

    let mut police_city = city_with_utilities(home, work);
    police_city.tick(HAPPINESS_TICKS - 1);
    set_coverage_flags(
        &mut police_city,
        home.0,
        home.1,
        crate::happiness::COVERAGE_POLICE,
    );
    set_needs_and_health(&mut police_city, 80.0, 90.0);
    police_city.tick(1);

    let mut no_police_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut no_police_city);

    let h_police = first_citizen_happiness(&mut police_city);
    let h_no_police = first_citizen_happiness(&mut no_police_city);

    assert!(
        h_police > h_no_police,
        "Police coverage should increase happiness ({h_police} vs {h_no_police})"
    );
}

#[test]
fn test_happiness_entertainment_coverage() {
    let home = (100, 100);
    let work = (120, 100);

    let mut ent_city = city_with_utilities(home, work);
    ent_city.tick(HAPPINESS_TICKS - 1);
    set_coverage_flags(
        &mut ent_city,
        home.0,
        home.1,
        crate::happiness::COVERAGE_ENTERTAINMENT,
    );
    set_needs_and_health(&mut ent_city, 80.0, 90.0);
    ent_city.tick(1);

    let mut no_ent_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut no_ent_city);

    let h_ent = first_citizen_happiness(&mut ent_city);
    let h_no_ent = first_citizen_happiness(&mut no_ent_city);

    assert!(
        h_ent > h_no_ent,
        "Entertainment coverage should increase happiness ({h_ent} vs {h_no_ent})"
    );
}

#[test]
fn test_happiness_telecom_coverage() {
    let home = (100, 100);
    let work = (120, 100);

    let mut telecom_city = city_with_utilities(home, work);
    telecom_city.tick(HAPPINESS_TICKS - 1);
    set_coverage_flags(
        &mut telecom_city,
        home.0,
        home.1,
        crate::happiness::COVERAGE_TELECOM,
    );
    set_needs_and_health(&mut telecom_city, 80.0, 90.0);
    telecom_city.tick(1);

    let mut no_telecom_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut no_telecom_city);

    let h_telecom = first_citizen_happiness(&mut telecom_city);
    let h_no_telecom = first_citizen_happiness(&mut no_telecom_city);

    assert!(
        h_telecom > h_no_telecom,
        "Telecom coverage should increase happiness ({h_telecom} vs {h_no_telecom})"
    );
}

#[test]
fn test_happiness_transport_coverage() {
    let home = (100, 100);
    let work = (120, 100);

    let mut transport_city = city_with_utilities(home, work);
    transport_city.tick(HAPPINESS_TICKS - 1);
    set_coverage_flags(
        &mut transport_city,
        home.0,
        home.1,
        crate::happiness::COVERAGE_TRANSPORT,
    );
    set_needs_and_health(&mut transport_city, 80.0, 90.0);
    transport_city.tick(1);

    let mut no_transport_city = city_with_utilities(home, work);
    tick_with_stable_needs(&mut no_transport_city);

    let h_transport = first_citizen_happiness(&mut transport_city);
    let h_no_transport = first_citizen_happiness(&mut no_transport_city);

    assert!(
        h_transport > h_no_transport,
        "Transport coverage should increase happiness ({h_transport} vs {h_no_transport})"
    );
}

#[test]
fn test_happiness_land_value_bonus() {
    let home = (100, 100);
    let work = (120, 100);

    let mut high_lv_city = city_with_utilities(home, work);
    high_lv_city.tick(HAPPINESS_TICKS - 1);
    {
        let world = high_lv_city.world_mut();
        let mut land_value = world.resource_mut::<crate::land_value::LandValueGrid>();
        land_value.set(home.0, home.1, 200);
    }
    set_needs_and_health(&mut high_lv_city, 80.0, 90.0);
    high_lv_city.tick(1);

    let mut low_lv_city = city_with_utilities(home, work);
    low_lv_city.tick(HAPPINESS_TICKS - 1);
    {
        let world = low_lv_city.world_mut();
        let mut land_value = world.resource_mut::<crate::land_value::LandValueGrid>();
        land_value.set(home.0, home.1, 0);
    }
    set_needs_and_health(&mut low_lv_city, 80.0, 90.0);
    low_lv_city.tick(1);

    let h_high = first_citizen_happiness(&mut high_lv_city);
    let h_low = first_citizen_happiness(&mut low_lv_city);

    assert!(
        h_high > h_low,
        "High land value should increase happiness ({h_high} vs {h_low})"
    );
}

#[test]
fn test_happiness_garbage_penalty_threshold() {
    let home = (100, 100);
    let work = (120, 100);

    // Garbage below threshold
    let mut low_garbage_city = city_with_utilities(home, work);
    {
        let world = low_garbage_city.world_mut();
        let mut garbage = world.resource_mut::<crate::garbage::GarbageGrid>();
        garbage.set(home.0, home.1, 5);
    }
    tick_with_stable_needs(&mut low_garbage_city);

    // Garbage above threshold
    let mut high_garbage_city = city_with_utilities(home, work);
    {
        let world = high_garbage_city.world_mut();
        let mut garbage = world.resource_mut::<crate::garbage::GarbageGrid>();
        garbage.set(home.0, home.1, 50);
    }
    tick_with_stable_needs(&mut high_garbage_city);

    let h_low = first_citizen_happiness(&mut low_garbage_city);
    let h_high = first_citizen_happiness(&mut high_garbage_city);

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

    // Run 1 tick for initialization
    city.tick(1);

    // Inject full coverage
    set_coverage_flags(
        &mut city,
        home.0,
        home.1,
        crate::happiness::COVERAGE_HEALTH
            | crate::happiness::COVERAGE_EDUCATION
            | crate::happiness::COVERAGE_POLICE
            | crate::happiness::COVERAGE_PARK
            | crate::happiness::COVERAGE_ENTERTAINMENT
            | crate::happiness::COVERAGE_TELECOM
            | crate::happiness::COVERAGE_TRANSPORT,
    );

    // Max needs and health
    set_needs_and_health(&mut city, 100.0, 100.0);

    city.tick(HAPPINESS_TICKS - 1);

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

    city.tick(1);
    set_coverage_flags(
        &mut city,
        home.0,
        home.1,
        crate::happiness::COVERAGE_HEALTH
            | crate::happiness::COVERAGE_EDUCATION
            | crate::happiness::COVERAGE_POLICE
            | crate::happiness::COVERAGE_PARK
            | crate::happiness::COVERAGE_ENTERTAINMENT
            | crate::happiness::COVERAGE_TELECOM
            | crate::happiness::COVERAGE_TRANSPORT
            | crate::happiness::COVERAGE_FIRE,
    );
    set_needs_and_health(&mut city, 100.0, 100.0);
    city.tick(HAPPINESS_TICKS - 1);

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

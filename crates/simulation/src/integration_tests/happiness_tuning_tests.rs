//! Integration tests for happiness formula tuning (issue #552).
//!
//! Tests that diminishing returns, critical thresholds, weather factor,
//! and wealth satisfaction factor work correctly in the full simulation.
//!
//! The happiness system fires every HAPPINESS_UPDATE_INTERVAL ticks (20).

use crate::citizen::{CitizenDetails, Needs};
use crate::grid::ZoneType;
use crate::happiness::HAPPINESS_UPDATE_INTERVAL;
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
    city.tick(1);
}

// ====================================================================
// 1. Wealth satisfaction affects happiness (diminishing returns verified
//    by unit tests in happiness/tests.rs, here we verify the integration)
// ====================================================================

#[test]
fn test_happiness_tuning_wealth_affects_happiness() {
    use crate::happiness::wealth_satisfaction;
    let home = (100, 100);
    let work = (102, 100);

    // Verify the wealth_satisfaction function itself has diminishing returns
    // (this is deterministic, no simulation variance)
    let first_quarter = wealth_satisfaction(2500.0) - wealth_satisfaction(0.01);
    let last_quarter = wealth_satisfaction(10_000.0) - wealth_satisfaction(7500.0);
    assert!(
        first_quarter > last_quarter,
        "First $2500 gain ({:.2}) should exceed last $2500 gain ({:.2}) in wealth_satisfaction",
        first_quarter,
        last_quarter,
    );

    // Integration check: a single city, two measurements with different savings.
    // Use the late-inject pattern: advance to tick N-1, set state, tick once.
    let mut city = city_with_utilities(home, work);
    city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city, 50.0, 70.0);
    set_savings(&mut city, 0.01);
    city.tick(1);
    let h_poor = first_citizen_happiness(&mut city);

    // Advance to the next happiness boundary (tick 2*HAPPINESS_TICKS)
    // using the same late-inject pattern.
    city.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city, 50.0, 70.0);
    set_savings(&mut city, 10_000.0);
    city.tick(1);
    let h_rich = first_citizen_happiness(&mut city);

    assert!(
        h_rich > h_poor,
        "Higher savings should yield higher happiness: rich={:.2}, poor={:.2}",
        h_rich,
        h_poor,
    );
}

// ====================================================================
// 2. Critical threshold: no water causes severe penalty
// ====================================================================

#[test]
fn test_happiness_tuning_no_water_critical_penalty() {
    let home = (100, 100);
    let work = (102, 100);

    let mut city_water = city_with_utilities(home, work);
    tick_with_stable_needs(&mut city_water);
    let h_water = first_citizen_happiness(&mut city_water);

    let mut city_no_water = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 + 1, UtilityType::PowerPlant);
    tick_with_stable_needs(&mut city_no_water);
    let h_no_water = first_citizen_happiness(&mut city_no_water);

    let drop = h_water - h_no_water;
    assert!(
        drop > 25.0,
        "No water should cause > 25 point drop, got {:.2} (with={:.2}, without={:.2})",
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

    let mut city_power = city_with_utilities(home, work);
    tick_with_stable_needs(&mut city_power);
    let h_power = first_citizen_happiness(&mut city_power);

    let mut city_no_power = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .with_citizen(home, work)
        .with_utility(home.0, home.1 - 1, UtilityType::WaterTower);
    tick_with_stable_needs(&mut city_no_power);
    let h_no_power = first_citizen_happiness(&mut city_no_power);

    let drop = h_power - h_no_power;
    assert!(
        drop > 20.0,
        "No power should cause > 20 point drop, got {:.2} (with={:.2}, without={:.2})",
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

    let mut city_poor = city_with_utilities(home, work);
    city_poor.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_poor, 50.0, 70.0);
    set_savings(&mut city_poor, 0.0);
    city_poor.tick(1);
    let h_poor = first_citizen_happiness(&mut city_poor);

    let mut city_rich = city_with_utilities(home, work);
    city_rich.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_rich, 50.0, 70.0);
    set_savings(&mut city_rich, 10_000.0);
    city_rich.tick(1);
    let h_rich = first_citizen_happiness(&mut city_rich);

    assert!(
        h_rich > h_poor,
        "Wealthy citizen ({:.2}) should be happier than poor ({:.2})",
        h_rich,
        h_poor
    );
    let gap = h_rich - h_poor;
    assert!(
        gap > 5.0,
        "Wealth gap should be > 5 happiness points, got {:.2}",
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

    let mut city_healthy = city_with_utilities(home, work);
    city_healthy.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_healthy, 50.0, 90.0);
    city_healthy.tick(1);
    let h_healthy = first_citizen_happiness(&mut city_healthy);

    let mut city_sick = city_with_utilities(home, work);
    city_sick.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_sick, 50.0, 15.0);
    city_sick.tick(1);
    let h_sick = first_citizen_happiness(&mut city_sick);

    let drop = h_healthy - h_sick;
    assert!(
        drop > 15.0,
        "Critical health should cause > 15 point drop, got {:.2} (healthy={:.2}, sick={:.2})",
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

// ====================================================================
// 9. Diminishing returns: pollution impact levels off at extreme values
// ====================================================================

#[test]
fn test_happiness_tuning_pollution_diminishing_returns() {
    let home = (100, 100);
    let work = (102, 100);

    // No pollution (baseline)
    let mut city_clean = city_with_utilities(home, work);
    city_clean.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_clean, 50.0, 70.0);
    city_clean.tick(1);
    let h_clean = first_citizen_happiness(&mut city_clean);

    // Moderate pollution: 100
    let mut city_moderate = city_with_utilities(home, work);
    {
        let world = city_moderate.world_mut();
        world
            .resource_mut::<crate::pollution::PollutionGrid>()
            .set(home.0, home.1, 100);
    }
    city_moderate.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_moderate, 50.0, 70.0);
    city_moderate.tick(1);
    let h_moderate = first_citizen_happiness(&mut city_moderate);

    // Max pollution: 255
    let mut city_max = city_with_utilities(home, work);
    {
        let world = city_max.world_mut();
        world
            .resource_mut::<crate::pollution::PollutionGrid>()
            .set(home.0, home.1, 255);
    }
    city_max.tick(HAPPINESS_TICKS - 1);
    set_needs_and_health(&mut city_max, 50.0, 70.0);
    city_max.tick(1);
    let h_max = first_citizen_happiness(&mut city_max);

    // More pollution should always be worse
    assert!(
        h_clean > h_moderate,
        "Clean ({:.2}) should be better than moderate pollution ({:.2})",
        h_clean,
        h_moderate
    );
    assert!(
        h_moderate >= h_max,
        "Moderate pollution ({:.2}) should be no worse than max ({:.2})",
        h_moderate,
        h_max
    );

    // Due to diminishing returns, the drop from 0->100 should be bigger
    // than the drop from 100->255 (going from bad to worse hurts less).
    let drop_first = h_clean - h_moderate;
    let drop_last = h_moderate - h_max;
    assert!(
        drop_first > drop_last,
        "Drop 0->100 ({:.2}) should exceed drop 100->255 ({:.2})",
        drop_first,
        drop_last
    );
}

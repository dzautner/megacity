//! Integration tests for the crime calculation system (TEST-041).
//!
//! Tests verify that:
//! - Low land value increases crime
//! - Police coverage reduces crime
//! - Crime rates stay in valid range (u8 0–25 for base crime)
//! - Zero police coverage results in high crime for low-value areas
//!
//! The crime system (`update_crime`) runs in FixedUpdate gated by
//! `SlowTickTimer::should_run()` (every 100 ticks). Tests use
//! `tick_slow_cycles()` to trigger it.
//!
//! **Important**: The land value system uses exponential smoothing
//! (alpha=0.1), so values converge slowly. Tests that need divergent land
//! values run multiple slow cycles to let the smoothing converge.

use crate::budget::ExtendedBudget;
use crate::crime::CrimeGrid;
use crate::grid::ZoneType;
use crate::immigration::CityAttractiveness;
use crate::land_value::LandValueGrid;
use crate::pollution::PollutionGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

/// Read crime level at (x, y).
fn crime_at(city: &TestCity, x: usize, y: usize) -> u8 {
    city.resource::<CrimeGrid>().get(x, y)
}

/// Read land value at (x, y).
fn land_value_at(city: &TestCity, x: usize, y: usize) -> u8 {
    city.resource::<LandValueGrid>().get(x, y)
}

/// Set police budget multiplier (default is 1.0).
fn set_police_budget(city: &mut TestCity, budget: f32) {
    let world = city.world_mut();
    world
        .resource_mut::<ExtendedBudget>()
        .service_budgets
        .police = budget;
}

/// Prevent emigration by setting attractiveness high.
fn prevent_emigration(city: &mut TestCity) {
    let world = city.world_mut();
    world.resource_mut::<CityAttractiveness>().overall_score = 80.0;
}

/// Add heavy pollution to an area to drive land value down.
fn pollute_area(city: &mut TestCity, x0: usize, y0: usize, x1: usize, y1: usize, amount: u8) {
    let world = city.world_mut();
    let mut pollution = world.resource_mut::<PollutionGrid>();
    for y in y0..=y1 {
        for x in x0..=x1 {
            pollution.set(x, y, amount);
        }
    }
}

/// Number of slow cycles to let land value smoothing converge sufficiently.
/// With alpha=0.1, after 20 cycles: value reaches ~87% of target.
const CONVERGENCE_CYCLES: u32 = 20;

// -----------------------------------------------------------------------
// 1. Low land value increases crime
// -----------------------------------------------------------------------

#[test]
fn test_crime_low_land_value_produces_higher_crime_than_high_land_value() {
    // Two zoned areas far apart: one polluted (drives land value down),
    // one with a park (drives land value up). Run enough cycles for
    // land values to converge.
    let mut city = TestCity::new()
        .with_zone_rect(30, 30, 35, 35, ZoneType::ResidentialLow)
        .with_zone_rect(130, 130, 135, 135, ZoneType::ResidentialLow)
        .with_service(132, 132, ServiceType::SmallPark);

    prevent_emigration(&mut city);
    // Heavy pollution in the low-value area (large region to avoid diffusion)
    pollute_area(&mut city, 20, 20, 50, 50, 255);

    city.tick_slow_cycles(CONVERGENCE_CYCLES);

    let lv_polluted = land_value_at(&city, 32, 32);
    let lv_park = land_value_at(&city, 132, 132);
    let crime_polluted = crime_at(&city, 32, 32);
    let crime_park = crime_at(&city, 132, 132);

    // Verify land values diverged
    assert!(
        lv_polluted < lv_park,
        "Polluted area should have lower land value ({lv_polluted}) than park area ({lv_park})"
    );

    // Verify crime is inversely related to land value
    assert!(
        crime_polluted > crime_park,
        "Polluted (low land value) area should have more crime ({crime_polluted}) \
         than park (high land value) area ({crime_park})"
    );
}

#[test]
fn test_crime_inversely_proportional_to_land_value() {
    // Verify the inverse relationship after convergence: heavily polluted
    // cells should have lower land value and higher crime.
    let mut city = TestCity::new()
        .with_zone_rect(50, 50, 55, 55, ZoneType::ResidentialLow)
        .with_zone_rect(150, 150, 155, 155, ZoneType::ResidentialLow);

    prevent_emigration(&mut city);
    // Pollute one area heavily (large region)
    pollute_area(&mut city, 40, 40, 65, 65, 255);

    city.tick_slow_cycles(CONVERGENCE_CYCLES);

    let lv_low = land_value_at(&city, 52, 52);
    let lv_high = land_value_at(&city, 152, 152);
    let crime_low_lv = crime_at(&city, 52, 52);
    let crime_high_lv = crime_at(&city, 152, 152);

    assert!(
        lv_low < lv_high,
        "Polluted area land value ({lv_low}) should be lower than clean area ({lv_high})"
    );
    assert!(
        crime_low_lv >= crime_high_lv,
        "Lower land value area should have >= crime ({crime_low_lv}) \
         compared to higher land value area ({crime_high_lv})"
    );
}

// -----------------------------------------------------------------------
// 2. Police coverage reduces crime
// -----------------------------------------------------------------------

#[test]
fn test_crime_police_station_reduces_nearby_crime() {
    // Compare crime at the same location with and without a police station.
    let (x, y) = (100, 100);

    // City WITHOUT police
    let mut city_no_police = TestCity::new()
        .with_zone(x, y, ZoneType::ResidentialLow);
    prevent_emigration(&mut city_no_police);
    pollute_area(&mut city_no_police, 90, 90, 110, 110, 255);
    city_no_police.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_no_police = crime_at(&city_no_police, x, y);

    // City WITH police station near the cell (same pollution)
    let mut city_with_police = TestCity::new()
        .with_zone(x, y, ZoneType::ResidentialLow)
        .with_service(x + 2, y, ServiceType::PoliceStation);
    prevent_emigration(&mut city_with_police);
    pollute_area(&mut city_with_police, 90, 90, 110, 110, 255);
    city_with_police.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_with_police = crime_at(&city_with_police, x, y);

    // Ensure there's some crime to reduce
    assert!(
        crime_no_police > 0,
        "Crime without police should be > 0, got {crime_no_police}"
    );

    assert!(
        crime_with_police < crime_no_police,
        "Police station should reduce crime: with police = {crime_with_police}, \
         without police = {crime_no_police}"
    );
}

#[test]
fn test_crime_police_hq_reduces_crime_more_than_kiosk() {
    // PoliceHQ has base_reduction=30 vs PoliceKiosk=10.
    let (x, y) = (100, 100);

    // City with PoliceKiosk
    let mut city_kiosk = TestCity::new()
        .with_zone(x, y, ZoneType::ResidentialLow)
        .with_service(x, y + 2, ServiceType::PoliceKiosk);
    prevent_emigration(&mut city_kiosk);
    pollute_area(&mut city_kiosk, 90, 90, 110, 110, 255);
    city_kiosk.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_kiosk = crime_at(&city_kiosk, x, y);

    // City with PoliceHQ
    let mut city_hq = TestCity::new()
        .with_zone(x, y, ZoneType::ResidentialLow)
        .with_service(x, y + 2, ServiceType::PoliceHQ);
    prevent_emigration(&mut city_hq);
    pollute_area(&mut city_hq, 90, 90, 110, 110, 255);
    city_hq.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_hq = crime_at(&city_hq, x, y);

    assert!(
        crime_hq <= crime_kiosk,
        "PoliceHQ should reduce crime at least as much as PoliceKiosk: \
         HQ = {crime_hq}, Kiosk = {crime_kiosk}"
    );
}

// -----------------------------------------------------------------------
// 3. Crime rate in valid range
// -----------------------------------------------------------------------

#[test]
fn test_crime_values_always_within_valid_range() {
    // Set up a city with a mix of zones and conditions, run slow cycles,
    // and verify all crime values are within expected bounds.
    let mut city = TestCity::new()
        .with_zone_rect(50, 50, 70, 70, ZoneType::ResidentialLow)
        .with_zone_rect(80, 80, 100, 100, ZoneType::CommercialLow)
        .with_zone_rect(110, 110, 130, 130, ZoneType::Industrial)
        .with_service(60, 60, ServiceType::PoliceStation);

    prevent_emigration(&mut city);

    // Heavy pollution in one area
    pollute_area(&mut city, 50, 50, 70, 70, 255);

    city.tick_slow_cycles(CONVERGENCE_CYCLES);

    let grid = city.resource::<CrimeGrid>();
    // Base crime formula: ((100 - lv).max(0) / 4) gives max 25.
    // Police can only subtract, so values should remain in [0, 25].
    for y in 0..grid.height {
        for x in 0..grid.width {
            let val = grid.get(x, y);
            assert!(
                val <= 25,
                "Crime at ({x}, {y}) = {val} exceeds maximum base crime of 25"
            );
        }
    }
}

// -----------------------------------------------------------------------
// 4. Zero police coverage results in high crime
// -----------------------------------------------------------------------

#[test]
fn test_crime_no_police_polluted_area_has_elevated_crime() {
    // A heavily polluted zoned area with NO police should have elevated crime.
    let mut city = TestCity::new()
        .with_zone_rect(90, 90, 110, 110, ZoneType::ResidentialLow);

    prevent_emigration(&mut city);
    // Maximum pollution over a large area
    pollute_area(&mut city, 80, 80, 120, 120, 255);

    city.tick_slow_cycles(CONVERGENCE_CYCLES);

    // With maximum pollution and no police, crime should be significant
    let crime_center = crime_at(&city, 100, 100);
    assert!(
        crime_center > 10,
        "Heavily polluted area with no police should have significant crime (>10), \
         got {crime_center}"
    );

    // All cells in the polluted zone should have non-zero crime
    for y in 90..=110 {
        for x in 90..=110 {
            let val = crime_at(&city, x, y);
            assert!(
                val > 0,
                "All cells in polluted zone should have crime > 0, \
                 but ({x}, {y}) has {val}"
            );
        }
    }
}

#[test]
fn test_crime_zero_police_budget_negates_police_effectiveness() {
    // With police budget at 0, police stations should provide no reduction.
    // Compare crime with budget=0 police vs no police at all.
    let (x, y) = (100, 100);

    // City with NO police
    let mut city_no_police =
        TestCity::new().with_zone(x, y, ZoneType::ResidentialLow);
    prevent_emigration(&mut city_no_police);
    pollute_area(&mut city_no_police, 90, 90, 110, 110, 255);
    city_no_police.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_no_police = crime_at(&city_no_police, x, y);

    // City WITH police but budget=0
    let mut city_zero_budget = TestCity::new()
        .with_zone(x, y, ZoneType::ResidentialLow)
        .with_service(x + 1, y, ServiceType::PoliceStation);
    prevent_emigration(&mut city_zero_budget);
    pollute_area(&mut city_zero_budget, 90, 90, 110, 110, 255);
    set_police_budget(&mut city_zero_budget, 0.0);
    city_zero_budget.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_zero_budget = crime_at(&city_zero_budget, x, y);

    // With zero budget, police should provide no reduction
    assert_eq!(
        crime_zero_budget, crime_no_police,
        "With zero police budget, crime ({crime_zero_budget}) should equal \
         no-police crime ({crime_no_police})"
    );
}

// -----------------------------------------------------------------------
// 5. Unzoned/empty cells have zero crime
// -----------------------------------------------------------------------

#[test]
fn test_crime_unzoned_cell_without_building_has_zero_crime() {
    // Cells with no zone and no building should always have crime = 0,
    // regardless of other conditions.
    let mut city = TestCity::new();
    prevent_emigration(&mut city);
    // Add pollution but no zones
    pollute_area(&mut city, 120, 120, 140, 140, 255);

    city.tick_slow_cycle();

    let crime = crime_at(&city, 130, 130);
    assert_eq!(
        crime, 0,
        "Unzoned cell with no building should have zero crime, got {crime}"
    );
}

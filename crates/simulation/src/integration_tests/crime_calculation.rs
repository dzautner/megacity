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
//! `tick_slow_cycle()` to trigger it.

use crate::budget::ExtendedBudget;
use crate::crime::CrimeGrid;
use crate::grid::ZoneType;
use crate::immigration::CityAttractiveness;
use crate::land_value::LandValueGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

/// Read crime level at (x, y).
fn crime_at(city: &TestCity, x: usize, y: usize) -> u8 {
    city.resource::<CrimeGrid>().get(x, y)
}

/// Set land value for a rectangular area.
fn set_land_value_rect(city: &mut TestCity, x0: usize, y0: usize, x1: usize, y1: usize, val: u8) {
    let world = city.world_mut();
    let mut lv = world.resource_mut::<LandValueGrid>();
    for y in y0..=y1 {
        for x in x0..=x1 {
            lv.set(x, y, val);
        }
    }
}

/// Set land value for a single cell.
fn set_land_value(city: &mut TestCity, x: usize, y: usize, val: u8) {
    let world = city.world_mut();
    world.resource_mut::<LandValueGrid>().set(x, y, val);
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

// -----------------------------------------------------------------------
// 1. Low land value increases crime
// -----------------------------------------------------------------------

#[test]
fn test_crime_low_land_value_produces_higher_crime_than_high_land_value() {
    // Create two zoned cells: one with low land value, one with high.
    let mut city = TestCity::new()
        .with_zone(100, 100, ZoneType::Residential)
        .with_zone(120, 120, ZoneType::Residential);

    prevent_emigration(&mut city);

    // Set land values: low (10) and high (90)
    set_land_value(&mut city, 100, 100, 10);
    set_land_value(&mut city, 120, 120, 90);

    city.tick_slow_cycle();

    let crime_low_value = crime_at(&city, 100, 100);
    let crime_high_value = crime_at(&city, 120, 120);

    assert!(
        crime_low_value > crime_high_value,
        "Low land value cell should have more crime ({crime_low_value}) \
         than high land value cell ({crime_high_value})"
    );
}

#[test]
fn test_crime_zero_land_value_produces_maximum_base_crime() {
    // A zoned cell with land value 0 should get the maximum base crime of 25.
    let mut city = TestCity::new().with_zone(100, 100, ZoneType::Commercial);

    prevent_emigration(&mut city);
    set_land_value(&mut city, 100, 100, 0);

    city.tick_slow_cycle();

    let crime = crime_at(&city, 100, 100);
    // Base formula: ((100 - 0).max(0) / 4) = 25
    assert_eq!(
        crime, 25,
        "Zero land value should produce base crime of 25, got {crime}"
    );
}

#[test]
fn test_crime_max_land_value_produces_zero_base_crime() {
    // A zoned cell with land value >= 100 should get zero base crime.
    let mut city = TestCity::new().with_zone(100, 100, ZoneType::Residential);

    prevent_emigration(&mut city);
    set_land_value(&mut city, 100, 100, 100);

    city.tick_slow_cycle();

    let crime = crime_at(&city, 100, 100);
    assert_eq!(
        crime, 0,
        "Land value of 100 should produce zero base crime, got {crime}"
    );
}

// -----------------------------------------------------------------------
// 2. Police coverage reduces crime
// -----------------------------------------------------------------------

#[test]
fn test_crime_police_station_reduces_nearby_crime() {
    // Place a zoned cell with low land value, then add a police station
    // and verify crime is reduced compared to a city without police.
    let (x, y) = (100, 100);

    // City WITHOUT police
    let mut city_no_police = TestCity::new().with_zone(x, y, ZoneType::Residential);
    prevent_emigration(&mut city_no_police);
    set_land_value(&mut city_no_police, x, y, 10);
    city_no_police.tick_slow_cycle();
    let crime_no_police = crime_at(&city_no_police, x, y);

    // City WITH police station near the cell
    let mut city_with_police = TestCity::new()
        .with_zone(x, y, ZoneType::Residential)
        .with_service(x + 2, y, ServiceType::PoliceStation);
    prevent_emigration(&mut city_with_police);
    set_land_value(&mut city_with_police, x, y, 10);
    city_with_police.tick_slow_cycle();
    let crime_with_police = crime_at(&city_with_police, x, y);

    assert!(
        crime_with_police < crime_no_police,
        "Police station should reduce crime: with police = {crime_with_police}, \
         without police = {crime_no_police}"
    );
}

#[test]
fn test_crime_police_hq_reduces_crime_more_than_kiosk() {
    // PoliceHQ has base_reduction=30 vs PoliceKiosk=10.
    // Both placed at the same location, comparing reduction at a nearby cell.
    let (x, y) = (100, 100);

    // City with PoliceKiosk
    let mut city_kiosk = TestCity::new()
        .with_zone(x, y, ZoneType::Residential)
        .with_service(x, y + 2, ServiceType::PoliceKiosk);
    prevent_emigration(&mut city_kiosk);
    set_land_value(&mut city_kiosk, x, y, 0);
    city_kiosk.tick_slow_cycle();
    let crime_kiosk = crime_at(&city_kiosk, x, y);

    // City with PoliceHQ
    let mut city_hq = TestCity::new()
        .with_zone(x, y, ZoneType::Residential)
        .with_service(x, y + 2, ServiceType::PoliceHQ);
    prevent_emigration(&mut city_hq);
    set_land_value(&mut city_hq, x, y, 0);
    city_hq.tick_slow_cycle();
    let crime_hq = crime_at(&city_hq, x, y);

    assert!(
        crime_hq < crime_kiosk,
        "PoliceHQ should reduce crime more than PoliceKiosk: \
         HQ = {crime_hq}, Kiosk = {crime_kiosk}"
    );
}

// -----------------------------------------------------------------------
// 3. Crime rate in valid range
// -----------------------------------------------------------------------

#[test]
fn test_crime_values_always_within_valid_u8_range() {
    // Set up a city with a mix of zones and land values, run several slow
    // cycles, and verify all crime values are within expected bounds.
    let mut city = TestCity::new()
        .with_zone_rect(50, 50, 70, 70, ZoneType::Residential)
        .with_zone_rect(80, 80, 100, 100, ZoneType::Commercial)
        .with_zone_rect(110, 110, 130, 130, ZoneType::Industrial)
        .with_service(60, 60, ServiceType::PoliceStation);

    prevent_emigration(&mut city);

    // Set diverse land values
    set_land_value_rect(&mut city, 50, 50, 70, 70, 5); // very low
    set_land_value_rect(&mut city, 80, 80, 100, 100, 50); // medium
    set_land_value_rect(&mut city, 110, 110, 130, 130, 95); // high

    city.tick_slow_cycle();

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
fn test_crime_no_police_low_land_value_area_has_high_crime() {
    // A low-value zoned area with NO police should have crime at the
    // maximum base level (25 for land value 0).
    let mut city = TestCity::new().with_zone_rect(90, 90, 110, 110, ZoneType::Residential);

    prevent_emigration(&mut city);
    set_land_value_rect(&mut city, 90, 90, 110, 110, 0);

    city.tick_slow_cycle();

    // Check the center of the area
    let crime_center = crime_at(&city, 100, 100);
    assert_eq!(
        crime_center, 25,
        "Low-value area with no police should have max base crime (25), got {crime_center}"
    );

    // Check multiple cells to confirm the entire area has high crime
    for y in 90..=110 {
        for x in 90..=110 {
            let val = crime_at(&city, x, y);
            assert_eq!(
                val, 25,
                "All cells in the low-value zone should have crime 25, \
                 but ({x}, {y}) has {val}"
            );
        }
    }
}

#[test]
fn test_crime_zero_police_budget_negates_police_coverage() {
    // Even with a police station present, setting the police budget to 0
    // should result in zero reduction (base_reduction * 0.0 = 0).
    let (x, y) = (100, 100);

    let mut city = TestCity::new()
        .with_zone(x, y, ZoneType::Residential)
        .with_service(x + 1, y, ServiceType::PoliceStation);

    prevent_emigration(&mut city);
    set_land_value(&mut city, x, y, 0);
    set_police_budget(&mut city, 0.0);

    city.tick_slow_cycle();

    let crime = crime_at(&city, x, y);
    assert_eq!(
        crime, 25,
        "With zero police budget, crime should remain at base (25), got {crime}"
    );
}

// -----------------------------------------------------------------------
// 5. Unzoned cells have zero crime
// -----------------------------------------------------------------------

#[test]
fn test_crime_unzoned_cell_without_building_has_zero_crime() {
    // Cells with no zone and no building should always have crime = 0,
    // regardless of land value.
    let mut city = TestCity::new();

    prevent_emigration(&mut city);
    set_land_value(&mut city, 128, 128, 0); // lowest possible land value

    city.tick_slow_cycle();

    let crime = crime_at(&city, 128, 128);
    assert_eq!(
        crime, 0,
        "Unzoned cell with no building should have zero crime, got {crime}"
    );
}

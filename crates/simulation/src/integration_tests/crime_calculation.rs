//! Integration tests for the crime calculation system (TEST-041).
//!
//! Tests verify that:
//! - Low land value increases crime
//! - Police coverage reduces crime
//! - Crime rates stay in valid range (u8 0–25 for base crime)
//! - Zero police coverage results in high crime for low-value areas
//!
//! The crime system (`update_crime`) runs in FixedUpdate gated by
//! `SlowTickTimer::should_run()` (every 100 ticks).
//!
//! **Note**: Both pollution and land value are recomputed each slow tick.
//! Manual pollution injections are overwritten. Tests use persistent
//! conditions (industrial buildings for pollution, parks for value boosts)
//! to create divergent land values that survive across ticks.

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

/// Number of slow cycles to let land value smoothing converge.
const CONVERGENCE_CYCLES: u32 = 20;

// -----------------------------------------------------------------------
// 1. Low land value increases crime
// -----------------------------------------------------------------------

#[test]
fn test_crime_low_land_value_produces_higher_crime_than_high_land_value() {
    // Area A: surrounded by industrial buildings (pollution -> low land value).
    // Area B: near a park (boosts land value).
    // Crime should be higher in area A than area B.
    let mut city = TestCity::new()
        // Area A: zoned residential, surrounded by industrial buildings
        .with_zone(50, 50, ZoneType::ResidentialLow)
        .with_building(48, 50, ZoneType::Industrial, 3)
        .with_building(52, 50, ZoneType::Industrial, 3)
        .with_building(50, 48, ZoneType::Industrial, 3)
        .with_building(50, 52, ZoneType::Industrial, 3)
        // Area B: zoned residential near a park (far from industry)
        .with_zone(150, 150, ZoneType::ResidentialLow)
        .with_service(152, 150, ServiceType::SmallPark);

    prevent_emigration(&mut city);
    city.tick_slow_cycles(CONVERGENCE_CYCLES);

    let lv_industry = land_value_at(&city, 50, 50);
    let lv_park = land_value_at(&city, 150, 150);
    let crime_industry = crime_at(&city, 50, 50);
    let crime_park = crime_at(&city, 150, 150);

    assert!(
        lv_industry < lv_park,
        "Industrial area should have lower land value ({lv_industry}) \
         than park area ({lv_park})"
    );
    assert!(
        crime_industry > crime_park,
        "Industrial (low land value) area should have more crime ({crime_industry}) \
         than park (high land value) area ({crime_park})"
    );
}

#[test]
fn test_crime_inversely_proportional_to_land_value() {
    // Compare two residential cells: one near heavy industry, one in a
    // clean area. After convergence, the industrial neighbor should have
    // lower land value and higher crime.
    let mut city = TestCity::new()
        // Near industry
        .with_zone(60, 60, ZoneType::ResidentialLow)
        .with_building(58, 60, ZoneType::Industrial, 3)
        .with_building(62, 60, ZoneType::Industrial, 3)
        .with_building(60, 58, ZoneType::Industrial, 3)
        .with_building(60, 62, ZoneType::Industrial, 3)
        .with_building(58, 58, ZoneType::Industrial, 3)
        .with_building(62, 62, ZoneType::Industrial, 3)
        // Clean area (no industry nearby)
        .with_zone(180, 180, ZoneType::ResidentialLow);

    prevent_emigration(&mut city);
    city.tick_slow_cycles(CONVERGENCE_CYCLES);

    let lv_industrial = land_value_at(&city, 60, 60);
    let lv_clean = land_value_at(&city, 180, 180);
    let crime_industrial = crime_at(&city, 60, 60);
    let crime_clean = crime_at(&city, 180, 180);

    assert!(
        lv_industrial < lv_clean,
        "Near-industry land value ({lv_industrial}) should be lower \
         than clean area ({lv_clean})"
    );
    assert!(
        crime_industrial >= crime_clean,
        "Near-industry crime ({crime_industrial}) should be >= \
         clean area crime ({crime_clean})"
    );
}

// -----------------------------------------------------------------------
// 2. Police coverage reduces crime
// -----------------------------------------------------------------------

#[test]
fn test_crime_police_station_reduces_nearby_crime() {
    // Compare crime at the same location with and without a police station.
    // Use industrial buildings to ensure there's crime to reduce.
    let (x, y) = (100, 100);

    // City WITHOUT police
    let mut city_no_police = TestCity::new()
        .with_zone(x, y, ZoneType::ResidentialLow)
        .with_building(x - 2, y, ZoneType::Industrial, 3)
        .with_building(x + 2, y, ZoneType::Industrial, 3);
    prevent_emigration(&mut city_no_police);
    city_no_police.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_no_police = crime_at(&city_no_police, x, y);

    // City WITH police station near the cell (same industrial buildings)
    let mut city_with_police = TestCity::new()
        .with_zone(x, y, ZoneType::ResidentialLow)
        .with_building(x - 2, y, ZoneType::Industrial, 3)
        .with_building(x + 2, y, ZoneType::Industrial, 3)
        .with_service(x, y + 1, ServiceType::PoliceStation);
    prevent_emigration(&mut city_with_police);
    city_with_police.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_with_police = crime_at(&city_with_police, x, y);

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
        .with_building(x - 2, y, ZoneType::Industrial, 3)
        .with_building(x + 2, y, ZoneType::Industrial, 3)
        .with_service(x, y + 2, ServiceType::PoliceKiosk);
    prevent_emigration(&mut city_kiosk);
    city_kiosk.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_kiosk = crime_at(&city_kiosk, x, y);

    // City with PoliceHQ
    let mut city_hq = TestCity::new()
        .with_zone(x, y, ZoneType::ResidentialLow)
        .with_building(x - 2, y, ZoneType::Industrial, 3)
        .with_building(x + 2, y, ZoneType::Industrial, 3)
        .with_service(x, y + 2, ServiceType::PoliceHQ);
    prevent_emigration(&mut city_hq);
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
        .with_building(55, 55, ZoneType::Industrial, 3)
        .with_building(65, 65, ZoneType::Industrial, 3)
        .with_service(90, 90, ServiceType::PoliceStation);

    prevent_emigration(&mut city);
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
fn test_crime_no_police_industrial_area_has_elevated_crime() {
    // An industrial area with NO police should have elevated crime due
    // to pollution driving land values down.
    let mut city = TestCity::new()
        .with_zone_rect(95, 95, 105, 105, ZoneType::ResidentialLow)
        .with_building(93, 100, ZoneType::Industrial, 3)
        .with_building(107, 100, ZoneType::Industrial, 3)
        .with_building(100, 93, ZoneType::Industrial, 3)
        .with_building(100, 107, ZoneType::Industrial, 3);

    prevent_emigration(&mut city);
    city.tick_slow_cycles(CONVERGENCE_CYCLES);

    let crime_center = crime_at(&city, 100, 100);
    assert!(
        crime_center > 0,
        "Industrial area with no police should have crime > 0, got {crime_center}"
    );
}

#[test]
fn test_crime_zero_police_budget_negates_police_effectiveness() {
    // With police budget at 0, police stations should provide no crime
    // reduction. Compare zero-budget police vs full-budget police: zero
    // budget should result in MORE crime.
    let (x, y) = (100, 100);

    // City WITH police at full budget (default=1.0)
    let mut city_full_budget = TestCity::new()
        .with_zone(x, y, ZoneType::ResidentialLow)
        .with_building(x - 2, y, ZoneType::Industrial, 3)
        .with_building(x + 2, y, ZoneType::Industrial, 3)
        .with_service(x, y + 1, ServiceType::PoliceStation);
    prevent_emigration(&mut city_full_budget);
    city_full_budget.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_full_budget = crime_at(&city_full_budget, x, y);

    // City WITH police but budget=0
    let mut city_zero_budget = TestCity::new()
        .with_zone(x, y, ZoneType::ResidentialLow)
        .with_building(x - 2, y, ZoneType::Industrial, 3)
        .with_building(x + 2, y, ZoneType::Industrial, 3)
        .with_service(x, y + 1, ServiceType::PoliceStation);
    prevent_emigration(&mut city_zero_budget);
    set_police_budget(&mut city_zero_budget, 0.0);
    city_zero_budget.tick_slow_cycles(CONVERGENCE_CYCLES);
    let crime_zero_budget = crime_at(&city_zero_budget, x, y);

    // Zero budget should negate police effectiveness -> more crime
    assert!(
        crime_zero_budget > crime_full_budget,
        "Zero police budget should result in more crime ({crime_zero_budget}) \
         than full budget ({crime_full_budget})"
    );
}

// -----------------------------------------------------------------------
// 5. Unzoned/empty cells have zero crime
// -----------------------------------------------------------------------

#[test]
fn test_crime_unzoned_cell_without_building_has_zero_crime() {
    // Cells with no zone and no building should always have crime = 0.
    let mut city = TestCity::new()
        .with_building(130, 130, ZoneType::Industrial, 3);

    prevent_emigration(&mut city);
    city.tick_slow_cycle();

    // Cell near industry but NOT zoned
    let crime = crime_at(&city, 128, 128);
    assert_eq!(
        crime, 0,
        "Unzoned cell with no building should have zero crime, got {crime}"
    );
}

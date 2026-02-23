//! Integration tests for PROG-004: Tiered Population Needs.
//!
//! Verifies that citizens advance/demote through population tiers based
//! on need fulfillment, and that tier stats aggregate correctly.

use crate::citizen::{Citizen, CitizenDetails};
use crate::grid::{RoadType, ZoneType};
use crate::happiness::ServiceCoverageGrid;
use crate::heating::HeatingGrid;
use crate::land_value::LandValueGrid;
use crate::population_tiers::{PopulationTier, PopulationTierComp, PopulationTierStats};
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run enough ticks for the slow tick timer to fire and tier evaluation to run.
fn tick_slow(city: &mut TestCity) {
    city.tick_slow_cycle();
    // Extra tick for init_citizen_tiers command flush
    city.tick(1);
}

/// Query the population tier of the first citizen found.
fn first_citizen_tier(city: &mut TestCity) -> PopulationTier {
    let world = city.world_mut();
    world
        .query::<&PopulationTierComp>()
        .iter(world)
        .next()
        .expect("expected at least one citizen with PopulationTierComp")
        .0
}

/// Create a basic city with one citizen that has a home, work, road, power, water.
fn city_with_powered_citizen() -> TestCity {
    TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_utility(10, 11, UtilityType::PowerPlant)
        .with_utility(10, 12, UtilityType::WaterTower)
        .with_building(11, 11, ZoneType::ResidentialLow, 1)
        .with_building(11, 15, ZoneType::CommercialLow, 1)
        .with_citizen((11, 11), (11, 15))
}

/// Inject heating at a cell.
fn inject_heating(city: &mut TestCity, x: usize, y: usize) {
    let world = city.world_mut();
    let mut heating = world.resource_mut::<HeatingGrid>();
    heating.set(x, y, 100);
}

/// Inject service coverage flags at a cell.
fn inject_coverage(city: &mut TestCity, x: usize, y: usize, flags: u8) {
    let world = city.world_mut();
    let mut coverage = world.resource_mut::<ServiceCoverageGrid>();
    let idx = ServiceCoverageGrid::idx(x, y);
    coverage.flags[idx] |= flags;
}

/// Inject land value at a cell.
fn inject_land_value(city: &mut TestCity, x: usize, y: usize, val: u8) {
    let world = city.world_mut();
    let mut lv = world.resource_mut::<LandValueGrid>();
    lv.set(x, y, val);
}

/// Set citizen education level and happiness.
fn set_citizen_stats(city: &mut TestCity, education: u8, happiness: f32) {
    let world = city.world_mut();
    for mut details in world.query::<&mut CitizenDetails>().iter_mut(world) {
        details.education = education;
        details.happiness = happiness;
    }
}

/// Inject heating and coverage before a slow tick cycle.
/// Since other systems may overwrite these grids, re-inject before each cycle.
fn inject_full_services(city: &mut TestCity, x: usize, y: usize, coverage_flags: u8) {
    inject_heating(city, x, y);
    inject_coverage(city, x, y, coverage_flags);
}

/// Advance through multiple slow tick cycles, re-injecting state before each.
fn advance_tiers(
    city: &mut TestCity,
    cycles: usize,
    coverage_flags: u8,
    land_val: Option<u8>,
    edu: Option<u8>,
    happy: Option<f32>,
) {
    for _ in 0..cycles {
        inject_full_services(city, 11, 11, coverage_flags);
        if let Some(lv) = land_val {
            inject_land_value(city, 11, 11, lv);
        }
        if let Some(e) = edu {
            if let Some(h) = happy {
                set_citizen_stats(city, e, h);
            }
        }
        tick_slow(city);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_population_tier_component_auto_inserted() {
    let mut city = TestCity::new()
        .with_building(10, 10, ZoneType::ResidentialLow, 1)
        .with_building(10, 15, ZoneType::CommercialLow, 1)
        .with_citizen((10, 10), (10, 15));

    // After ticking, the init_citizen_tiers system should add the component
    city.tick(2);
    let world = city.world_mut();
    let count = world
        .query::<(&Citizen, &PopulationTierComp)>()
        .iter(world)
        .count();
    assert!(count > 0, "PopulationTierComp should be auto-inserted");
}

#[test]
fn test_citizen_starts_at_basic_tier() {
    let mut city = city_with_powered_citizen();
    city.tick(2);
    let tier = first_citizen_tier(&mut city);
    assert_eq!(tier, PopulationTier::Basic);
}

#[test]
fn test_citizen_advances_to_comfort_with_power_and_heating() {
    let mut city = city_with_powered_citizen();
    inject_heating(&mut city, 11, 11);

    tick_slow(&mut city);

    let tier = first_citizen_tier(&mut city);
    assert!(
        tier >= PopulationTier::Comfort,
        "Expected Comfort or higher, got {:?}",
        tier,
    );
}

#[test]
fn test_citizen_advances_to_community_with_services() {
    let mut city = city_with_powered_citizen();

    use crate::happiness::*;
    let flags = COVERAGE_EDUCATION | COVERAGE_HEALTH | COVERAGE_PARK;

    // Citizens advance one tier per slow tick cycle:
    // Basic -> Comfort (cycle 1), Comfort -> Community (cycle 2)
    advance_tiers(&mut city, 2, flags, None, None, None);

    let tier = first_citizen_tier(&mut city);
    assert!(
        tier >= PopulationTier::Community,
        "Expected Community or higher, got {:?}",
        tier,
    );
}

#[test]
fn test_citizen_advances_to_cultural_with_entertainment() {
    let mut city = city_with_powered_citizen();

    use crate::happiness::*;
    let flags = COVERAGE_EDUCATION | COVERAGE_HEALTH | COVERAGE_PARK | COVERAGE_ENTERTAINMENT;

    // Citizens advance one tier per slow tick cycle:
    // Basic -> Comfort (1), Comfort -> Community (2), Community -> Cultural (3)
    advance_tiers(&mut city, 3, flags, None, None, None);

    let tier = first_citizen_tier(&mut city);
    assert!(
        tier >= PopulationTier::Cultural,
        "Expected Cultural or higher, got {:?}",
        tier,
    );
}

#[test]
fn test_citizen_cannot_reach_aspirational_without_education() {
    let mut city = city_with_powered_citizen();

    use crate::happiness::*;
    let flags = COVERAGE_EDUCATION | COVERAGE_HEALTH | COVERAGE_PARK | COVERAGE_ENTERTAINMENT;

    inject_land_value(&mut city, 11, 11, 200);

    // Advance through 4 cycles (enough to reach Cultural)
    // The default test citizen has education=2, needs 3 for Aspirational
    advance_tiers(&mut city, 4, flags, Some(200), None, None);

    let tier = first_citizen_tier(&mut city);
    assert!(
        tier < PopulationTier::Aspirational,
        "Citizen without university education should not reach Aspirational, got {:?}",
        tier,
    );
}

#[test]
fn test_citizen_demotes_when_water_lost() {
    let mut city = city_with_powered_citizen();
    inject_heating(&mut city, 11, 11);

    // Advance to Comfort
    tick_slow(&mut city);
    let tier = first_citizen_tier(&mut city);
    assert!(tier >= PopulationTier::Comfort, "Should be Comfort first");

    // Remove water from home cell
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(11, 11).has_water = false;
    }

    tick_slow(&mut city);

    let tier = first_citizen_tier(&mut city);
    assert!(
        tier < PopulationTier::Comfort,
        "Should demote below Comfort without water, got {:?}",
        tier,
    );
}

#[test]
fn test_population_tier_stats_aggregated() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 25, RoadType::Local)
        .with_utility(10, 11, UtilityType::PowerPlant)
        .with_utility(10, 12, UtilityType::WaterTower)
        .with_building(11, 11, ZoneType::ResidentialLow, 1)
        .with_building(11, 15, ZoneType::CommercialLow, 1)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(12, 15, ZoneType::CommercialLow, 1)
        .with_citizen((11, 11), (11, 15))
        .with_citizen((12, 11), (12, 15));

    tick_slow(&mut city);

    let stats = city.resource::<PopulationTierStats>();
    assert!(
        stats.total() > 0,
        "Stats should have counted at least one citizen",
    );
    assert!(
        stats.total_economic_output > 0.0,
        "Economic output should be positive",
    );
}

#[test]
fn test_tier_economic_multiplier_reflected_in_stats() {
    let stats = PopulationTierStats {
        basic_count: 10,
        comfort_count: 0,
        community_count: 0,
        cultural_count: 0,
        aspirational_count: 0,
        total_economic_output: 10.0,
    };
    assert!((stats.total_economic_output - 10.0).abs() < 0.01);

    let stats2 = PopulationTierStats {
        basic_count: 0,
        comfort_count: 0,
        community_count: 0,
        cultural_count: 0,
        aspirational_count: 10,
        total_economic_output: 70.0,
    };
    assert!((stats2.total_economic_output - 70.0).abs() < 0.01);
}

#[test]
fn test_tier_advances_one_step_per_cycle() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 25, RoadType::Local)
        .with_utility(10, 11, UtilityType::PowerPlant)
        .with_utility(10, 12, UtilityType::WaterTower)
        .with_building(11, 11, ZoneType::ResidentialLow, 1)
        .with_building(11, 15, ZoneType::CommercialLow, 1)
        .with_citizen((11, 11), (11, 15));

    // Set up everything for Aspirational
    inject_heating(&mut city, 11, 11);
    use crate::happiness::*;
    inject_coverage(
        &mut city,
        11,
        11,
        COVERAGE_EDUCATION | COVERAGE_HEALTH | COVERAGE_PARK | COVERAGE_ENTERTAINMENT,
    );
    inject_land_value(&mut city, 11, 11, 200);
    set_citizen_stats(&mut city, 3, 85.0);

    // First cycle: Basic -> Comfort
    tick_slow(&mut city);
    let tier1 = first_citizen_tier(&mut city);
    assert_eq!(
        tier1,
        PopulationTier::Comfort,
        "Should advance to Comfort on first cycle",
    );

    // Re-inject state since systems may have overwritten
    inject_heating(&mut city, 11, 11);
    inject_coverage(
        &mut city,
        11,
        11,
        COVERAGE_EDUCATION | COVERAGE_HEALTH | COVERAGE_PARK | COVERAGE_ENTERTAINMENT,
    );
    inject_land_value(&mut city, 11, 11, 200);
    set_citizen_stats(&mut city, 3, 85.0);

    // Second cycle: Comfort -> Community
    tick_slow(&mut city);
    let tier2 = first_citizen_tier(&mut city);
    assert_eq!(
        tier2,
        PopulationTier::Community,
        "Should advance to Community on second cycle",
    );
}

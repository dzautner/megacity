//! TEST-016: Integration Test — Full City Growth Pipeline
//!
//! Ultimate integration test: empty world -> place cross-shaped roads ->
//! zone Residential and Commercial -> run 500 ticks -> verify buildings,
//! citizens, economy, and traffic all functioning.

use crate::economy::CityBudget;
use crate::grid::{RoadType, ZoneType};
use crate::immigration::CityAttractiveness;
use crate::stats::CityStats;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::traffic::TrafficGrid;
use crate::utilities::UtilityType;

/// Build a cross-shaped road network centered at (128, 128) with:
/// - Horizontal road from (108, 128) to (148, 128)
/// - Vertical road from (128, 108) to (128, 148)
/// Zone residential in the top-left quadrant and commercial in the
/// top-right quadrant (both adjacent to roads for utility BFS reach).
fn build_cross_city() -> TestCity {
    TestCity::new()
        .with_budget(500_000.0)
        // Cross-shaped road network
        .with_road(108, 128, 148, 128, RoadType::Avenue) // horizontal
        .with_road(128, 108, 128, 148, RoadType::Avenue) // vertical
        // Residential zone: top-left quadrant, adjacent to both roads
        .with_zone_rect(110, 125, 126, 127, ZoneType::ResidentialLow)
        // Commercial zone: top-right quadrant, adjacent to both roads
        .with_zone_rect(130, 125, 146, 127, ZoneType::CommercialLow)
        // Utilities on the road for BFS propagation
        .with_utility(108, 128, UtilityType::PowerPlant)
        .with_utility(148, 128, UtilityType::WaterTower)
}

/// Helper: boost city attractiveness so immigration fires during tests.
/// The attractiveness system recalculates every 50 ticks, so we set a
/// high score that should persist even after recalculation since the
/// city has buildings and zones.
fn boost_attractiveness(city: &mut TestCity) {
    let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
    attr.overall_score = 90.0;
    attr.housing_factor = 1.0;
    attr.employment_factor = 1.0;
}

/// Core acceptance test: empty world -> cross roads -> zone R + C ->
/// run 500 ticks -> verify buildings spawned.
#[test]
fn test_full_growth_pipeline_buildings_spawn() {
    let mut city = build_cross_city();

    assert_eq!(city.building_count(), 0, "No buildings before simulation");
    assert_eq!(city.citizen_count(), 0, "No citizens before simulation");

    boost_attractiveness(&mut city);
    city.tick(500);

    let building_count = city.building_count();
    assert!(
        building_count > 0,
        "After 500 ticks with cross roads, zones, and utilities, \
         buildings should have spawned. Got {building_count}"
    );
}

/// Verify that both residential and commercial buildings appear.
#[test]
fn test_full_growth_pipeline_mixed_zones_produce_buildings() {
    let mut city = build_cross_city();
    boost_attractiveness(&mut city);

    city.tick(500);

    let residential = city.buildings_in_zone(ZoneType::ResidentialLow);
    let commercial = city.buildings_in_zone(ZoneType::CommercialLow);

    let total = residential + commercial;
    assert!(
        total > 0,
        "At least one building type should spawn. \
         Residential={residential}, Commercial={commercial}"
    );
}

/// Verify citizens appear after buildings are completed. We run enough
/// ticks for buildings to spawn and complete construction, then
/// re-boost attractiveness and run more ticks for immigration.
#[test]
fn test_full_growth_pipeline_citizens_appear() {
    let mut city = build_cross_city();
    boost_attractiveness(&mut city);

    // Phase 1: Let buildings spawn and complete construction (~300 ticks)
    city.tick(300);

    // Re-boost attractiveness (system may have recalculated to a lower
    // value since the city is small)
    boost_attractiveness(&mut city);

    // Phase 2: Immigration should fire now that completed buildings exist
    city.tick(200);

    let citizen_count = city.citizen_count();
    assert!(
        citizen_count > 0,
        "After 500 ticks with residential buildings, citizens should \
         have immigrated. Got {citizen_count}"
    );
}

/// Verify that the economy is functioning: either monthly_income > 0 or
/// last_collection_day > 0 (taxes have been collected at least once).
/// We advance the game clock to day 32 so that the tax collection
/// interval (30 days) is reached within the remaining tick window.
#[test]
fn test_full_growth_pipeline_economy_active() {
    let mut city = build_cross_city();
    boost_attractiveness(&mut city);

    // Phase 1: Let buildings spawn and complete construction
    city.tick(300);

    // Re-boost attractiveness and advance clock past tax collection threshold
    boost_attractiveness(&mut city);
    {
        let mut clock = city.world_mut().resource_mut::<GameClock>();
        // Set day to 32 so collect_taxes fires (checks clock.day > last_collection_day + 30)
        clock.day = 32;
    }

    // Phase 2: Immigration + tax collection
    city.tick(200);

    let budget = city.resource::<CityBudget>();
    let income_positive = budget.monthly_income > 0.0;
    let taxes_collected = budget.last_collection_day > 0;

    assert!(
        income_positive || taxes_collected,
        "Economy should be active after tax collection triggers. \
         monthly_income={}, last_collection_day={}",
        budget.monthly_income,
        budget.last_collection_day
    );
}

/// Verify CityStats.population matches citizen count after 500 ticks.
/// CityStats.population includes virtual population, so it should be
/// >= the actual citizen entity count.
#[test]
fn test_full_growth_pipeline_population_stats_consistent() {
    let mut city = build_cross_city();
    boost_attractiveness(&mut city);

    city.tick(300);
    boost_attractiveness(&mut city);
    city.tick(200);

    let citizen_count = city.citizen_count();
    let stats_pop = city.resource::<CityStats>().population;

    assert!(
        stats_pop >= citizen_count as u32,
        "CityStats.population ({stats_pop}) should be >= citizen entity count ({citizen_count})"
    );
}

/// Verify the TrafficGrid resource exists and is initialized, confirming
/// the traffic system is wired up and functioning.
#[test]
fn test_full_growth_pipeline_traffic_grid_exists() {
    let mut city = build_cross_city();
    boost_attractiveness(&mut city);

    city.tick(500);

    let traffic = city.resource::<TrafficGrid>();
    assert_eq!(
        traffic.width, 256,
        "TrafficGrid width should match grid size"
    );
    assert_eq!(
        traffic.height, 256,
        "TrafficGrid height should match grid size"
    );
}

/// Verify road cells exist on the grid after placement, confirming the
/// cross-shaped road network was rasterized correctly.
#[test]
fn test_full_growth_pipeline_road_network_established() {
    let city = build_cross_city();

    let road_count = city.road_cell_count();
    assert!(
        road_count > 0,
        "Cross-shaped roads should produce road cells. Got {road_count}"
    );

    // Check a cell at the intersection of the cross
    city.assert_has_road(128, 128);
}

/// Combined end-to-end assertion: all acceptance criteria in one test.
/// This is the "ultimate" integration test from the issue description.
#[test]
fn test_full_growth_pipeline_all_criteria() {
    let mut city = build_cross_city();
    boost_attractiveness(&mut city);

    // Verify empty starting state
    assert_eq!(city.building_count(), 0, "No buildings at start");
    assert_eq!(city.citizen_count(), 0, "No citizens at start");

    // Phase 1: Let buildings spawn and complete construction (300 ticks)
    city.tick(300);

    // Phase 2: Re-boost attractiveness and advance clock for tax collection
    boost_attractiveness(&mut city);
    {
        let mut clock = city.world_mut().resource_mut::<GameClock>();
        clock.day = 32;
    }
    city.tick(200);

    // AC1: Cross-shaped road network placed
    let road_count = city.road_cell_count();
    assert!(
        road_count > 0,
        "Roads should be rasterized to grid. Got {road_count}"
    );

    // AC2: Residential and Commercial zones present
    let res_zones = city.zoned_cell_count(ZoneType::ResidentialLow);
    let com_zones = city.zoned_cell_count(ZoneType::CommercialLow);
    assert!(res_zones > 0, "Residential zones should exist");
    assert!(com_zones > 0, "Commercial zones should exist");

    // AC3: buildings.len() > 0
    let building_count = city.building_count();
    assert!(
        building_count > 0,
        "After 500 ticks, buildings should have spawned. Got {building_count}"
    );

    // AC4: citizens.len() > 0
    let citizen_count = city.citizen_count();
    assert!(
        citizen_count > 0,
        "After 500 ticks, citizens should have immigrated. Got {citizen_count}"
    );

    // AC5: budget.monthly_income > 0 or last_collection_day > 0
    let budget = city.resource::<CityBudget>();
    assert!(
        budget.monthly_income > 0.0 || budget.last_collection_day > 0,
        "Economy should be active. monthly_income={}, last_collection_day={}",
        budget.monthly_income,
        budget.last_collection_day
    );

    // AC6: CityStats.population matches citizen count
    // (population includes virtual pop, so >= citizen entities)
    let stats = city.resource::<CityStats>();
    assert!(
        stats.population >= citizen_count as u32,
        "CityStats.population ({}) should be >= citizen count ({citizen_count})",
        stats.population
    );
}

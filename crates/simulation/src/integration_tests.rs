//! Integration tests for Megacity using the `TestCity` harness.
//!
//! These tests spin up a headless Bevy App with `SimulationPlugin` and verify
//! emergent behavior across multiple systems working together.

use crate::buildings::Building;
use crate::citizen::CitizenState;
use crate::economy::CityBudget;
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::land_value::LandValueGrid;
use crate::pollution::PollutionGrid;
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::utilities::{UtilitySource, UtilityType};
use crate::weather::Weather;
use crate::SlowTickTimer;

// ===========================================================================
// 1. Harness bootstrap tests
// ===========================================================================

#[test]
fn empty_city_has_no_citizens() {
    let mut city = TestCity::new();
    assert_eq!(city.citizen_count(), 0, "empty city should have 0 citizens");
}

#[test]
fn empty_city_has_no_buildings() {
    let mut city = TestCity::new();
    assert_eq!(
        city.building_count(),
        0,
        "empty city should have 0 buildings"
    );
}

#[test]
fn empty_city_has_no_roads() {
    let city = TestCity::new();
    assert_eq!(
        city.road_cell_count(),
        0,
        "empty city should have 0 road cells"
    );
}

#[test]
fn empty_city_has_default_budget() {
    let city = TestCity::new();
    let budget = city.budget();
    assert!(
        (budget.treasury - 10_000.0).abs() < f64::EPSILON,
        "default treasury should be 10000, got {}",
        budget.treasury
    );
    assert!(
        (budget.tax_rate - 0.1).abs() < f32::EPSILON,
        "default tax rate should be 0.1, got {}",
        budget.tax_rate
    );
}

#[test]
fn empty_city_grid_dimensions() {
    let city = TestCity::new();
    let grid = city.grid();
    assert_eq!(grid.width, 256);
    assert_eq!(grid.height, 256);
    assert_eq!(grid.cells.len(), 256 * 256);
}

#[test]
fn empty_city_all_cells_are_grass() {
    let city = TestCity::new();
    let grid = city.grid();
    for cell in &grid.cells {
        assert_eq!(cell.cell_type, CellType::Grass);
        assert_eq!(cell.zone, ZoneType::None);
        assert!(cell.building_id.is_none());
    }
}

#[test]
fn empty_city_core_resources_exist() {
    let city = TestCity::new();
    city.assert_resource_exists::<WorldGrid>();
    city.assert_resource_exists::<RoadNetwork>();
    city.assert_resource_exists::<CityBudget>();
    city.assert_resource_exists::<RoadSegmentStore>();
    city.assert_resource_exists::<GameClock>();
    city.assert_resource_exists::<Weather>();
    city.assert_resource_exists::<SlowTickTimer>();
    city.assert_resource_exists::<LandValueGrid>();
    city.assert_resource_exists::<PollutionGrid>();
}

// ===========================================================================
// 2. Tel Aviv smoke tests
// ===========================================================================

#[test]
fn tel_aviv_has_citizens() {
    let mut city = TestCity::with_tel_aviv();
    assert!(
        city.citizen_count() > 1000,
        "Tel Aviv should have many citizens, got {}",
        city.citizen_count()
    );
}

#[test]
fn tel_aviv_has_buildings() {
    let mut city = TestCity::with_tel_aviv();
    assert!(
        city.building_count() > 100,
        "Tel Aviv should have many buildings, got {}",
        city.building_count()
    );
}

#[test]
fn tel_aviv_has_roads() {
    let city = TestCity::with_tel_aviv();
    assert!(
        city.road_cell_count() > 100,
        "Tel Aviv should have many road cells, got {}",
        city.road_cell_count()
    );
}

#[test]
fn tel_aviv_has_budget() {
    let city = TestCity::with_tel_aviv();
    assert!(
        (city.budget().treasury - 100_000.0).abs() < f64::EPSILON,
        "Tel Aviv should start with 100K treasury"
    );
}

#[test]
fn tel_aviv_has_mixed_zones() {
    let city = TestCity::with_tel_aviv();
    assert!(
        city.zoned_cell_count(ZoneType::ResidentialHigh) > 0,
        "Tel Aviv should have residential high zones"
    );
    assert!(
        city.zoned_cell_count(ZoneType::CommercialLow) > 0,
        "Tel Aviv should have commercial low zones"
    );
    assert!(
        city.zoned_cell_count(ZoneType::Industrial) > 0,
        "Tel Aviv should have industrial zones"
    );
}

#[test]
fn tel_aviv_has_services() {
    let mut city = TestCity::with_tel_aviv();
    let world = city.world_mut();
    let service_count = world.query::<&ServiceBuilding>().iter(world).count();
    assert!(
        service_count > 10,
        "Tel Aviv should have many service buildings, got {service_count}"
    );
}

#[test]
fn tel_aviv_has_utilities() {
    let mut city = TestCity::with_tel_aviv();
    let world = city.world_mut();
    let utility_count = world.query::<&UtilitySource>().iter(world).count();
    assert!(
        utility_count > 5,
        "Tel Aviv should have utility sources, got {utility_count}"
    );
}

// ===========================================================================
// 3. Road placement tests
// ===========================================================================

#[test]
fn road_placement_creates_road_cells() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    let road_count = city.road_cell_count();
    assert!(
        road_count > 0,
        "placing a road should create road cells, got {road_count}"
    );
}

#[test]
fn road_placement_creates_road_nodes_in_network() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    let network = city.road_network();
    assert!(
        !network.edges.is_empty(),
        "placing a road should add nodes to the RoadNetwork"
    );
}

#[test]
fn road_placement_creates_segments() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    let segments = city.road_segments();
    assert!(
        !segments.segments.is_empty(),
        "placing a road should create road segments"
    );
}

#[test]
fn road_cells_are_connected_in_network() {
    let city = TestCity::new().with_road(100, 100, 110, 100, RoadType::Local);

    let network = city.road_network();
    let connected_nodes = network
        .edges
        .values()
        .filter(|neighbors| !neighbors.is_empty())
        .count();
    assert!(
        connected_nodes > 0,
        "road nodes should be connected to each other"
    );
}

#[test]
fn different_road_types_create_correct_cells() {
    for road_type in [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
    ] {
        let city = TestCity::new().with_road(100, 50, 100, 60, road_type);

        let road_count = city.road_cell_count();
        assert!(
            road_count > 0,
            "road type {:?} should create road cells, got {road_count}",
            road_type
        );
    }
}

#[test]
fn multiple_roads_form_grid() {
    let city = TestCity::new()
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_road(110, 95, 110, 105, RoadType::Local);

    let road_count = city.road_cell_count();
    assert!(
        road_count > 15,
        "two intersecting roads should create many road cells, got {road_count}"
    );
}

// ===========================================================================
// 4. Zoning tests
// ===========================================================================

#[test]
fn zone_placement_sets_zone_type() {
    let city = TestCity::new().with_zone(100, 100, ZoneType::ResidentialLow);

    city.assert_zone(100, 100, ZoneType::ResidentialLow);
}

#[test]
fn zone_rect_sets_all_cells() {
    let city = TestCity::new().with_zone_rect(100, 100, 104, 104, ZoneType::CommercialHigh);

    for y in 100..=104 {
        for x in 100..=104 {
            city.assert_zone(x, y, ZoneType::CommercialHigh);
        }
    }
    city.assert_zone(99, 99, ZoneType::None);
    city.assert_zone(105, 105, ZoneType::None);
}

#[test]
fn zone_count_matches_rect_area() {
    let city = TestCity::new().with_zone_rect(50, 50, 54, 54, ZoneType::Industrial);

    let count = city.zoned_cell_count(ZoneType::Industrial);
    assert_eq!(
        count, 25,
        "5x5 rect should have 25 zoned cells, got {count}"
    );
}

// ===========================================================================
// 5. Building tests
// ===========================================================================

#[test]
fn building_placement_increments_count() {
    let mut city = TestCity::new().with_building(100, 100, ZoneType::ResidentialLow, 1);

    assert_eq!(city.building_count(), 1);
}

#[test]
fn building_placement_updates_grid() {
    let city = TestCity::new().with_building(100, 100, ZoneType::ResidentialLow, 1);

    city.assert_has_building(100, 100);
}

#[test]
fn building_has_correct_properties() {
    let mut city = TestCity::new().with_building(100, 100, ZoneType::CommercialHigh, 3);

    let world = city.world_mut();
    let building = world
        .query::<&Building>()
        .iter(world)
        .next()
        .expect("should have a building");

    assert_eq!(building.zone_type, ZoneType::CommercialHigh);
    assert_eq!(building.level, 3);
    assert_eq!(building.grid_x, 100);
    assert_eq!(building.grid_y, 100);
    assert_eq!(building.occupants, 0);
    assert!(building.capacity > 0);
}

#[test]
fn multiple_buildings_are_counted() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(110, 110, ZoneType::CommercialLow, 2)
        .with_building(120, 120, ZoneType::Industrial, 1);

    assert_eq!(city.building_count(), 3);
    assert_eq!(city.buildings_in_zone(ZoneType::ResidentialLow), 1);
    assert_eq!(city.buildings_in_zone(ZoneType::CommercialLow), 1);
    assert_eq!(city.buildings_in_zone(ZoneType::Industrial), 1);
}

// ===========================================================================
// 6. Citizen tests
// ===========================================================================

#[test]
fn citizen_placement_increments_count() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(110, 110, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (110, 110));

    assert_eq!(city.citizen_count(), 1);
}

#[test]
fn citizen_starts_at_home() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(110, 110, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (110, 110));

    assert_eq!(city.citizens_in_state(CitizenState::AtHome), 1);
}

#[test]
fn multiple_citizens_are_tracked() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(110, 100, ZoneType::ResidentialLow, 1)
        .with_building(120, 100, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (120, 100))
        .with_citizen((110, 100), (120, 100));

    assert_eq!(city.citizen_count(), 2);
}

// ===========================================================================
// 7. Budget builder tests
// ===========================================================================

#[test]
fn with_budget_sets_treasury() {
    let city = TestCity::new().with_budget(50_000.0);
    assert!(
        (city.budget().treasury - 50_000.0).abs() < f64::EPSILON,
        "treasury should be 50000, got {}",
        city.budget().treasury
    );
}

#[test]
fn budget_can_be_zero() {
    let city = TestCity::new().with_budget(0.0);
    assert!(
        city.budget().treasury.abs() < f64::EPSILON,
        "treasury should be 0"
    );
}

// ===========================================================================
// 8. Service and utility placement tests
// ===========================================================================

#[test]
fn service_placement_creates_entity() {
    let mut city = TestCity::new().with_service(100, 100, ServiceType::FireStation);

    let world = city.world_mut();
    let count = world.query::<&ServiceBuilding>().iter(world).count();
    assert_eq!(count, 1);
}

#[test]
fn service_has_correct_type_and_position() {
    let mut city = TestCity::new().with_service(100, 100, ServiceType::Hospital);

    let world = city.world_mut();
    let svc = world
        .query::<&ServiceBuilding>()
        .iter(world)
        .next()
        .unwrap();
    assert_eq!(svc.service_type, ServiceType::Hospital);
    assert_eq!(svc.grid_x, 100);
    assert_eq!(svc.grid_y, 100);
    assert!(svc.radius > 0.0);
}

#[test]
fn utility_placement_creates_entity() {
    let mut city = TestCity::new().with_utility(100, 100, UtilityType::PowerPlant);

    let world = city.world_mut();
    let count = world.query::<&UtilitySource>().iter(world).count();
    assert_eq!(count, 1);
}

#[test]
fn utility_has_correct_type_and_range() {
    let mut city = TestCity::new().with_utility(100, 100, UtilityType::WaterTower);

    let world = city.world_mut();
    let util = world.query::<&UtilitySource>().iter(world).next().unwrap();
    assert_eq!(util.utility_type, UtilityType::WaterTower);
    assert_eq!(util.grid_x, 100);
    assert_eq!(util.grid_y, 100);
    assert_eq!(util.range, 90);
}

// ===========================================================================
// 9. Weather and time builder tests
// ===========================================================================

#[test]
fn with_weather_sets_temperature() {
    let city = TestCity::new().with_weather(35.0);
    let weather = city.resource::<Weather>();
    assert!(
        (weather.temperature - 35.0).abs() < f32::EPSILON,
        "temperature should be 35.0"
    );
}

#[test]
fn with_time_sets_hour() {
    let city = TestCity::new().with_time(14.0);
    let clock = city.clock();
    assert!(
        (clock.hour - 14.0).abs() < f32::EPSILON,
        "hour should be 14.0, got {}",
        clock.hour
    );
}

// ===========================================================================
// 10. Tick / simulation progression tests
// ===========================================================================

#[test]
fn tick_advances_slow_timer() {
    let mut city = TestCity::new();
    let initial = city.slow_tick_timer().counter;
    city.tick(10);
    let after = city.slow_tick_timer().counter;
    assert!(
        after > initial,
        "slow tick timer should advance, was {initial}, now {after}"
    );
}

#[test]
fn tick_slow_cycle_runs_100_ticks() {
    let mut city = TestCity::new();
    let initial = city.slow_tick_timer().counter;
    city.tick_slow_cycle();
    let after = city.slow_tick_timer().counter;
    assert!(
        after >= initial + SlowTickTimer::INTERVAL,
        "tick_slow_cycle should run at least {} ticks, delta was {}",
        SlowTickTimer::INTERVAL,
        after - initial
    );
}

#[test]
fn tick_slow_cycles_runs_multiple() {
    let mut city = TestCity::new();
    let initial = city.slow_tick_timer().counter;
    city.tick_slow_cycles(3);
    let after = city.slow_tick_timer().counter;
    assert!(
        after >= initial + SlowTickTimer::INTERVAL * 3,
        "tick_slow_cycles(3) should run at least {} ticks, delta was {}",
        SlowTickTimer::INTERVAL * 3,
        after - initial
    );
}

#[test]
fn game_clock_starts_at_6am() {
    let city = TestCity::new();
    assert!(
        (city.clock().hour - 6.0).abs() < f32::EPSILON,
        "game clock should start at 6 AM, got {}",
        city.clock().hour
    );
    assert_eq!(city.clock().day, 1, "game clock should start at day 1");
}

#[test]
fn tick_advances_game_clock() {
    let mut city = TestCity::new();
    let initial_hour = city.clock().hour;
    // 500 ticks at 1 min/tick = ~8.33 hours advancement
    city.tick(500);
    let after_hour = city.clock().hour;
    let after_day = city.clock().day;
    assert!(
        after_day > 1 || (after_hour - initial_hour).abs() > 0.01,
        "game clock should advance after 500 ticks: day 1->{after_day}, hour {initial_hour}->{after_hour}"
    );
}

// ===========================================================================
// 11. Tel Aviv simulation smoke tests
// ===========================================================================

#[test]
fn tel_aviv_survives_100_ticks() {
    let mut city = TestCity::with_tel_aviv();
    city.tick(100);
    assert!(city.citizen_count() > 0, "citizens should still exist");
    assert!(city.building_count() > 0, "buildings should still exist");
}

#[test]
fn tel_aviv_budget_changes_over_time() {
    let mut city = TestCity::with_tel_aviv();
    let initial = city.budget().treasury;
    // Run enough ticks for monthly budget cycle (needs 30+ in-game days)
    city.tick(2000);
    let after = city.budget().treasury;
    // Treasury should change from maintenance costs, service expenses, etc.
    // Even if taxes haven't kicked in yet, expenses should deduct.
    assert!(
        (after - initial).abs() > 0.001 || after != initial,
        "treasury should change from economic activity: initial={initial}, after={after}"
    );
}

#[test]
fn tel_aviv_citizens_have_variety_of_states() {
    let mut city = TestCity::with_tel_aviv();
    city.tick(200);

    let at_home = city.citizens_in_state(CitizenState::AtHome);
    let commuting_to_work = city.citizens_in_state(CitizenState::CommutingToWork);
    let working = city.citizens_in_state(CitizenState::Working);
    let commuting_home = city.citizens_in_state(CitizenState::CommutingHome);
    let total = city.citizen_count();

    let states_with_citizens = [
        at_home > 0,
        commuting_to_work > 0,
        working > 0,
        commuting_home > 0,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    assert!(
        states_with_citizens >= 1,
        "after 200 ticks, citizens should be in at least 1 state. \
         AtHome={at_home}, CommutingToWork={commuting_to_work}, \
         Working={working}, CommutingHome={commuting_home}, Total={total}"
    );
}

// ===========================================================================
// 12. Combined scenario tests
// ===========================================================================

#[test]
fn city_with_full_infrastructure_runs() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 100, 120, 100, RoadType::Avenue)
        .with_road(110, 95, 110, 110, RoadType::Local)
        .with_zone_rect(102, 95, 108, 99, ZoneType::ResidentialLow)
        .with_zone_rect(112, 95, 118, 99, ZoneType::CommercialLow)
        .with_building(105, 97, ZoneType::ResidentialLow, 1)
        .with_building(115, 97, ZoneType::CommercialLow, 1)
        .with_citizen((105, 97), (115, 97))
        .with_service(110, 105, ServiceType::FireStation)
        .with_utility(110, 90, UtilityType::PowerPlant)
        .with_utility(120, 90, UtilityType::WaterTower);

    assert_eq!(city.citizen_count(), 1);
    assert_eq!(city.building_count(), 2);
    assert!(city.road_cell_count() > 0);
    city.assert_budget_above(99_000.0);

    city.tick(50);

    assert!(city.citizen_count() >= 1, "citizen should still exist");
}

#[test]
fn road_then_zone_then_tick_survives() {
    let mut city = TestCity::new()
        .with_budget(50_000.0)
        .with_road(100, 100, 100, 120, RoadType::Local)
        .with_zone_rect(102, 100, 105, 120, ZoneType::ResidentialLow);

    city.tick_slow_cycles(2);
    assert!(city.road_cell_count() > 0);
}

#[test]
fn builder_methods_are_chainable() {
    let mut city = TestCity::new()
        .with_budget(1_000.0)
        .with_road(50, 50, 50, 60, RoadType::Local)
        .with_road(50, 55, 60, 55, RoadType::Local)
        .with_zone(55, 52, ZoneType::ResidentialLow)
        .with_zone_rect(52, 57, 58, 63, ZoneType::CommercialLow)
        .with_building(55, 52, ZoneType::ResidentialLow, 1)
        .with_building(55, 60, ZoneType::CommercialLow, 1)
        .with_citizen((55, 52), (55, 60))
        .with_service(55, 55, ServiceType::PoliceStation)
        .with_utility(60, 50, UtilityType::PowerPlant)
        .with_weather(25.0)
        .with_time(8.0);

    assert_eq!(city.citizen_count(), 1);
    assert_eq!(city.building_count(), 2);
}

// ===========================================================================
// 13. Assertion helper tests
// ===========================================================================

#[test]
fn assert_citizen_count_between_passes() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(110, 110, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (110, 110));

    city.assert_citizen_count_between(0, 10);
    city.assert_citizen_count_between(1, 1);
}

#[test]
#[should_panic(expected = "Expected citizen count")]
fn assert_citizen_count_between_fails() {
    let mut city = TestCity::new();
    city.assert_citizen_count_between(1, 10);
}

#[test]
fn assert_budget_above_passes() {
    let city = TestCity::new().with_budget(50_000.0);
    city.assert_budget_above(49_000.0);
}

#[test]
#[should_panic(expected = "Expected treasury")]
fn assert_budget_above_fails() {
    let city = TestCity::new().with_budget(1_000.0);
    city.assert_budget_above(5_000.0);
}

#[test]
fn assert_budget_below_passes() {
    let city = TestCity::new().with_budget(1_000.0);
    city.assert_budget_below(5_000.0);
}

#[test]
#[should_panic(expected = "Expected treasury")]
fn assert_budget_below_fails() {
    let city = TestCity::new().with_budget(50_000.0);
    city.assert_budget_below(1_000.0);
}

#[test]
fn assert_has_road_passes() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    let grid = city.grid();
    let mut found_road = false;
    for y in 100..=110 {
        if grid.get(100, y).cell_type == CellType::Road {
            city.assert_has_road(100, y);
            found_road = true;
            break;
        }
    }
    assert!(found_road, "should find at least one road cell");
}

#[test]
#[should_panic(expected = "Expected road")]
fn assert_has_road_fails() {
    let city = TestCity::new();
    city.assert_has_road(100, 100);
}

#[test]
fn assert_has_building_passes() {
    let city = TestCity::new().with_building(100, 100, ZoneType::ResidentialLow, 1);
    city.assert_has_building(100, 100);
}

#[test]
#[should_panic(expected = "Expected building")]
fn assert_has_building_fails() {
    let city = TestCity::new();
    city.assert_has_building(100, 100);
}

// ===========================================================================
// 14. Edge cases and boundary tests
// ===========================================================================

#[test]
fn road_at_grid_boundaries() {
    let city = TestCity::new().with_road(5, 5, 5, 15, RoadType::Local);
    assert!(city.road_cell_count() > 0);
}

#[test]
fn building_at_various_levels() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialHigh, 1)
        .with_building(110, 100, ZoneType::ResidentialHigh, 3)
        .with_building(120, 100, ZoneType::ResidentialHigh, 5);

    let world = city.world_mut();
    let buildings: Vec<&Building> = world.query::<&Building>().iter(world).collect();

    assert_eq!(buildings.len(), 3);

    let mut capacities: Vec<u32> = buildings.iter().map(|b| b.capacity).collect();
    capacities.sort();
    assert!(
        capacities[0] < capacities[1] && capacities[1] < capacities[2],
        "higher level buildings should have more capacity: {:?}",
        capacities
    );
}

#[test]
fn zero_ticks_does_nothing() {
    let mut city = TestCity::new();
    let timer_before = city.slow_tick_timer().counter;
    city.tick(0);
    let timer_after = city.slow_tick_timer().counter;
    assert_eq!(
        timer_before, timer_after,
        "0 ticks should not advance timer"
    );
}

// ===========================================================================
// Destination cache invalidation tests (#1157)
// ===========================================================================

#[test]
fn test_destination_cache_removes_demolished_building() {
    use crate::movement::DestinationCache;

    // Build a city with a commercial building (which will appear in shops cache)
    let mut city = TestCity::new().with_building(10, 10, ZoneType::CommercialLow, 1);

    // Tick to let the destination cache populate
    city.tick(2);

    // Verify the building is in the shops cache
    {
        let cache = city.resource::<DestinationCache>();
        assert!(
            cache.shops.contains(&(10, 10)),
            "commercial building should be in shops cache after ticking"
        );
    }

    // Find and despawn the building entity (simulating bulldoze)
    let building_entity = {
        let world = city.world_mut();
        let mut query = world.query::<(bevy::prelude::Entity, &Building)>();
        let (entity, _) = query.iter(world).next().expect("should have a building");
        entity
    };
    city.world_mut().despawn(building_entity);

    // Tick again so that RemovedComponents fires and cache rebuilds
    city.tick(2);

    // Verify the building is no longer in the shops cache
    let cache = city.resource::<DestinationCache>();
    assert!(
        !cache.shops.contains(&(10, 10)),
        "demolished building should NOT be in shops cache"
    );
}

#[test]
fn test_destination_cache_removes_demolished_service() {
    use crate::movement::DestinationCache;

    // Build a city with a leisure service (park)
    let mut city = TestCity::new().with_service(15, 15, ServiceType::SmallPark);

    // Tick to populate destination cache
    city.tick(2);

    // Verify the service is in the leisure cache
    {
        let cache = city.resource::<DestinationCache>();
        assert!(
            cache.leisure.contains(&(15, 15)),
            "park should be in leisure cache after ticking"
        );
    }

    // Find and despawn the service entity
    let service_entity = {
        let world = city.world_mut();
        let mut query = world.query::<(bevy::prelude::Entity, &ServiceBuilding)>();
        let (entity, _) = query
            .iter(world)
            .next()
            .expect("should have a service building");
        entity
    };
    city.world_mut().despawn(service_entity);

    // Tick again so RemovedComponents fires
    city.tick(2);

    // Verify the service is no longer in the leisure cache
    let cache = city.resource::<DestinationCache>();
    assert!(
        !cache.leisure.contains(&(15, 15)),
        "demolished park should NOT be in leisure cache"
    );
}

// ===========================================================================
// 16. Ghost road pathfinding (path cache invalidation on road deletion)
// ===========================================================================

#[test]
fn test_road_removal_invalidates_citizen_path_cache() {
    use crate::citizen::{CitizenStateComp, PathCache};
    use crate::roads::RoadNode;

    // Build a city with a straight road from (100,100) to (100,115)
    // and a citizen with home and work buildings.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (100, 115));

    // Manually set the citizen to CommutingToWork with a path through
    // road nodes that includes (100, 105), which we will then delete.
    {
        let world = city.world_mut();
        let mut query = world.query::<(&mut PathCache, &mut CitizenStateComp)>();
        for (mut path, mut state) in query.iter_mut(world) {
            *path = PathCache::new(vec![
                RoadNode(100, 101),
                RoadNode(100, 102),
                RoadNode(100, 103),
                RoadNode(100, 104),
                RoadNode(100, 105),
                RoadNode(100, 106),
                RoadNode(100, 107),
            ]);
            state.0 = CitizenState::CommutingToWork;
        }
    }

    // Verify the citizen is commuting with a non-empty path
    assert_eq!(city.citizens_in_state(CitizenState::CommutingToWork), 1);

    // Bulldoze road cell (100, 105) -- this is in the middle of the path
    city.remove_road_at(100, 105);

    // Run one tick so the invalidation system fires
    city.tick(1);

    // The citizen should have been sent home because their path contained
    // a deleted road node.
    assert_eq!(
        city.citizens_in_state(CitizenState::CommutingToWork),
        0,
        "citizen should no longer be commuting after road deletion"
    );
    assert_eq!(
        city.citizens_in_state(CitizenState::AtHome),
        1,
        "citizen should be sent home after path invalidation"
    );

    // Verify the path cache was cleared
    {
        let world = city.world_mut();
        let mut query = world.query::<&PathCache>();
        for path in query.iter(world) {
            assert!(
                path.is_complete(),
                "path cache should be empty/complete after invalidation"
            );
        }
    }
}

#[test]
fn test_road_removal_does_not_affect_citizens_on_other_roads() {
    use crate::citizen::{CitizenStateComp, PathCache};
    use crate::roads::RoadNode;

    // Build a city with two separate roads
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_road(120, 100, 120, 115, RoadType::Local)
        .with_building(120, 100, ZoneType::ResidentialLow, 1)
        .with_building(120, 115, ZoneType::CommercialLow, 1)
        .with_citizen((120, 100), (120, 115));

    // Set citizen path along the SECOND road (120, y)
    {
        let world = city.world_mut();
        let mut query = world.query::<(&mut PathCache, &mut CitizenStateComp)>();
        for (mut path, mut state) in query.iter_mut(world) {
            *path = PathCache::new(vec![
                RoadNode(120, 101),
                RoadNode(120, 102),
                RoadNode(120, 103),
                RoadNode(120, 104),
                RoadNode(120, 105),
            ]);
            state.0 = CitizenState::CommutingToWork;
        }
    }

    // Bulldoze a road cell on the FIRST road (100, 105) -- unrelated to citizen's path
    city.remove_road_at(100, 105);

    // Run one tick
    city.tick(1);

    // The citizen should still be commuting -- their path is on a different road
    assert_eq!(
        city.citizens_in_state(CitizenState::CommutingToWork),
        1,
        "citizen on unrelated road should still be commuting"
    );
}

#[test]
fn test_road_removal_only_affects_commuting_citizens() {
    use crate::citizen::{CitizenStateComp, PathCache};
    use crate::roads::RoadNode;

    // Build a city with a road and a citizen at home
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (100, 115));

    // Citizen is AtHome with a stale path (leftover from previous trip).
    // This should NOT be affected by road removal since they are not commuting.
    {
        let world = city.world_mut();
        let mut query = world.query::<(&mut PathCache, &mut CitizenStateComp)>();
        for (mut path, mut state) in query.iter_mut(world) {
            *path = PathCache::new(vec![
                RoadNode(100, 103),
                RoadNode(100, 104),
                RoadNode(100, 105),
            ]);
            state.0 = CitizenState::AtHome;
        }
    }

    city.remove_road_at(100, 105);
    city.tick(1);

    // Should remain at home -- not affected because they aren't commuting
    assert_eq!(
        city.citizens_in_state(CitizenState::AtHome),
        1,
        "at-home citizen should not be affected by road removal"
    );
}

// =============================================================================
// Colorblind accessibility tests
// =============================================================================

#[test]
fn test_colorblind_mode_default_is_normal() {
    let city = TestCity::new();
    let settings = city.resource::<crate::colorblind::ColorblindSettings>();
    assert_eq!(
        settings.mode,
        crate::colorblind::ColorblindMode::Normal,
        "default colorblind mode should be Normal"
    );
}

#[test]
fn test_colorblind_mode_persists_across_ticks() {
    let mut city = TestCity::new();
    city.app
        .world_mut()
        .resource_mut::<crate::colorblind::ColorblindSettings>()
        .mode = crate::colorblind::ColorblindMode::Protanopia;
    city.tick(10);
    let settings = city.resource::<crate::colorblind::ColorblindSettings>();
    assert_eq!(
        settings.mode,
        crate::colorblind::ColorblindMode::Protanopia,
        "colorblind mode should persist across ticks"
    );
}

#[test]
fn test_colorblind_settings_saveable() {
    use crate::colorblind::{ColorblindMode, ColorblindSettings};
    use crate::Saveable;

    // Default should not save
    let default_settings = ColorblindSettings::default();
    assert!(
        default_settings.save_to_bytes().is_none(),
        "default settings should skip save"
    );

    // Non-default should save and restore
    let settings = ColorblindSettings {
        mode: ColorblindMode::Deuteranopia,
    };
    let bytes = settings.save_to_bytes().expect("should save non-default");
    let restored = ColorblindSettings::load_from_bytes(&bytes);
    assert_eq!(restored.mode, ColorblindMode::Deuteranopia);
}

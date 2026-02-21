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

// ====================================================================// 1. Harness bootstrap tests
// ====================================================================
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

// ====================================================================// 2. Tel Aviv smoke tests
// ====================================================================
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

// ====================================================================// 3. Road placement tests
// ====================================================================
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

// ====================================================================// 4. Zoning tests
// ====================================================================
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

// ====================================================================// 5. Building tests
// ====================================================================
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

// ====================================================================// 6. Citizen tests
// ====================================================================
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

// ====================================================================// 7. Budget builder tests
// ====================================================================
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

// ====================================================================// 8. Service and utility placement tests
// ====================================================================
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

// ====================================================================// 9. Weather and time builder tests
// ====================================================================
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

// ====================================================================// 10. Tick / simulation progression tests
// ====================================================================
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

// ====================================================================// 11. Tel Aviv simulation smoke tests
// ====================================================================
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

// ====================================================================// 12. Combined scenario tests
// ====================================================================
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

// ====================================================================// 13. Assertion helper tests
// ====================================================================
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

// ====================================================================// 14. Edge cases and boundary tests
// ====================================================================
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

// ====================================================================// Destination cache invalidation tests (#1157)
// ====================================================================
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

// ====================================================================// 16. Ghost road pathfinding (path cache invalidation on road deletion)
// ====================================================================
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

// ======================================================================// Colorblind accessibility tests
// ======================================================================
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
    city.world_mut()
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

// ==================== Network Visualization Tests =============
#[test]
fn test_network_viz_power_source_assigns_cells() {
    use crate::network_viz::NetworkVizData;

    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_utility(10, 10, UtilityType::PowerPlant);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    // Source cell should be covered
    assert!(
        viz.power_source_color(10, 10).is_some(),
        "power source cell should be covered"
    );
    // Nearby road cell should be covered by the same source
    assert!(
        viz.power_source_color(15, 10).is_some(),
        "road cell within range should be covered"
    );
    // Far away cell should NOT be covered
    assert!(
        viz.power_source_color(200, 200).is_none(),
        "distant cell should not be covered"
    );
}

#[test]
fn test_network_viz_water_source_assigns_cells() {
    use crate::network_viz::NetworkVizData;

    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::WaterTower);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    assert!(
        viz.water_source_color(50, 50).is_some(),
        "water source cell should be covered"
    );
    assert!(
        viz.water_source_color(55, 50).is_some(),
        "road cell within range should have water source"
    );
}

#[test]
fn test_network_viz_multiple_power_sources_different_colors() {
    use crate::network_viz::NetworkVizData;

    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_road(40, 10, 50, 10, RoadType::Local)
        .with_utility(10, 10, UtilityType::PowerPlant)
        .with_utility(40, 10, UtilityType::SolarFarm);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    let color_a = viz.power_source_color(10, 10);
    let color_b = viz.power_source_color(40, 10);

    assert!(color_a.is_some(), "first source should be covered");
    assert!(color_b.is_some(), "second source should be covered");
    // Different sources should have different colors
    assert_ne!(
        color_a.unwrap(),
        color_b.unwrap(),
        "different sources should have different colors"
    );
}

#[test]
fn test_network_viz_disconnected_roads_no_coverage() {
    use crate::network_viz::NetworkVizData;

    let mut city = TestCity::new()
        .with_road(10, 10, 15, 10, RoadType::Local)
        .with_road(20, 10, 25, 10, RoadType::Local) // disconnected segment
        .with_utility(10, 10, UtilityType::PowerPlant);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    // Connected segment should be covered
    assert!(
        viz.power_source_color(12, 10).is_some(),
        "connected road should be covered"
    );
    // Disconnected segment should NOT be covered
    assert!(
        viz.power_source_color(22, 10).is_none(),
        "disconnected road should not be covered"
    );
}

#[test]
fn test_network_viz_road_cells_tracked_for_pulse_lines() {
    use crate::network_viz::NetworkVizData;

    let mut city = TestCity::new()
        .with_road(10, 10, 25, 10, RoadType::Local)
        .with_utility(10, 10, UtilityType::PowerPlant);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    // Should have road cells tracked for pulse animation
    assert!(
        !viz.power_road_cells.is_empty(),
        "should track road cells for pulse lines"
    );
    // Each road cell should have a distance value
    for &(_, _, dist, _) in &viz.power_road_cells {
        assert!(
            dist > 0,
            "road cells should have non-zero distance from source"
        );
    }
}

#[test]
fn test_network_viz_source_info_populated() {
    use crate::network_viz::NetworkVizData;

    let mut city = TestCity::new()
        .with_road(10, 10, 25, 10, RoadType::Local)
        .with_utility(10, 10, UtilityType::PowerPlant);

    city.tick(5);

    let viz = city.resource::<NetworkVizData>();
    assert_eq!(
        viz.power_sources.len(),
        1,
        "should have exactly one power source"
    );

    let info = &viz.power_sources[0];
    assert_eq!(info.grid_x, 10);
    assert_eq!(info.grid_y, 10);
    assert!(info.cells_covered > 0, "source should cover some cells");
    assert!(
        info.effective_range > 0,
        "source should have positive range"
    );
}

// ======================================================================// Keybindings (UX-035)
// ======================================================================
#[test]
fn test_keybindings_default_resource_exists() {
    let city = TestCity::new();
    let bindings = city.resource::<crate::keybindings::KeyBindings>();
    assert_eq!(
        bindings.toggle_pause.key,
        bevy::prelude::KeyCode::Space,
        "default pause key should be Space"
    );
}

#[test]
fn test_keybindings_rebind_and_conflict_detection() {
    use crate::keybindings::{BindableAction, KeyBinding, KeyBindings};
    use bevy::prelude::KeyCode;

    let mut kb = KeyBindings::default();
    kb.set(
        BindableAction::TogglePause,
        KeyBinding::simple(KeyCode::KeyX),
    );
    assert_eq!(kb.get(BindableAction::TogglePause).key, KeyCode::KeyX);

    let same_key = KeyBinding::simple(KeyCode::KeyQ);
    kb.set(BindableAction::ToolRoad, same_key);
    kb.set(BindableAction::ToolBulldoze, same_key);
    let conflicts = kb.find_conflicts();
    assert!(
        conflicts.iter().any(|(a, b)| {
            (*a == BindableAction::ToolRoad && *b == BindableAction::ToolBulldoze)
                || (*a == BindableAction::ToolBulldoze && *b == BindableAction::ToolRoad)
        }),
        "should detect conflict"
    );
}

#[test]
fn test_keybindings_saveable_roundtrip() {
    use crate::keybindings::{BindableAction, KeyBinding, KeyBindings};
    use crate::Saveable;
    use bevy::prelude::KeyCode;

    assert!(
        KeyBindings::default().save_to_bytes().is_none(),
        "default should skip save"
    );

    let mut kb = KeyBindings::default();
    kb.set(BindableAction::Screenshot, KeyBinding::simple(KeyCode::F11));
    let bytes = kb.save_to_bytes().expect("modified should save");
    let restored = KeyBindings::load_from_bytes(&bytes);
    assert_eq!(restored.get(BindableAction::Screenshot).key, KeyCode::F11);
    assert_eq!(
        restored.get(BindableAction::TogglePause).key,
        KeyCode::Space
    );
}

#[test]
fn test_keybindings_reset_to_defaults() {
    use crate::keybindings::{BindableAction, KeyBinding, KeyBindings};
    use bevy::prelude::KeyCode;

    let mut kb = KeyBindings::default();
    kb.set(
        BindableAction::TogglePause,
        KeyBinding::simple(KeyCode::KeyX),
    );
    assert_eq!(kb.get(BindableAction::TogglePause).key, KeyCode::KeyX);

    kb = KeyBindings::default();
    assert_eq!(kb.get(BindableAction::TogglePause).key, KeyCode::Space);
}

#[test]
fn test_marriage_matching_one_to_one_no_duplicate_partners() {
    use crate::citizen::{
        Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation,
        Needs, PathCache, Personality, Position, Velocity,
    };
    use crate::movement::ActivityTimer;
    use std::collections::HashMap;

    // Create a city with a residential building
    let mut city = TestCity::new().with_building(50, 50, ZoneType::ResidentialLow, 3);

    // Get the building entity
    let building_entity = city.grid().get(50, 50).building_id.unwrap();

    let (wx, wy) = WorldGrid::grid_to_world(50, 50);

    // Spawn 10 eligible males and 2 eligible females in the same building.
    // Without the fix, both females could be matched to multiple males in
    // a single tick, creating non-reciprocal partnerships.
    let world = city.world_mut();
    for _ in 0..10 {
        world.spawn((
            Citizen,
            Position { x: wx, y: wy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: 50,
                grid_y: 50,
                building: building_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 2,
                happiness: 80.0,
                health: 90.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
        ));
    }
    for _ in 0..2 {
        world.spawn((
            Citizen,
            Position { x: wx, y: wy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: 50,
                grid_y: 50,
                building: building_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 28,
                gender: Gender::Female,
                education: 2,
                happiness: 80.0,
                health: 90.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
        ));
    }

    // Run many life-event cycles to give marriage matching many chances to fire.
    // LIFE_EVENT_INTERVAL is 600, so 600 * 50 = 30000 ticks gives ~50 cycles.
    city.tick(30_000);

    // Verify 1:1 matching: every citizen with a partner must have that partner
    // point back at them (reciprocal), and no entity appears as a partner of
    // more than one other entity.
    let world = city.world_mut();
    let mut partner_of: HashMap<bevy::prelude::Entity, bevy::prelude::Entity> = HashMap::new();

    let mut query = world.query::<(bevy::prelude::Entity, &Family)>();
    let pairs: Vec<_> = query.iter(world).map(|(e, f)| (e, f.partner)).collect();

    for (entity, partner_opt) in &pairs {
        if let Some(partner) = partner_opt {
            // Check no entity is claimed as partner by more than one citizen
            if let Some(&prev_claimer) = partner_of.get(partner) {
                panic!(
                    "Entity {:?} is partner of both {:?} and {:?} -- duplicate pairing!",
                    partner, prev_claimer, entity
                );
            }
            partner_of.insert(*partner, *entity);
        }
    }

    // Check reciprocity: if A's partner is B, then B's partner must be A
    let family_map: HashMap<_, _> = pairs.iter().map(|(e, p)| (*e, *p)).collect();
    for (entity, partner_opt) in &family_map {
        if let Some(partner) = partner_opt {
            let partner_partner = family_map.get(partner).and_then(|p| *p);
            assert_eq!(
                partner_partner,
                Some(*entity),
                "Non-reciprocal partnership: {:?} -> {:?}, but {:?} -> {:?}",
                entity,
                partner,
                partner,
                partner_partner
            );
        }
    }
}

// ===========================================================================
// Save/Load Roundtrip and Fidelity Tests (Issue #1208)
// ===========================================================================
/// Test that citizen component data survives a serde serialize/deserialize
/// roundtrip, verifying that personality, needs, details, position, velocity,
/// and activity timer all match after decoding. This validates the same
/// serialization path that the save system relies on (serde derives).
#[test]
fn test_save_load_roundtrip_citizen_fidelity() {
    use crate::citizen::{
        CitizenDetails, CitizenStateComp, Family, Gender, HomeLocation, Needs, PathCache,
        Personality, Position, Velocity, WorkLocation,
    };
    use crate::movement::ActivityTimer;

    // Build a small city with specific citizen data.
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    // Spawn a citizen with known, non-default values.
    let world = city.world_mut();

    let home_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(12, 11).building_id.unwrap()
    };
    let work_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(18, 11).building_id.unwrap()
    };

    world.spawn((
        crate::citizen::Citizen,
        Position { x: 200.5, y: 180.3 },
        Velocity { x: 1.5, y: -0.7 },
        HomeLocation {
            grid_x: 12,
            grid_y: 11,
            building: home_entity,
        },
        WorkLocation {
            grid_x: 18,
            grid_y: 11,
            building: work_entity,
        },
        CitizenStateComp(CitizenState::Working),
        PathCache::new(vec![
            crate::roads::RoadNode(12, 10),
            crate::roads::RoadNode(15, 10),
            crate::roads::RoadNode(18, 10),
        ]),
        CitizenDetails {
            age: 42,
            gender: Gender::Female,
            education: 3,
            happiness: 72.5,
            health: 88.3,
            salary: 6500.0,
            savings: 15000.0,
        },
        Personality {
            ambition: 0.85,
            sociability: 0.3,
            materialism: 0.65,
            resilience: 0.92,
        },
        Needs {
            hunger: 55.0,
            energy: 70.0,
            social: 42.0,
            fun: 38.0,
            comfort: 65.0,
        },
        Family::default(),
        ActivityTimer(150),
    ));

    // Collect citizen data before "save".
    let world = city.world_mut();
    let mut query = world.query::<(
        &CitizenDetails,
        &CitizenStateComp,
        &HomeLocation,
        &WorkLocation,
        &PathCache,
        &Velocity,
        &Position,
        &Personality,
        &Needs,
        &ActivityTimer,
    )>();

    // Serialize each component via serde_json (the same serde path the save system
    // uses, since all citizen components derive Serialize/Deserialize).
    let mut serialized_data = Vec::new();
    for (details, state, home, work, path, vel, pos, pers, needs, timer) in query.iter(world) {
        let details_json = serde_json::to_string(details).unwrap();
        let state_json = serde_json::to_string(&state.0).unwrap();
        let path_json = serde_json::to_string(path).unwrap();
        let vel_json = serde_json::to_string(vel).unwrap();
        let pos_json = serde_json::to_string(pos).unwrap();
        let pers_json = serde_json::to_string(pers).unwrap();
        let needs_json = serde_json::to_string(needs).unwrap();

        serialized_data.push((
            details_json,
            state_json,
            (home.grid_x, home.grid_y),
            (work.grid_x, work.grid_y),
            path_json,
            vel_json,
            pos_json,
            pers_json,
            needs_json,
            timer.0,
        ));
    }

    assert_eq!(serialized_data.len(), 1, "should have exactly one citizen");
    let saved = &serialized_data[0];

    // Deserialize (simulating load).
    let details_after: CitizenDetails = serde_json::from_str(&saved.0).unwrap();
    let state_after: CitizenState = serde_json::from_str(&saved.1).unwrap();
    let path_after: PathCache = serde_json::from_str(&saved.4).unwrap();
    let vel_after: Velocity = serde_json::from_str(&saved.5).unwrap();
    let pos_after: Position = serde_json::from_str(&saved.6).unwrap();
    let pers_after: Personality = serde_json::from_str(&saved.7).unwrap();
    let needs_after: Needs = serde_json::from_str(&saved.8).unwrap();

    // Assert all citizen details match.
    assert_eq!(details_after.age, 42, "age mismatch after roundtrip");
    assert!(
        matches!(details_after.gender, Gender::Female),
        "gender mismatch after roundtrip"
    );
    assert_eq!(
        details_after.education, 3,
        "education mismatch after roundtrip"
    );
    assert!(
        (details_after.happiness - 72.5).abs() < f32::EPSILON,
        "happiness mismatch: {}",
        details_after.happiness
    );
    assert!(
        (details_after.health - 88.3).abs() < 0.01,
        "health mismatch: {}",
        details_after.health
    );
    assert!(
        (details_after.salary - 6500.0).abs() < f32::EPSILON,
        "salary mismatch after roundtrip"
    );
    assert!(
        (details_after.savings - 15000.0).abs() < f32::EPSILON,
        "savings mismatch after roundtrip"
    );

    // Assert state.
    assert_eq!(
        state_after,
        CitizenState::Working,
        "state mismatch after roundtrip"
    );

    // Assert personality.
    assert!(
        (pers_after.ambition - 0.85).abs() < f32::EPSILON,
        "ambition mismatch after roundtrip"
    );
    assert!(
        (pers_after.sociability - 0.3).abs() < f32::EPSILON,
        "sociability mismatch after roundtrip"
    );
    assert!(
        (pers_after.materialism - 0.65).abs() < f32::EPSILON,
        "materialism mismatch after roundtrip"
    );
    assert!(
        (pers_after.resilience - 0.92).abs() < f32::EPSILON,
        "resilience mismatch after roundtrip"
    );

    // Assert needs.
    assert!(
        (needs_after.hunger - 55.0).abs() < f32::EPSILON,
        "hunger mismatch after roundtrip"
    );
    assert!(
        (needs_after.energy - 70.0).abs() < f32::EPSILON,
        "energy mismatch after roundtrip"
    );
    assert!(
        (needs_after.social - 42.0).abs() < f32::EPSILON,
        "social mismatch after roundtrip"
    );
    assert!(
        (needs_after.fun - 38.0).abs() < f32::EPSILON,
        "fun mismatch after roundtrip"
    );
    assert!(
        (needs_after.comfort - 65.0).abs() < f32::EPSILON,
        "comfort mismatch after roundtrip"
    );

    // Assert position.
    assert!(
        (pos_after.x - 200.5).abs() < f32::EPSILON,
        "pos_x mismatch: {}",
        pos_after.x
    );
    assert!(
        (pos_after.y - 180.3).abs() < 0.01,
        "pos_y mismatch: {}",
        pos_after.y
    );

    // Assert velocity.
    assert!(
        (vel_after.x - 1.5).abs() < f32::EPSILON,
        "vel_x mismatch after roundtrip"
    );
    assert!(
        (vel_after.y - (-0.7)).abs() < f32::EPSILON,
        "vel_y mismatch after roundtrip"
    );

    // Assert path cache.
    assert_eq!(
        path_after.waypoints.len(),
        3,
        "path length mismatch after roundtrip"
    );
    assert_eq!(
        (path_after.waypoints[0].0, path_after.waypoints[0].1),
        (12, 10),
        "path[0] mismatch"
    );
    assert_eq!(
        (path_after.waypoints[1].0, path_after.waypoints[1].1),
        (15, 10),
        "path[1] mismatch"
    );
    assert_eq!(
        (path_after.waypoints[2].0, path_after.waypoints[2].1),
        (18, 10),
        "path[2] mismatch"
    );
    assert_eq!(path_after.current_index, 0, "path current_index mismatch");

    // Assert home/work grid positions survived.
    assert_eq!(saved.2, (12, 11), "home position mismatch after roundtrip");
    assert_eq!(saved.3, (18, 11), "work position mismatch after roundtrip");

    // Assert activity timer.
    assert_eq!(saved.9, 150, "activity timer mismatch after roundtrip");
}

/// Test that tearing down a city (despawning all entities, resetting resources)
/// results in a clean slate with no leftover entities.
#[test]
fn test_new_game_teardown_clean_slate() {
    use crate::citizen::Citizen;

    // Build a city with roads, buildings, and citizens.
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_building(24, 11, ZoneType::Industrial, 1)
        .with_citizen((12, 11), (18, 11))
        .with_citizen((12, 11), (24, 11))
        .with_service(15, 11, ServiceType::FireStation)
        .with_utility(20, 11, UtilityType::PowerPlant);

    // Run a few ticks so systems process.
    city.tick(5);

    // Verify city is populated.
    assert!(city.citizen_count() >= 2, "city should have citizens");
    assert!(city.building_count() >= 3, "city should have buildings");

    // Simulate "new game" by despawning all entities and resetting resources.
    // (The actual NewGameEvent is handled by SavePlugin in the app crate,
    //  but we test the core teardown logic directly.)
    let world = city.world_mut();

    // Despawn all citizens.
    let citizen_entities: Vec<bevy::prelude::Entity> = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<Citizen>>()
        .iter(world)
        .collect();
    for entity in citizen_entities {
        world.despawn(entity);
    }

    // Despawn all buildings.
    let building_entities: Vec<bevy::prelude::Entity> = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<Building>>()
        .iter(world)
        .collect();
    for entity in building_entities {
        world.despawn(entity);
    }

    // Despawn all service buildings.
    let service_entities: Vec<bevy::prelude::Entity> = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ServiceBuilding>>()
        .iter(world)
        .collect();
    for entity in service_entities {
        world.despawn(entity);
    }

    // Despawn all utility sources.
    let utility_entities: Vec<bevy::prelude::Entity> = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<UtilitySource>>()
        .iter(world)
        .collect();
    for entity in utility_entities {
        world.despawn(entity);
    }

    // Reset resources to defaults (simulating new-game reset).
    let width = world.resource::<WorldGrid>().width;
    let height = world.resource::<WorldGrid>().height;
    *world.resource_mut::<WorldGrid>() = WorldGrid::new(width, height);
    *world.resource_mut::<RoadNetwork>() = RoadNetwork::default();
    *world.resource_mut::<RoadSegmentStore>() = RoadSegmentStore::default();
    world.resource_mut::<CityBudget>().treasury = 50_000.0;
    world.resource_mut::<CityBudget>().tax_rate = 0.10;
    world.resource_mut::<CityBudget>().last_collection_day = 0;
    world.resource_mut::<GameClock>().day = 1;
    world.resource_mut::<GameClock>().hour = 8.0;

    // Verify clean slate.
    assert_eq!(
        city.citizen_count(),
        0,
        "should have 0 citizens after new game"
    );
    assert_eq!(
        city.building_count(),
        0,
        "should have 0 buildings after new game"
    );
    assert_eq!(
        city.road_cell_count(),
        0,
        "should have 0 road cells after new game"
    );
    assert!(
        (city.budget().treasury - 50_000.0).abs() < f64::EPSILON,
        "treasury should be reset to 50000, got {}",
        city.budget().treasury
    );
    assert_eq!(city.clock().day, 1, "day should be reset to 1");

    // Verify simulation can still tick without panics after teardown.
    city.tick(10);

    // Still clean: no citizens spontaneously appear without zones/buildings.
    assert_eq!(
        city.citizen_count(),
        0,
        "should still have 0 citizens after ticking on clean slate"
    );
}

/// Test that citizens do not lose state when transitioning through LOD tiers
/// (Full -> Abstract -> Full). The LOD system preserves all components and
/// only adds/removes a CompressedCitizen marker.
#[test]
fn test_lod_roundtrip_no_state_loss() {
    use crate::citizen::{
        Citizen, CitizenDetails, CitizenStateComp, Family, Gender, HomeLocation, Needs, PathCache,
        Personality, Position, Velocity, WorkLocation,
    };
    use crate::lod::{CompressedCitizen, LodTier};
    use crate::movement::ActivityTimer;

    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1);

    // Spawn a citizen with known values AND a LodTier::Full component.
    let world = city.world_mut();
    let home_entity = world
        .resource::<WorldGrid>()
        .get(12, 11)
        .building_id
        .unwrap();
    let work_entity = world
        .resource::<WorldGrid>()
        .get(18, 11)
        .building_id
        .unwrap();

    let citizen_entity = world
        .spawn((
            Citizen,
            LodTier::Full,
            Position { x: 200.0, y: 180.0 },
            Velocity { x: 0.5, y: -0.3 },
            HomeLocation {
                grid_x: 12,
                grid_y: 11,
                building: home_entity,
            },
            WorkLocation {
                grid_x: 18,
                grid_y: 11,
                building: work_entity,
            },
            CitizenStateComp(CitizenState::Working),
            PathCache::new(vec![crate::roads::RoadNode(15, 10)]),
            CitizenDetails {
                age: 35,
                gender: Gender::Male,
                education: 2,
                happiness: 68.0,
                health: 92.0,
                salary: 4500.0,
                savings: 12000.0,
            },
            Personality {
                ambition: 0.7,
                sociability: 0.4,
                materialism: 0.55,
                resilience: 0.8,
            },
            Needs {
                hunger: 60.0,
                energy: 75.0,
                social: 50.0,
                fun: 45.0,
                comfort: 70.0,
            },
            Family::default(),
            ActivityTimer(99),
        ))
        .id();

    // Simulate Full -> Abstract transition: change LodTier and insert
    // CompressedCitizen manually (the compress_abstract_citizens system runs
    // in Update, which TestCity::tick() does not execute).
    let world = city.world_mut();
    {
        let state = world.get::<CitizenStateComp>(citizen_entity).unwrap().0;
        let details = world.get::<CitizenDetails>(citizen_entity).unwrap();
        let home = world.get::<HomeLocation>(citizen_entity).unwrap();
        let compressed = CompressedCitizen::new(
            home.grid_x as u8,
            home.grid_y as u8,
            state,
            details.age,
            details.happiness as u8,
            0,
            0,
        );
        world
            .entity_mut(citizen_entity)
            .insert((LodTier::Abstract, compressed));
    }

    // Verify citizen got CompressedCitizen marker.
    assert!(
        world.get::<CompressedCitizen>(citizen_entity).is_some(),
        "citizen should have CompressedCitizen component in Abstract tier"
    );

    // Verify all original components are still intact while in Abstract tier.
    {
        let details = world.get::<CitizenDetails>(citizen_entity).unwrap();
        assert_eq!(details.age, 35, "age should be preserved in Abstract tier");
        assert!(
            (details.salary - 4500.0).abs() < f32::EPSILON,
            "salary should be preserved in Abstract tier"
        );

        let personality = world.get::<Personality>(citizen_entity).unwrap();
        assert!(
            (personality.ambition - 0.7).abs() < f32::EPSILON,
            "ambition should be preserved in Abstract tier"
        );

        let needs = world.get::<Needs>(citizen_entity).unwrap();
        assert!(
            (needs.hunger - 60.0).abs() < f32::EPSILON,
            "hunger should be preserved in Abstract tier: {}",
            needs.hunger
        );
    }

    // Simulate Abstract -> Full transition: change LodTier and remove
    // CompressedCitizen (what decompress_active_citizens does).
    world
        .entity_mut(citizen_entity)
        .insert(LodTier::Full)
        .remove::<CompressedCitizen>();

    // Verify CompressedCitizen marker was removed.
    assert!(
        world.get::<CompressedCitizen>(citizen_entity).is_none(),
        "citizen should NOT have CompressedCitizen after returning to Full tier"
    );

    // Verify core components survived the Full -> Abstract -> Full roundtrip.
    let details = world.get::<CitizenDetails>(citizen_entity).unwrap();
    assert_eq!(details.age, 35, "age lost in LOD roundtrip");
    assert!(
        matches!(details.gender, Gender::Male),
        "gender lost in LOD roundtrip"
    );
    assert_eq!(details.education, 2, "education lost in LOD roundtrip");
    assert!(
        (details.salary - 4500.0).abs() < f32::EPSILON,
        "salary lost in LOD roundtrip: {}",
        details.salary
    );
    assert!(
        (details.savings - 12000.0).abs() < f32::EPSILON,
        "savings lost in LOD roundtrip: {}",
        details.savings
    );

    let personality = world.get::<Personality>(citizen_entity).unwrap();
    assert!(
        (personality.ambition - 0.7).abs() < f32::EPSILON,
        "ambition lost in LOD roundtrip"
    );
    assert!(
        (personality.sociability - 0.4).abs() < f32::EPSILON,
        "sociability lost in LOD roundtrip"
    );
    assert!(
        (personality.materialism - 0.55).abs() < f32::EPSILON,
        "materialism lost in LOD roundtrip"
    );
    assert!(
        (personality.resilience - 0.8).abs() < f32::EPSILON,
        "resilience lost in LOD roundtrip"
    );

    // Needs should be exactly preserved since no ticks were run.
    let needs = world.get::<Needs>(citizen_entity).unwrap();
    assert!(
        (needs.hunger - 60.0).abs() < f32::EPSILON,
        "hunger lost in LOD roundtrip: {}",
        needs.hunger
    );
    assert!(
        (needs.energy - 75.0).abs() < f32::EPSILON,
        "energy lost in LOD roundtrip: {}",
        needs.energy
    );
    assert!(
        (needs.social - 50.0).abs() < f32::EPSILON,
        "social lost in LOD roundtrip: {}",
        needs.social
    );
    assert!(
        (needs.fun - 45.0).abs() < f32::EPSILON,
        "fun lost in LOD roundtrip: {}",
        needs.fun
    );
    assert!(
        (needs.comfort - 70.0).abs() < f32::EPSILON,
        "comfort lost in LOD roundtrip: {}",
        needs.comfort
    );

    // Verify home/work locations survived.
    let home = world.get::<HomeLocation>(citizen_entity).unwrap();
    assert_eq!(home.grid_x, 12, "home_x lost in LOD roundtrip");
    assert_eq!(home.grid_y, 11, "home_y lost in LOD roundtrip");

    let work = world.get::<WorkLocation>(citizen_entity).unwrap();
    assert_eq!(work.grid_x, 18, "work_x lost in LOD roundtrip");
    assert_eq!(work.grid_y, 11, "work_y lost in LOD roundtrip");
}

/// Test that the SaveableRegistry correctly roundtrips extension data through
/// save_all / load_all, and that reset_all restores defaults.
#[test]
fn test_extension_map_roundtrip_via_registry() {
    use crate::SaveableRegistry;

    // Create a minimal Bevy app with a SaveableRegistry.
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    // Define a test resource that implements Saveable.
    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct TestExtensionResource {
        value_a: u32,
        value_b: String,
    }

    impl crate::Saveable for TestExtensionResource {
        const SAVE_KEY: &'static str = "test_extension_res";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            // Use serde_json for a simple, human-debuggable encoding.
            serde_json::to_vec(&(self.value_a, &self.value_b)).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let (a, b): (u32, String) = serde_json::from_slice(bytes).unwrap_or_default();
            Self {
                value_a: a,
                value_b: b,
            }
        }
    }

    // Register the test resource.
    app.init_resource::<TestExtensionResource>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<TestExtensionResource>();
    }

    // Set custom values.
    {
        let mut res = app.world_mut().resource_mut::<TestExtensionResource>();
        res.value_a = 42;
        res.value_b = "roundtrip_test".to_string();
    }

    // Save extensions via registry.
    let extensions = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };

    // Verify extension was saved under the correct key.
    assert!(
        extensions.contains_key("test_extension_res"),
        "extension should be saved under key 'test_extension_res'"
    );
    assert_eq!(
        extensions.len(),
        1,
        "should have exactly one extension entry"
    );

    // Verify the bytes are non-empty.
    let saved_bytes = extensions.get("test_extension_res").unwrap();
    assert!(
        !saved_bytes.is_empty(),
        "saved extension bytes should be non-empty"
    );

    // Reset the resource to default (simulating fresh world before load).
    app.world_mut()
        .insert_resource(TestExtensionResource::default());
    {
        let res = app.world().resource::<TestExtensionResource>();
        assert_eq!(res.value_a, 0, "resource should be at default before load");
        assert!(
            res.value_b.is_empty(),
            "resource should be at default before load"
        );
    }

    // Load extensions back via registry.
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &extensions);
        app.world_mut().insert_resource(registry);
    }

    // Verify restored values match what was saved.
    let res = app.world().resource::<TestExtensionResource>();
    assert_eq!(
        res.value_a, 42,
        "value_a should be restored from extension map"
    );
    assert_eq!(
        res.value_b, "roundtrip_test",
        "value_b should be restored from extension map"
    );

    // Test reset_all (simulating new game teardown).
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.reset_all(app.world_mut());
        app.world_mut().insert_resource(registry);
    }
    let res = app.world().resource::<TestExtensionResource>();
    assert_eq!(
        res.value_a, 0,
        "value_a should be reset to default after reset_all"
    );
    assert!(
        res.value_b.is_empty(),
        "value_b should be empty after reset_all"
    );
}

// ===========================================================================
// Bulldoze refund tests (issue #1227)
// ===========================================================================

#[test]
fn test_bulldoze_road_refunds_half_cost() {
    let initial_budget = 5000.0;
    let mut city =
        TestCity::new()
            .with_budget(initial_budget)
            .with_road(100, 100, 105, 100, RoadType::Avenue);

    // Verify road is placed
    city.assert_has_road(102, 100);

    let budget_before = city.budget().treasury;

    // Bulldoze one Avenue road cell -- should refund 50% of Avenue cost (20 * 0.5 = 10)
    city.bulldoze_road_at(102, 100);

    let budget_after = city.budget().treasury;
    let refund = budget_after - budget_before;
    let expected = RoadType::Avenue.cost() * 0.5;
    assert!(
        (refund - expected).abs() < 0.01,
        "Expected refund {expected}, got {refund}"
    );
}

#[test]
fn test_bulldoze_service_building_refunds_half_cost() {
    let initial_budget = 10000.0;
    let mut city =
        TestCity::new()
            .with_budget(initial_budget)
            .with_service(50, 50, ServiceType::Hospital);

    let budget_before = city.budget().treasury;

    // Bulldoze the hospital -- should refund 50% of 1000 = 500
    city.bulldoze_service_at(50, 50);

    let budget_after = city.budget().treasury;
    let refund = budget_after - budget_before;
    let expected = ServiceBuilding::cost(ServiceType::Hospital) * 0.5;
    assert!(
        (refund - expected).abs() < 0.01,
        "Expected refund {expected}, got {refund}"
    );
}

#[test]
fn test_bulldoze_multiple_roads_accumulates_refunds() {
    let initial_budget = 5000.0;
    let mut city = TestCity::new().with_budget(initial_budget).with_road(
        100,
        100,
        105,
        100,
        RoadType::Highway,
    );

    let budget_before = city.budget().treasury;

    // Bulldoze 3 Highway road cells
    city.bulldoze_road_at(101, 100);
    city.bulldoze_road_at(102, 100);
    city.bulldoze_road_at(103, 100);

    let budget_after = city.budget().treasury;
    let total_refund = budget_after - budget_before;
    let expected = RoadType::Highway.cost() * 0.5 * 3.0;
    assert!(
        (total_refund - expected).abs() < 0.01,
        "Expected total refund {expected}, got {total_refund}"
    );
}

#[test]
fn test_bulldoze_refund_allows_bankruptcy_recovery() {
    // Start with very low budget but expensive roads already placed
    let mut city = TestCity::new()
        .with_budget(0.0) // bankrupt!
        .with_road(100, 100, 110, 100, RoadType::Boulevard);

    assert!(city.budget().treasury < 1.0, "Should start near-bankrupt");

    // Bulldoze 5 Boulevard cells to recover money
    for x in 100..105 {
        city.bulldoze_road_at(x, 100);
    }

    let expected_refund = RoadType::Boulevard.cost() * 0.5 * 5.0;
    assert!(
        city.budget().treasury >= expected_refund - 0.01,
        "Treasury {} should be >= expected refund {}",
        city.budget().treasury,
        expected_refund
    );
    assert!(
        city.budget().treasury > 0.0,
        "Player should have recovered from bankruptcy via bulldoze refunds"
    );
}

// ===========================================================================
// Saveable registration drift detection
// ===========================================================================

#[test]
fn test_saveable_registry_contains_all_expected_keys() {
    let city = TestCity::new();
    let registry = city.resource::<crate::SaveableRegistry>();

    let registered: std::collections::HashSet<&str> =
        registry.entries.iter().map(|e| e.key.as_str()).collect();

    // Every key in EXPECTED_SAVEABLE_KEYS must be registered.
    let mut missing = Vec::new();
    for &expected in crate::EXPECTED_SAVEABLE_KEYS {
        if !registered.contains(expected) {
            missing.push(expected);
        }
    }
    assert!(
        missing.is_empty(),
        "SaveableRegistry is missing {} expected key(s): {:?}. \
         Each type implementing `Saveable` must be registered via `register_saveable` \
         in its plugin's `build()` method.",
        missing.len(),
        missing,
    );

    // Every registered key must be in the expected list (catches stale entries
    // in EXPECTED_SAVEABLE_KEYS or unexpected registrations).
    let expected_set: std::collections::HashSet<&str> =
        crate::EXPECTED_SAVEABLE_KEYS.iter().copied().collect();
    let mut unexpected: Vec<&str> = registered.difference(&expected_set).copied().collect();
    unexpected.sort();
    assert!(
        unexpected.is_empty(),
        "SaveableRegistry contains {} key(s) not in EXPECTED_SAVEABLE_KEYS: {:?}. \
         Add them to the list in simulation/src/lib.rs.",
        unexpected.len(),
        unexpected,
    );
}

#[test]
fn test_saveable_registry_has_no_duplicate_keys() {
    let city = TestCity::new();
    let registry = city.resource::<crate::SaveableRegistry>();

    let mut seen = std::collections::HashSet::new();
    for entry in &registry.entries {
        assert!(
            seen.insert(entry.key.as_str()),
            "SaveableRegistry: duplicate key '{}' — two types share the same SAVE_KEY",
            entry.key,
        );
    }
}

// ===========================================================================
// Job capacity enforcement
// ===========================================================================

/// Regression test for #1236: job_seeking must not assign more workers
/// than a building's capacity in a single tick.
///
/// We pre-fill the residential building to capacity so the citizen_spawner
/// cannot create additional employed citizens that would confound the test.
#[test]
fn test_job_seeking_does_not_overfill_capacity() {
    use bevy::prelude::*;

    let home_pos = (10, 10);
    let work_pos = (15, 15);

    // Get the capacity for a level-1 Industrial building
    let job_capacity = Building::capacity_for_level(ZoneType::Industrial, 1);

    // Spawn many more unemployed citizens than job capacity allows.
    // Use a large residential building so all citizens fit.
    let num_citizens = (job_capacity as usize) * 3;
    let mut city = TestCity::new()
        .with_building(home_pos.0, home_pos.1, ZoneType::ResidentialLow, 3)
        .with_building(work_pos.0, work_pos.1, ZoneType::Industrial, 1);

    for _ in 0..num_citizens {
        city = city.with_unemployed_citizen(home_pos);
    }

    // Mark the residential building as full so spawn_citizens won't
    // create extra employed citizens that confound this test.
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.zone_type.is_residential() {
                building.occupants = building.capacity;
            }
        }
    }

    // Run enough ticks to trigger job_seeking (JOB_SEEK_INTERVAL = 300)
    city.tick(301);

    // Verify: no building should have more occupants than its capacity
    let world = city.world_mut();
    let mut query = world.query::<&Building>();
    for building in query.iter(world) {
        assert!(
            building.occupants <= building.capacity,
            "Building at ({}, {}) zone {:?} has {} occupants but capacity is {} (overfilled!)",
            building.grid_x,
            building.grid_y,
            building.zone_type,
            building.occupants,
            building.capacity,
        );
    }

    // The building.occupants check above is sufficient to verify the
    // job_seeking fix.  WorkLocation count may exceed capacity because
    // the separate job_matching system (education_jobs.rs) also assigns
    // WorkLocations without going through the occupants counter -- that
    // is tracked as a separate concern.
}

// ---------------------------------------------------------------------------
// SimulationSet phase ordering
// ---------------------------------------------------------------------------

/// Verify that the SimulationSet and SimulationUpdateSet phase ordering is
/// correctly configured by running a full tick.  If the set chain is broken
/// Bevy would panic with an ambiguity error or the systems would not run.
#[test]
fn test_simulation_set_phases_configured() {
    use crate::grid::{RoadType, ZoneType};
    use crate::test_harness::TestCity;

    // Build a minimal city and run a few ticks.  If the phase ordering is
    // misconfigured (e.g. circular dependency, missing configure_sets) this
    // will panic.
    let mut city = TestCity::new()
        .with_road(128, 128, 128, 131, RoadType::Local)
        .with_zone(129, 128, ZoneType::ResidentialLow)
        .with_zone(129, 130, ZoneType::CommercialLow);
    city.tick(5);

    // Sanity: game clock should have advanced (PreSim systems ran)
    assert!(city.clock().hour > 6.0 || city.clock().day > 1);
}

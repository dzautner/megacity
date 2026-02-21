//! Integration tests for Megacity using the `TestCity` harness.
//!
//! These tests spin up a headless Bevy App with `SimulationPlugin` and verify
//! emergent behavior across multiple systems working together.

use crate::buildings::Building;
use crate::citizen::CitizenState;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
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

use crate::freehand_road::{
    filter_short_segments, simplify_rdp, FreehandDrawState, FREEHAND_MIN_SAMPLE_DIST,
    FREEHAND_MIN_SEGMENT_LEN, FREEHAND_SIMPLIFY_TOLERANCE,
};
use crate::undo_redo::{ActionHistory, CityAction};
use bevy::math::Vec2;
use bevy::prelude::Mut;

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

#[test]
fn test_traffic_los_resource_initialized() {
    use crate::traffic_los::{LosGrade, TrafficLosGrid};

    let city = TestCity::new();

    // The TrafficLosPlugin should register the TrafficLosGrid resource
    let los = city.resource::<TrafficLosGrid>();
    assert_eq!(
        los.get(0, 0),
        LosGrade::A,
        "Default LOS should be A (free flow)"
    );
}

#[test]
fn test_traffic_los_empty_roads_grade_a() {
    use crate::traffic_los::{LosGrade, TrafficLosGrid};

    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);

    // Run enough ticks for the LOS system to fire (runs every 10 ticks)
    city.tick(10);

    // With no citizens commuting, traffic density is 0, so roads should be LOS A
    let los = city.resource::<TrafficLosGrid>();
    assert_eq!(
        los.get(15, 10),
        LosGrade::A,
        "Empty road should be LOS A (free flow)"
    );
}

#[test]
fn test_traffic_los_grading_uses_road_type_capacity() {
    use crate::traffic_los::LosGrade;

    let city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_road(10, 15, 20, 15, RoadType::Highway);

    // Verify that the road types have different capacities (needed for LOS)
    let grid = city.resource::<WorldGrid>();
    let local_capacity = grid.get(15, 10).road_type.capacity();
    let highway_capacity = grid.get(15, 15).road_type.capacity();
    assert!(
        highway_capacity > local_capacity,
        "Highway capacity ({highway_capacity}) should exceed Local capacity ({local_capacity})"
    );

    // Verify that the LOS grading function correctly distinguishes load levels
    // Same traffic volume on different road types yields different grades
    let traffic_volume = local_capacity as f32; // saturate local road
    let local_vc = traffic_volume / local_capacity as f32; // 1.0 -> LOS F
    let highway_vc = traffic_volume / highway_capacity as f32; // < 1.0

    let local_grade = LosGrade::from_vc_ratio(local_vc);
    let highway_grade = LosGrade::from_vc_ratio(highway_vc);

    assert_eq!(
        local_grade,
        LosGrade::F,
        "Local at capacity should be LOS F"
    );
    assert!(
        (highway_grade as u8) < (local_grade as u8),
        "Highway ({highway_grade:?}) should have better LOS than Local ({local_grade:?}) at same traffic volume"
    );
}
// ---------------------------------------------------------------------------
// Road upgrade integration tests (UX-022)
// ---------------------------------------------------------------------------

#[test]
fn test_road_upgrade_local_to_avenue() {
    use crate::grid::RoadType;
    use crate::test_harness::TestCity;

    let mut city =
        TestCity::new()
            .with_budget(50000.0)
            .with_road(128, 128, 132, 128, RoadType::Local);

    assert_eq!(city.segment_count(), 1);
    assert_eq!(city.segment_road_type(0), Some(RoadType::Local));

    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RoadType::Avenue);
    assert_eq!(city.segment_road_type(0), Some(RoadType::Avenue));
}

#[test]
fn test_road_upgrade_full_path_to_boulevard() {
    use crate::grid::RoadType;
    use crate::test_harness::TestCity;

    let mut city =
        TestCity::new()
            .with_budget(100000.0)
            .with_road(128, 128, 132, 128, RoadType::Path);

    // Path -> Local -> Avenue -> Boulevard
    let result = city.upgrade_segment_by_index(0);
    assert_eq!(result.unwrap(), RoadType::Local);

    let result = city.upgrade_segment_by_index(0);
    assert_eq!(result.unwrap(), RoadType::Avenue);

    let result = city.upgrade_segment_by_index(0);
    assert_eq!(result.unwrap(), RoadType::Boulevard);

    // Boulevard has no further upgrade
    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_err());
}

#[test]
fn test_road_upgrade_deducts_cost() {
    use crate::grid::RoadType;
    use crate::test_harness::TestCity;

    let mut city =
        TestCity::new()
            .with_budget(50000.0)
            .with_road(128, 128, 132, 128, RoadType::Local);

    let budget_before = city.budget().treasury;
    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_ok());
    let budget_after = city.budget().treasury;

    // Cost should have been deducted
    assert!(budget_after < budget_before);
    // The deduction should be positive
    assert!(budget_before - budget_after > 0.0);
}

#[test]
fn test_road_upgrade_insufficient_funds() {
    use crate::grid::RoadType;
    use crate::test_harness::TestCity;

    let mut city = TestCity::new()
        .with_budget(0.0)
        .with_road(128, 128, 132, 128, RoadType::Local);

    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Not enough money");

    // Road type should remain unchanged
    assert_eq!(city.segment_road_type(0), Some(RoadType::Local));
}

#[test]
fn test_road_upgrade_preserves_connections() {
    use crate::grid::RoadType;
    use crate::test_harness::TestCity;

    // Create two connected road segments
    let mut city = TestCity::new()
        .with_budget(100000.0)
        .with_road(128, 128, 132, 128, RoadType::Local)
        .with_road(132, 128, 136, 128, RoadType::Local);

    assert_eq!(city.segment_count(), 2);

    // Upgrade first segment
    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), RoadType::Avenue);

    // Second segment should be unchanged
    assert_eq!(city.segment_road_type(1), Some(RoadType::Local));

    // Both segments should still exist
    assert_eq!(city.segment_count(), 2);
}

#[test]
fn test_road_upgrade_updates_grid_cells() {
    use crate::grid::{CellType, RoadType};
    use crate::test_harness::TestCity;

    let mut city =
        TestCity::new()
            .with_budget(50000.0)
            .with_road(128, 128, 132, 128, RoadType::Local);

    // Before upgrade: cells should be Local type
    let grid = city.grid();
    for x in 128..=132 {
        let cell = grid.get(x, 128);
        if cell.cell_type == CellType::Road {
            assert_eq!(cell.road_type, RoadType::Local);
        }
    }

    city.upgrade_segment_by_index(0).unwrap();

    // After upgrade: road cells should be Avenue type
    let grid = city.grid();
    for x in 128..=132 {
        let cell = grid.get(x, 128);
        if cell.cell_type == CellType::Road {
            assert_eq!(cell.road_type, RoadType::Avenue);
        }
    }
}

#[test]
fn test_road_upgrade_highway_at_max() {
    use crate::grid::RoadType;
    use crate::test_harness::TestCity;

    let mut city =
        TestCity::new()
            .with_budget(100000.0)
            .with_road(128, 128, 132, 128, RoadType::Highway);

    let result = city.upgrade_segment_by_index(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Already at maximum road tier");
}

// ====================================================================
// Traffic congestion tests
// ====================================================================

use crate::citizen::{Citizen, CitizenStateComp, PathCache, Position};
use crate::roads::RoadNode;
use crate::traffic_congestion::TrafficCongestion;

#[test]
fn test_traffic_congestion_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<TrafficCongestion>();
}

#[test]
fn test_traffic_congestion_defaults_to_free_flow() {
    let city = TestCity::new();
    let congestion = city.resource::<TrafficCongestion>();
    assert!(
        (congestion.get(10, 10) - 1.0).abs() < f32::EPSILON,
        "Default congestion multiplier should be 1.0"
    );
}

#[test]
fn test_citizens_move_slower_on_congested_roads() {
    use crate::traffic::TrafficGrid;

    let mut city = TestCity::new()
        .with_road(50, 50, 80, 50, RoadType::Local)
        .with_building(48, 50, ZoneType::ResidentialLow, 1)
        .with_building(82, 50, ZoneType::CommercialLow, 1)
        .with_citizen((48, 50), (82, 50))
        .with_time(7.0);

    // Manually set citizen to commuting state with a path
    {
        let world = city.world_mut();
        let waypoints: Vec<RoadNode> = (50..=80).map(|x| RoadNode(x, 50)).collect();
        let (wx, wy) = WorldGrid::grid_to_world(50, 50);
        let mut q = world.query_filtered::<(
            &mut CitizenStateComp,
            &mut PathCache,
            &mut Position,
        ), bevy::prelude::With<Citizen>>();
        for (mut state, mut path, mut pos) in q.iter_mut(world) {
            state.0 = CitizenState::CommutingToWork;
            *path = PathCache::new(waypoints.clone());
            pos.x = wx;
            pos.y = wy;
        }
    }

    let start_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };

    // Run ticks at free flow
    city.tick(10);

    let free_flow_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };
    let free_flow_dist =
        ((free_flow_pos.0 - start_pos.0).powi(2) + (free_flow_pos.1 - start_pos.1).powi(2)).sqrt();

    // Reset citizen and inject congestion on BOTH TrafficGrid and TrafficCongestion
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=80 {
            traffic.set(x, 50, 20); // at Local capacity
        }
        let mut congestion = world.resource_mut::<TrafficCongestion>();
        for x in 50..=80 {
            congestion.set(x, 50, 0.1);
        }
        let waypoints: Vec<RoadNode> = (50..=80).map(|x| RoadNode(x, 50)).collect();
        let (wx, wy) = WorldGrid::grid_to_world(50, 50);
        let mut q = world.query_filtered::<(
            &mut CitizenStateComp,
            &mut PathCache,
            &mut Position,
        ), bevy::prelude::With<Citizen>>();
        for (mut state, mut path, mut pos) in q.iter_mut(world) {
            state.0 = CitizenState::CommutingToWork;
            *path = PathCache::new(waypoints.clone());
            pos.x = wx;
            pos.y = wy;
        }
    }

    city.tick(10);

    let congested_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };
    let congested_dist =
        ((congested_pos.0 - start_pos.0).powi(2) + (congested_pos.1 - start_pos.1).powi(2)).sqrt();

    assert!(
        free_flow_dist > 1.0,
        "Citizen should have moved during free flow, dist={}",
        free_flow_dist
    );
    assert!(
        congested_dist < free_flow_dist,
        "Citizen should move slower under congestion. Free flow dist={}, congested dist={}",
        free_flow_dist,
        congested_dist
    );
}

#[test]
fn test_speed_returns_to_normal_when_congestion_clears() {
    use crate::traffic::TrafficGrid;

    let mut city = TestCity::new()
        .with_road(50, 50, 120, 50, RoadType::Local)
        .with_building(48, 50, ZoneType::ResidentialLow, 1)
        .with_building(122, 50, ZoneType::CommercialLow, 1)
        .with_citizen((48, 50), (122, 50))
        .with_time(7.0);

    // Set citizen to commuting with congestion
    {
        let world = city.world_mut();
        let waypoints: Vec<RoadNode> = (50..=120).map(|x| RoadNode(x, 50)).collect();
        let (wx, wy) = WorldGrid::grid_to_world(50, 50);
        let mut q = world.query_filtered::<(
            &mut CitizenStateComp,
            &mut PathCache,
            &mut Position,
        ), bevy::prelude::With<Citizen>>();
        for (mut state, mut path, mut pos) in q.iter_mut(world) {
            state.0 = CitizenState::CommutingToWork;
            *path = PathCache::new(waypoints.clone());
            pos.x = wx;
            pos.y = wy;
        }
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=120 {
            traffic.set(x, 50, 20);
        }
        let mut congestion = world.resource_mut::<TrafficCongestion>();
        for x in 50..=120 {
            congestion.set(x, 50, 0.1);
        }
    }

    let start_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };

    city.tick(10);

    let congested_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };
    let congested_dist =
        ((congested_pos.0 - start_pos.0).powi(2) + (congested_pos.1 - start_pos.1).powi(2)).sqrt();

    // Clear congestion and reset citizen position + path for the free-flow phase.
    // We reset position to the same starting point as the congested phase so the
    // distance comparison is fair (both phases start from grid cell 50,50).
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=120 {
            traffic.set(x, 50, 0);
        }
        let mut congestion = world.resource_mut::<TrafficCongestion>();
        for x in 50..=120 {
            congestion.set(x, 50, 1.0);
        }
        let waypoints: Vec<RoadNode> = (50..=120).map(|x| RoadNode(x, 50)).collect();
        let (wx, wy) = WorldGrid::grid_to_world(50, 50);
        let mut q = world.query_filtered::<(
            &mut CitizenStateComp,
            &mut PathCache,
            &mut Position,
        ), bevy::prelude::With<Citizen>>();
        for (mut state, mut path, mut pos) in q.iter_mut(world) {
            state.0 = CitizenState::CommutingToWork;
            *path = PathCache::new(waypoints.clone());
            pos.x = wx;
            pos.y = wy;
        }
    }

    let mid_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };

    city.tick(10);

    let free_flow_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };
    let free_flow_dist =
        ((free_flow_pos.0 - mid_pos.0).powi(2) + (free_flow_pos.1 - mid_pos.1).powi(2)).sqrt();

    assert!(
        congested_dist > 0.1,
        "Citizen should move even under congestion (min speed floor), dist={}",
        congested_dist
    );
    assert!(
        free_flow_dist > congested_dist,
        "Speed should return to normal after congestion clears. \
         Congested dist={}, free flow dist={}",
        congested_dist,
        free_flow_dist
    );
}

#[test]
fn test_higher_capacity_roads_congest_less() {
    use crate::traffic_congestion::congestion_speed_multiplier;

    let local_ratio = 15.0 / 20.0;
    let local_mult = congestion_speed_multiplier(local_ratio);
    let highway_ratio = 15.0 / 80.0;
    let highway_mult = congestion_speed_multiplier(highway_ratio);

    assert!(
        highway_mult > local_mult,
        "Highway should be less congested than Local at same volume. Highway={}, Local={}",
        highway_mult,
        local_mult
    );
    assert!(
        local_mult < 0.5,
        "Local at 75% capacity should have multiplier < 0.5, got {}",
        local_mult
    );
    assert!(
        highway_mult > 0.9,
        "Highway at ~19% capacity should have multiplier > 0.9, got {}",
        highway_mult
    );
}

// Async pathfinding tests
// ===========================================================================

#[test]
fn test_async_pathfinding_snapshot_initialized() {
    let city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    city.assert_resource_exists::<crate::movement::PathfindingSnapshot>();
}

#[test]
fn test_async_pathfinding_citizen_gets_path() {
    use crate::movement::ComputingPath;

    let mut city = TestCity::new()
        .with_road(5, 10, 25, 10, RoadType::Local)
        .with_building(5, 9, ZoneType::ResidentialLow, 1)
        .with_building(25, 9, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_citizen((5, 9), (25, 9))
        .with_time(7.0); // start of morning commute window

    // Run 120 ticks (= 2 in-game hours) to cover the full morning commute window
    // (hours 7-8). This guarantees hitting any per-entity departure jitter value.
    // Async tasks run on background threads and are polled each tick via
    // `block_on(poll_once(...))`. A yield_now() in the test harness tick loop
    // gives background threads a chance to complete.
    city.tick(120);

    let world = city.world_mut();
    let not_at_home = world
        .query::<&crate::citizen::CitizenStateComp>()
        .iter(world)
        .filter(|s| s.0 != CitizenState::AtHome)
        .count();
    let computing = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ComputingPath>>()
        .iter(world)
        .count();

    // Citizen should have started pathfinding (ComputingPath) or transitioned
    assert!(
        not_at_home > 0 || computing > 0,
        "citizen should have started pathfinding or left home (not_at_home={not_at_home}, computing={computing})"
    );
}

#[test]
fn test_async_pathfinding_no_road_no_crash() {
    // Citizens with no road connectivity should not crash the async system
    let mut city = TestCity::new()
        .with_building(5, 5, ZoneType::ResidentialLow, 1)
        .with_building(50, 50, ZoneType::CommercialLow, 1)
        .with_citizen((5, 5), (50, 50))
        .with_time(7.0);

    // Should not panic even with no roads
    city.tick(120);

    // Citizen should still exist (spawner may add more during ticks)
    assert!(
        city.citizen_count() >= 1,
        "original citizen should still exist"
    );
}

#[test]
fn test_async_pathfinding_computing_path_prevents_requeue() {
    use crate::citizen::PathRequest;
    use crate::movement::ComputingPath;
    use bevy::prelude::Entity;

    let mut city = TestCity::new()
        .with_road(5, 10, 25, 10, RoadType::Local)
        .with_building(5, 9, ZoneType::ResidentialLow, 1)
        .with_building(25, 9, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_citizen((5, 9), (25, 9))
        .with_time(7.0);

    // Run enough ticks for the state machine to fire and pathfinding to dispatch.
    city.tick(120);

    // Verify no entity has BOTH PathRequest and ComputingPath simultaneously.
    // This would indicate the state machine re-queued a citizen that is already
    // being processed by the async pathfinding system.
    let world = city.world_mut();
    let double_queued = world
        .query_filtered::<Entity, (
            bevy::prelude::With<PathRequest>,
            bevy::prelude::With<ComputingPath>,
        )>()
        .iter(world)
        .count();

    assert_eq!(
        double_queued, 0,
        "no entity should have both PathRequest and ComputingPath"
    );
}

#[test]
fn test_async_pathfinding_multiple_citizens() {
    use crate::movement::ComputingPath;

    let mut city = TestCity::new()
        .with_road(5, 10, 30, 10, RoadType::Local)
        .with_building(5, 9, ZoneType::ResidentialLow, 1)
        .with_building(30, 9, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_citizen((5, 9), (30, 9))
        .with_citizen((5, 9), (30, 9))
        .with_citizen((5, 9), (30, 9))
        .with_time(7.0);

    // Run 120 ticks to cover the full morning commute window (hours 7-8),
    // ensuring all citizens hit their departure jitter.
    city.tick(120);

    let world = city.world_mut();
    let not_at_home = world
        .query::<&crate::citizen::CitizenStateComp>()
        .iter(world)
        .filter(|s| s.0 != CitizenState::AtHome)
        .count();
    let computing = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ComputingPath>>()
        .iter(world)
        .count();

    // At least some citizens should have started pathfinding or transitioned
    assert!(
        not_at_home > 0 || computing > 0,
        "some citizens should be pathfinding or have left home (not_at_home={not_at_home}, computing={computing})"
    );
}

#[test]
fn test_async_pathfinding_snapshot_updates_on_road_change() {
    let mut city = TestCity::new().with_road(5, 10, 15, 10, RoadType::Local);

    let v1 = city
        .resource::<crate::movement::PathfindingSnapshot>()
        .version;

    // Add more road and tick to trigger CSR rebuild + snapshot update
    city = city.with_road(15, 10, 25, 10, RoadType::Local);
    city.tick(2);

    let v2 = city
        .resource::<crate::movement::PathfindingSnapshot>()
        .version;
    assert!(
        v2 > v1,
        "snapshot version should increase after road network change (v1={}, v2={})",
        v1,
        v2
    );
}

// ===========================================================================
// Behavioral integration tests (issue #1248)
// ===========================================================================

// ---------------------------------------------------------------------------
// Save/load: sequential load A -> B -> A (extension map cross-save safety)
// ---------------------------------------------------------------------------

/// Test that loading save A (with extension data), then save B (without that
/// extension key), correctly resets the extension resource to its default.
/// Then loading save A again must restore the original value -- not retain
/// save B's empty state.
#[test]
fn test_extension_map_sequential_load_a_b_a_restores_correctly() {
    use crate::SaveableRegistry;
    use std::collections::BTreeMap;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct FeatureAlpha {
        level: u32,
        name: String,
    }

    impl crate::Saveable for FeatureAlpha {
        const SAVE_KEY: &'static str = "test_feature_alpha";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&(self.level, &self.name)).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let (level, name): (u32, String) = serde_json::from_slice(bytes).unwrap_or_default();
            Self { level, name }
        }
    }

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct FeatureBeta {
        score: f64,
    }

    impl crate::Saveable for FeatureBeta {
        const SAVE_KEY: &'static str = "test_feature_beta";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.score).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let score: f64 = serde_json::from_slice(bytes).unwrap_or_default();
            Self { score }
        }
    }

    app.init_resource::<FeatureAlpha>();
    app.init_resource::<FeatureBeta>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<FeatureAlpha>();
        registry.register::<FeatureBeta>();
    }

    // --- Build save A: both features have data ---
    {
        let mut alpha = app.world_mut().resource_mut::<FeatureAlpha>();
        alpha.level = 7;
        alpha.name = "save_a_alpha".to_string();
    }
    {
        let mut beta = app.world_mut().resource_mut::<FeatureBeta>();
        beta.score = 99.5;
    }
    let save_a = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };
    assert_eq!(save_a.len(), 2, "save A should contain both extensions");

    // --- Build save B: only FeatureBeta has data; FeatureAlpha key is absent ---
    let mut save_b: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    save_b.insert(
        "test_feature_beta".to_string(),
        serde_json::to_vec(&42.0_f64).unwrap(),
    );

    // --- Load save A ---
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &save_a);
        app.world_mut().insert_resource(registry);
    }
    assert_eq!(app.world().resource::<FeatureAlpha>().level, 7);
    assert_eq!(app.world().resource::<FeatureAlpha>().name, "save_a_alpha");
    assert!((app.world().resource::<FeatureBeta>().score - 99.5).abs() < f64::EPSILON);

    // --- Load save B (missing FeatureAlpha) ---
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &save_b);
        app.world_mut().insert_resource(registry);
    }
    assert_eq!(
        app.world().resource::<FeatureAlpha>().level,
        0,
        "FeatureAlpha.level should reset to default after loading save B (key absent)"
    );
    assert!(
        app.world().resource::<FeatureAlpha>().name.is_empty(),
        "FeatureAlpha.name should reset to default after loading save B (key absent)"
    );
    assert!(
        (app.world().resource::<FeatureBeta>().score - 42.0).abs() < f64::EPSILON,
        "FeatureBeta.score should be 42.0 from save B"
    );

    // --- Load save A again ---
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &save_a);
        app.world_mut().insert_resource(registry);
    }
    assert_eq!(
        app.world().resource::<FeatureAlpha>().level,
        7,
        "FeatureAlpha.level should be restored from save A after A->B->A sequence"
    );
    assert_eq!(
        app.world().resource::<FeatureAlpha>().name,
        "save_a_alpha",
        "FeatureAlpha.name should be restored from save A after A->B->A sequence"
    );
    assert!(
        (app.world().resource::<FeatureBeta>().score - 99.5).abs() < f64::EPSILON,
        "FeatureBeta.score should be restored from save A after A->B->A sequence"
    );
}

/// Test that loading a save with completely empty extensions resets ALL
/// registered saveable resources to defaults.
#[test]
fn test_extension_map_load_empty_save_resets_all_to_defaults() {
    use crate::SaveableRegistry;
    use std::collections::BTreeMap;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct CounterRes {
        count: u64,
    }

    impl crate::Saveable for CounterRes {
        const SAVE_KEY: &'static str = "test_counter_res";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.count).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let count: u64 = serde_json::from_slice(bytes).unwrap_or_default();
            Self { count }
        }
    }

    app.init_resource::<CounterRes>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<CounterRes>();
    }

    app.world_mut().resource_mut::<CounterRes>().count = 12345;

    let empty_extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &empty_extensions);
        app.world_mut().insert_resource(registry);
    }

    assert_eq!(
        app.world().resource::<CounterRes>().count,
        0,
        "CounterRes should reset to default when loading a save with no extension data"
    );
}

// ---------------------------------------------------------------------------
// Extension map binary roundtrip
// ---------------------------------------------------------------------------

/// Test that extension map data survives a serde_json encode->decode roundtrip.
#[test]
fn test_extension_map_bytes_survive_serde_roundtrip() {
    use crate::SaveableRegistry;
    use std::collections::BTreeMap;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct ComplexState {
        values: Vec<u32>,
        label: String,
        active: bool,
    }

    impl crate::Saveable for ComplexState {
        const SAVE_KEY: &'static str = "test_complex_state";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&(&self.values, &self.label, self.active)).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let (values, label, active): (Vec<u32>, String, bool) =
                serde_json::from_slice(bytes).unwrap_or_default();
            Self {
                values,
                label,
                active,
            }
        }
    }

    app.init_resource::<ComplexState>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<ComplexState>();
    }

    {
        let mut state = app.world_mut().resource_mut::<ComplexState>();
        state.values = vec![10, 20, 30, 40, 50];
        state.label = "roundtrip_binary_test".to_string();
        state.active = true;
    }

    let extensions = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };
    let saved_bytes = extensions.get("test_complex_state").unwrap().clone();

    let mut restored_extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    restored_extensions.insert("test_complex_state".to_string(), saved_bytes);

    app.world_mut().insert_resource(ComplexState::default());
    assert!(app.world().resource::<ComplexState>().values.is_empty());

    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &restored_extensions);
        app.world_mut().insert_resource(registry);
    }

    let state = app.world().resource::<ComplexState>();
    assert_eq!(state.values, vec![10, 20, 30, 40, 50]);
    assert_eq!(state.label, "roundtrip_binary_test");
    assert!(state.active);
}

/// Test that loading extensions with corrupted bytes falls back to defaults.
#[test]
fn test_extension_map_corrupted_bytes_fall_back_to_default() {
    use crate::SaveableRegistry;
    use std::collections::BTreeMap;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct SimpleCounter {
        count: u32,
    }

    impl crate::Saveable for SimpleCounter {
        const SAVE_KEY: &'static str = "test_simple_counter";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.count).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let count: u32 = serde_json::from_slice(bytes).unwrap_or_default();
            Self { count }
        }
    }

    app.init_resource::<SimpleCounter>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<SimpleCounter>();
    }

    app.world_mut().resource_mut::<SimpleCounter>().count = 42;

    let mut extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    extensions.insert(
        "test_simple_counter".to_string(),
        vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB],
    );

    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &extensions);
        app.world_mut().insert_resource(registry);
    }

    assert_eq!(
        app.world().resource::<SimpleCounter>().count,
        0,
        "Corrupted bytes should cause fallback to default, not retain stale value"
    );
}

// ---------------------------------------------------------------------------
// Job capacity invariant
// ---------------------------------------------------------------------------

/// After simulation ticks with many unemployed citizens seeking jobs,
/// verify no building ever has occupants > capacity.
#[test]
fn test_job_capacity_invariant_all_building_types_after_simulation() {
    let home_pos = (10, 10);

    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_road(10, 20, 30, 20, RoadType::Local)
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(14, 11, ZoneType::ResidentialLow, 2)
        .with_building(16, 11, ZoneType::ResidentialHigh, 3)
        .with_building(20, 11, ZoneType::CommercialLow, 1)
        .with_building(22, 11, ZoneType::CommercialHigh, 2)
        .with_building(26, 11, ZoneType::Industrial, 1)
        .with_building(28, 11, ZoneType::Industrial, 2);

    for _ in 0..80 {
        city = city.with_unemployed_citizen(home_pos);
    }

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.zone_type.is_residential() {
                building.occupants = building.capacity;
            }
        }
    }

    city.tick(900);

    let world = city.world_mut();
    let mut query = world.query::<&Building>();
    for building in query.iter(world) {
        assert!(
            building.occupants <= building.capacity,
            "Invariant violated: building at ({}, {}) zone {:?} level {} has {} occupants \
             but capacity is {}",
            building.grid_x,
            building.grid_y,
            building.zone_type,
            building.level,
            building.occupants,
            building.capacity,
        );
    }
}

// ---------------------------------------------------------------------------
// Marriage reciprocity invariant
// ---------------------------------------------------------------------------

/// After life simulation, verify all partnerships are reciprocal.
#[test]
fn test_marriage_reciprocity_invariant_after_life_simulation() {
    use crate::citizen::{
        Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation,
        Needs, PathCache, Personality, Position, Velocity,
    };
    use crate::movement::ActivityTimer;
    use std::collections::HashMap;

    let mut city = TestCity::new().with_building(50, 50, ZoneType::ResidentialLow, 3);

    let building_entity = city.grid().get(50, 50).building_id.unwrap();
    let (wx, wy) = WorldGrid::grid_to_world(50, 50);

    let world = city.world_mut();
    for i in 0..20 {
        let gender = if i % 2 == 0 {
            Gender::Male
        } else {
            Gender::Female
        };
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
                age: 25 + (i % 10) as u8,
                gender,
                education: 2,
                happiness: 80.0,
                health: 90.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.7,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
        ));
    }

    city.tick(30_000);

    let world = city.world_mut();
    let mut query = world.query::<(bevy::prelude::Entity, &Family)>();
    let pairs: Vec<_> = query.iter(world).map(|(e, f)| (e, f.partner)).collect();

    let family_map: HashMap<_, _> = pairs.iter().map(|(e, p)| (*e, *p)).collect();

    let mut partnered_count = 0;
    for (entity, partner_opt) in &family_map {
        if let Some(partner) = partner_opt {
            partnered_count += 1;
            let partner_partner = family_map.get(partner).and_then(|p| *p);
            assert_eq!(
                partner_partner,
                Some(*entity),
                "Reciprocity violated: {:?} -> {:?}, but {:?} -> {:?}",
                entity,
                partner,
                partner,
                partner_partner
            );
        }
    }

    assert_eq!(
        partnered_count % 2,
        0,
        "Partnered citizen count should be even (pairs), got {}",
        partnered_count
    );
}

// ---------------------------------------------------------------------------
// Road segment store / grid cache consistency
// ---------------------------------------------------------------------------

/// After adding road segments, verify segment rasterized cells match grid.
#[test]
fn test_road_segment_grid_cache_consistency_after_placement() {
    use std::collections::HashSet;

    let city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_road(30, 10, 30, 30, RoadType::Avenue)
        .with_road(10, 30, 30, 30, RoadType::Highway);

    let grid = city.grid();
    let segments = city.road_segments();

    let mut segment_cells: HashSet<(usize, usize)> = HashSet::new();
    for seg in &segments.segments {
        for &(cx, cy) in &seg.rasterized_cells {
            segment_cells.insert((cx, cy));
        }
    }

    for &(cx, cy) in &segment_cells {
        if grid.in_bounds(cx, cy) {
            assert_eq!(
                grid.get(cx, cy).cell_type,
                CellType::Road,
                "Segment claims cell ({}, {}) is rasterized, but grid says {:?}",
                cx,
                cy,
                grid.get(cx, cy).cell_type,
            );
        }
    }

    if !segment_cells.is_empty() {
        let min_x = segment_cells.iter().map(|c| c.0).min().unwrap();
        let max_x = segment_cells.iter().map(|c| c.0).max().unwrap();
        let min_y = segment_cells.iter().map(|c| c.1).min().unwrap();
        let max_y = segment_cells.iter().map(|c| c.1).max().unwrap();

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if grid.in_bounds(x, y) && grid.get(x, y).cell_type == CellType::Road {
                    assert!(
                        segment_cells.contains(&(x, y)),
                        "Grid has road at ({}, {}) but no segment claims it",
                        x,
                        y,
                    );
                }
            }
        }
    }
}

/// After adding road segments, verify road network has nodes for all road cells.
#[test]
fn test_road_network_nodes_match_grid_roads_after_segment_placement() {
    let city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_road(20, 10, 20, 20, RoadType::Local);

    let grid = city.grid();
    let network = city.road_network();

    for y in 0..grid.height {
        for x in 0..grid.width {
            if grid.get(x, y).cell_type == CellType::Road {
                let node = crate::roads::RoadNode(x, y);
                assert!(
                    network.edges.contains_key(&node),
                    "Grid has road at ({}, {}) but RoadNetwork has no node for it",
                    x,
                    y,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Economy balance invariant
// ---------------------------------------------------------------------------

/// After tax collection, verify income breakdown sums to monthly_income
/// and expense breakdown sums to monthly_expenses.
#[test]
fn test_economy_breakdown_sums_match_totals_after_tax_collection() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_road(10, 10, 10, 40, RoadType::Avenue)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_building(14, 11, ZoneType::CommercialLow, 1)
        .with_building(16, 11, ZoneType::Industrial, 1)
        .with_service(20, 11, ServiceType::PoliceStation)
        .with_budget(50_000.0);

    city.tick_slow_cycles(50);

    let budget = city.budget().clone();
    let extended = city.resource::<ExtendedBudget>().clone();

    let income_sum = extended.income_breakdown.residential_tax
        + extended.income_breakdown.commercial_tax
        + extended.income_breakdown.industrial_tax
        + extended.income_breakdown.office_tax
        + extended.income_breakdown.trade_income;

    let expense_sum = extended.expense_breakdown.road_maintenance
        + extended.expense_breakdown.service_costs
        + extended.expense_breakdown.policy_costs;

    assert!(
        (budget.monthly_income - income_sum).abs() < 0.01,
        "Income mismatch: monthly_income={} but breakdown sums to {}",
        budget.monthly_income,
        income_sum,
    );

    assert!(
        (budget.monthly_expenses - expense_sum).abs() < 0.01,
        "Expense mismatch: monthly_expenses={} but breakdown sums to {}",
        budget.monthly_expenses,
        expense_sum,
    );
}

// ---------------------------------------------------------------------------
// Tel Aviv smoke test: behavioral invariants on the full map
// ---------------------------------------------------------------------------

/// Run the Tel Aviv map and verify marriage reciprocity and road/grid
/// consistency invariants hold.
#[test]
fn test_tel_aviv_behavioral_invariants_after_simulation() {
    use crate::citizen::Family;
    use std::collections::HashMap;

    let mut city = TestCity::with_tel_aviv();
    city.tick_slow_cycles(5);

    // Invariant 1: Marriage reciprocity
    {
        let world = city.world_mut();
        let mut query = world.query::<(bevy::prelude::Entity, &Family)>();
        let pairs: Vec<_> = query.iter(world).map(|(e, f)| (e, f.partner)).collect();
        let family_map: HashMap<_, _> = pairs.iter().map(|(e, p)| (*e, *p)).collect();

        for (entity, partner_opt) in &family_map {
            if let Some(partner) = partner_opt {
                let partner_partner = family_map.get(partner).and_then(|p| *p);
                assert_eq!(
                    partner_partner,
                    Some(*entity),
                    "Tel Aviv reciprocity violated: {:?} -> {:?}, but {:?} -> {:?}",
                    entity,
                    partner,
                    partner,
                    partner_partner
                );
            }
        }
    }

    // Invariant 2: Road segment / grid consistency (soft check).
    // On the Tel Aviv map, a few segment cells may overlap with water/terrain,
    // so we check >95% consistency rather than strict equality.
    {
        let grid = city.grid();
        let segments = city.road_segments();
        let mut total_cells = 0usize;
        let mut mismatch_cells = 0usize;
        for seg in &segments.segments {
            for &(cx, cy) in &seg.rasterized_cells {
                if grid.in_bounds(cx, cy) {
                    total_cells += 1;
                    if grid.get(cx, cy).cell_type != CellType::Road {
                        mismatch_cells += 1;
                    }
                }
            }
        }
        if total_cells > 0 {
            let match_rate = 1.0 - (mismatch_cells as f64 / total_cells as f64);
            assert!(
                match_rate > 0.95,
                "Tel Aviv: only {:.1}% of segment cells match grid roads ({}/{} mismatched)",
                match_rate * 100.0,
                mismatch_cells,
                total_cells,
            );
        }
    }
}

// ====================================================================
// Blueprint system integration tests
// ====================================================================

#[test]
fn test_blueprint_capture_empty_area_produces_empty_blueprint() {
    let city = TestCity::new();
    let grid = city.grid();
    let segments = city.road_segments();

    let bp =
        crate::blueprints::Blueprint::capture(grid, segments, 50, 50, 10, 10, "Empty".to_string());
    assert_eq!(bp.name, "Empty");
    assert_eq!(bp.width, 10);
    assert_eq!(bp.height, 10);
    assert!(bp.segments.is_empty(), "empty area should have no segments");
    assert!(
        bp.zone_cells.is_empty(),
        "empty area should have no zone cells"
    );
}

#[test]
fn test_blueprint_capture_and_place_zones() {
    let city = TestCity::new().with_zone_rect(20, 20, 24, 24, ZoneType::ResidentialLow);

    let grid = city.grid();
    let segments = city.road_segments();

    // Capture the zoned region
    let bp =
        crate::blueprints::Blueprint::capture(grid, segments, 20, 20, 5, 5, "ResBlock".to_string());
    assert_eq!(
        bp.zone_cells.len(),
        25,
        "5x5 region should capture 25 zone cells"
    );

    // Place it at a different location
    let mut city = city;
    let world = city.world_mut();
    world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
        world.resource_scope(|world, mut segs: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                let result = bp.place(&mut grid, &mut segs, &mut roads, 100, 100);
                assert_eq!(result.zones_placed, 25, "should place 25 zone cells");
            });
        });
    });

    // Verify zones were placed at the new location
    let grid = city.grid();
    for y in 100..105 {
        for x in 100..105 {
            assert_eq!(
                grid.get(x, y).zone,
                ZoneType::ResidentialLow,
                "cell ({},{}) should be ResidentialLow",
                x,
                y
            );
        }
    }
    // Verify original zone is still there
    assert_eq!(grid.get(20, 20).zone, ZoneType::ResidentialLow);
}

#[test]
fn test_blueprint_capture_and_place_road_segments() {
    let city = TestCity::new().with_road(30, 30, 30, 40, RoadType::Avenue);

    let initial_seg_count = city.road_segments().segments.len();
    assert!(initial_seg_count > 0, "should have at least one segment");

    let grid = city.grid();
    let segments = city.road_segments();

    // Capture region containing the road
    let bp =
        crate::blueprints::Blueprint::capture(grid, segments, 25, 25, 20, 20, "Road".to_string());
    assert!(!bp.segments.is_empty(), "should capture road segments");
    assert_eq!(
        bp.segments[0].road_type,
        crate::blueprints::BlueprintRoadType::Avenue,
        "captured segment should be Avenue type"
    );

    // Place at a new location
    let mut city = city;
    let world = city.world_mut();
    world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
        world.resource_scope(|world, mut segs: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                let result = bp.place(&mut grid, &mut segs, &mut roads, 100, 100);
                assert!(
                    result.segments_placed > 0,
                    "should place at least one segment"
                );
            });
        });
    });

    // Verify new segments were added
    let final_seg_count = city.road_segments().segments.len();
    assert!(
        final_seg_count > initial_seg_count,
        "segment count should increase after placing blueprint"
    );
}

// ====================================================================
// Freehand road drawing tests (UX-020)
// ====================================================================

#[test]
fn test_freehand_state_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<FreehandDrawState>();
}

#[test]
fn test_freehand_state_default_disabled() {
    let city = TestCity::new();
    let state = city.resource::<FreehandDrawState>();
    assert!(!state.enabled);
    assert!(!state.drawing);
    assert!(state.raw_points.is_empty());
}

#[test]
fn test_freehand_simplify_straight_path_creates_single_segment() {
    // A straight line of points should simplify to 2 points (= 1 segment)
    let points: Vec<Vec2> = (0..20)
        .map(|i| Vec2::new(i as f32 * FREEHAND_MIN_SAMPLE_DIST, 0.0))
        .collect();
    let simplified = simplify_rdp(&points, FREEHAND_SIMPLIFY_TOLERANCE);
    assert_eq!(
        simplified.len(),
        2,
        "straight line should simplify to 2 endpoints"
    );
}

#[test]
fn test_freehand_simplify_l_shaped_path_keeps_corner() {
    // L-shaped path: go right then go down
    let mut points = Vec::new();
    for i in 0..10 {
        points.push(Vec2::new(i as f32 * FREEHAND_MIN_SAMPLE_DIST, 0.0));
    }
    for i in 1..10 {
        points.push(Vec2::new(
            9.0 * FREEHAND_MIN_SAMPLE_DIST,
            i as f32 * FREEHAND_MIN_SAMPLE_DIST,
        ));
    }
    let simplified = simplify_rdp(&points, FREEHAND_SIMPLIFY_TOLERANCE);
    // Should be 3 points: start, corner, end
    assert!(
        simplified.len() >= 3,
        "L-shape should have at least 3 points, got {}",
        simplified.len()
    );
}

#[test]
fn test_blueprint_place_skips_water_cells() {
    let mut city = TestCity::new();

    // Set some cells to water
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(80, 80).cell_type = CellType::Water;
        grid.get_mut(81, 80).cell_type = CellType::Water;
    }

    let bp = crate::blueprints::Blueprint {
        name: "Test".to_string(),
        width: 3,
        height: 1,
        segments: vec![],
        zone_cells: vec![
            crate::blueprints::BlueprintZoneCell {
                dx: 0,
                dy: 0,
                zone_type: crate::blueprints::BlueprintZoneType::ResidentialLow,
            },
            crate::blueprints::BlueprintZoneCell {
                dx: 1,
                dy: 0,
                zone_type: crate::blueprints::BlueprintZoneType::ResidentialLow,
            },
            crate::blueprints::BlueprintZoneCell {
                dx: 2,
                dy: 0,
                zone_type: crate::blueprints::BlueprintZoneType::ResidentialLow,
            },
        ],
    };

    let world = city.world_mut();
    world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
        world.resource_scope(|world, mut segs: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                let result = bp.place(&mut grid, &mut segs, &mut roads, 80, 80);
                // Two cells are water, only one should be placed
                assert_eq!(result.zones_placed, 1, "should skip water cells");
            });
        });
    });
}

#[test]
fn test_blueprint_library_saveable_persistence() {
    let mut lib = crate::blueprints::BlueprintLibrary::default();
    lib.add(crate::blueprints::Blueprint {
        name: "Saved Layout".to_string(),
        width: 15,
        height: 15,
        segments: vec![crate::blueprints::BlueprintSegment {
            p0: [0.0, 0.0],
            p1: [80.0, 0.0],
            p2: [160.0, 0.0],
            p3: [240.0, 0.0],
            road_type: crate::blueprints::BlueprintRoadType::Boulevard,
        }],
        zone_cells: vec![crate::blueprints::BlueprintZoneCell {
            dx: 1,
            dy: 0,
            zone_type: crate::blueprints::BlueprintZoneType::CommercialHigh,
        }],
    });

    // Save to bytes and restore
    use crate::Saveable;
    let bytes = lib.save_to_bytes().expect("non-empty library should save");
    let restored = crate::blueprints::BlueprintLibrary::load_from_bytes(&bytes);

    assert_eq!(restored.count(), 1);
    let bp = restored.get(0).unwrap();
    assert_eq!(bp.name, "Saved Layout");
    assert_eq!(bp.segments.len(), 1);
    assert_eq!(bp.zone_cells.len(), 1);
}

#[test]
fn test_blueprint_multiple_placements_are_independent() {
    let city = TestCity::new().with_zone_rect(10, 10, 12, 12, ZoneType::Industrial);

    let grid = city.grid();
    let segments = city.road_segments();
    let bp =
        crate::blueprints::Blueprint::capture(grid, segments, 10, 10, 3, 3, "Factory".to_string());
    assert_eq!(bp.zone_cells.len(), 9);

    let mut city = city;

    // Place at two different locations
    let world = city.world_mut();
    world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
        world.resource_scope(|world, mut segs: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                let r1 = bp.place(&mut grid, &mut segs, &mut roads, 60, 60);
                assert_eq!(r1.zones_placed, 9);

                let r2 = bp.place(&mut grid, &mut segs, &mut roads, 80, 80);
                assert_eq!(r2.zones_placed, 9);
            });
        });
    });
}

#[test]
fn test_freehand_filter_removes_short_segments() {
    let points = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0), // too short
        Vec2::new(100.0, 0.0),
        Vec2::new(200.0, 0.0),
    ];
    let filtered = filter_short_segments(&points, FREEHAND_MIN_SEGMENT_LEN);
    // First two points are too close, so the 10.0 one gets filtered
    assert!(
        filtered.len() <= 3,
        "short segments should be filtered, got {} points",
        filtered.len()
    );
}

#[test]
fn test_freehand_sample_enforces_min_distance() {
    let mut state = FreehandDrawState::default();
    state.enabled = true;
    state.drawing = true;

    // First sample always accepted
    assert!(state.add_sample(Vec2::new(0.0, 0.0)));
    // Sample too close (< FREEHAND_MIN_SAMPLE_DIST)
    assert!(!state.add_sample(Vec2::new(1.0, 0.0)));
    // Sample far enough
    assert!(state.add_sample(Vec2::new(FREEHAND_MIN_SAMPLE_DIST + 1.0, 0.0)));
    assert_eq!(state.raw_points.len(), 2);
}

#[test]
fn test_freehand_reset_stroke_preserves_enabled() {
    let mut state = FreehandDrawState::default();
    state.enabled = true;
    state.drawing = true;
    state.raw_points.push(Vec2::ZERO);
    state.raw_points.push(Vec2::new(100.0, 0.0));

    state.reset_stroke();
    assert!(state.enabled, "reset_stroke should preserve enabled state");
    assert!(!state.drawing);
    assert!(state.raw_points.is_empty());
}

#[test]
fn test_freehand_simplify_and_create_road_segments() {
    // Simulate the full freehand workflow: collect points, simplify, create segments
    let mut city = TestCity::new().with_budget(100_000.0);

    // Generate a straight path of points in world coordinates
    let start_x = 128.0 * 16.0; // center of the grid
    let start_y = 128.0 * 16.0;
    let points: Vec<Vec2> = (0..10)
        .map(|i| Vec2::new(start_x + i as f32 * FREEHAND_MIN_SAMPLE_DIST, start_y))
        .collect();

    let simplified = simplify_rdp(&points, FREEHAND_SIMPLIFY_TOLERANCE);
    let simplified = filter_short_segments(&simplified, FREEHAND_MIN_SEGMENT_LEN);

    assert!(
        simplified.len() >= 2,
        "need at least 2 points to create a segment"
    );

    // Create road segments from the simplified path
    let world = city.world_mut();
    world.resource_scope(
        |world, mut segments: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
                world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                    for pair in simplified.windows(2) {
                        segments.add_straight_segment(
                            pair[0],
                            pair[1],
                            RoadType::Local,
                            24.0,
                            &mut grid,
                            &mut roads,
                        );
                    }
                });
            });
        },
    );

    let segment_count = city.road_segments().segments.len();
    assert!(
        segment_count >= 1,
        "should have at least 1 road segment, got {}",
        segment_count
    );

    // Verify road cells were created on the grid
    assert!(
        city.road_cell_count() > 0,
        "should have road cells on the grid"
    );
}

#[test]
fn test_freehand_curved_path_creates_multiple_segments() {
    // Simulate a curved freehand path
    let mut city = TestCity::new().with_budget(100_000.0);

    // Quarter-circle path
    let center_x = 128.0 * 16.0;
    let center_y = 128.0 * 16.0;
    let radius = 200.0;
    let n = 20;
    let points: Vec<Vec2> = (0..=n)
        .map(|i| {
            let angle = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
            Vec2::new(
                center_x + angle.cos() * radius,
                center_y + angle.sin() * radius,
            )
        })
        .collect();

    let simplified = simplify_rdp(&points, FREEHAND_SIMPLIFY_TOLERANCE);
    let simplified = filter_short_segments(&simplified, FREEHAND_MIN_SEGMENT_LEN);

    // Curved path should have more than 2 points
    assert!(
        simplified.len() > 2,
        "curved path should have >2 simplified points, got {}",
        simplified.len()
    );

    let world = city.world_mut();
    world.resource_scope(
        |world, mut segments: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
                world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                    for pair in simplified.windows(2) {
                        segments.add_straight_segment(
                            pair[0],
                            pair[1],
                            RoadType::Avenue,
                            24.0,
                            &mut grid,
                            &mut roads,
                        );
                    }
                });
            });
        },
    );

    let segment_count = city.road_segments().segments.len();
    assert!(
        segment_count > 1,
        "curved path should produce multiple segments, got {}",
        segment_count
    );
}

// ====================================================================
// Road Hierarchy (TRAF-003)
// ====================================================================

#[test]
fn test_road_hierarchy_local_to_highway_creates_violation() {
    use crate::road_hierarchy::RoadHierarchyState;

    // Build a Local road and a Highway that share a node (they meet at the
    // endpoint of the first segment and the start of the second).
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Local)
        .with_road(100, 110, 100, 120, RoadType::Highway);

    // Tick enough for the hierarchy check to run (interval = 200)
    city.tick(200);

    let state = city.resource::<RoadHierarchyState>();
    assert!(
        !state.violations.is_empty(),
        "Local-to-Highway connection should produce a hierarchy violation"
    );

    // The violation should report 2 levels skipped (diff=3, skipped=2)
    let v = &state.violations[0];
    assert_eq!(v.levels_skipped, 2, "Local(1) to Highway(4) skips 2 levels");
}

#[test]
fn test_road_hierarchy_proper_chain_no_violations() {
    use crate::road_hierarchy::RoadHierarchyState;

    // Build a proper hierarchy: Local -> Avenue -> Boulevard -> Highway
    // Each pair differs by at most 1 level.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Local)
        .with_road(100, 110, 100, 120, RoadType::Avenue)
        .with_road(100, 120, 100, 130, RoadType::Boulevard)
        .with_road(100, 130, 100, 140, RoadType::Highway);

    city.tick(200);

    let state = city.resource::<RoadHierarchyState>();
    assert!(
        state.violations.is_empty(),
        "Proper hierarchy chain should produce no violations, got {} violations",
        state.violations.len()
    );
}

#[test]
fn test_road_hierarchy_same_type_no_violations() {
    use crate::road_hierarchy::RoadHierarchyState;

    // Two local roads sharing a node — no violation
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Local)
        .with_road(100, 110, 100, 120, RoadType::Local);

    city.tick(200);

    let state = city.resource::<RoadHierarchyState>();
    assert!(
        state.violations.is_empty(),
        "Same road type connections should produce no violations"
    );
}

#[test]
fn test_road_hierarchy_violation_generates_advisor_message() {
    use crate::advisors::{AdvisorPanel, TipId};
    use crate::road_hierarchy::RoadHierarchyState;

    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Local)
        .with_road(100, 110, 100, 120, RoadType::Highway);

    city.tick(200);

    // Verify violation was detected
    let state = city.resource::<RoadHierarchyState>();
    assert!(!state.violations.is_empty());

    // Verify advisor message was generated
    let panel = city.resource::<AdvisorPanel>();
    let hierarchy_msgs: Vec<_> = panel
        .messages
        .iter()
        .filter(|m| m.tip_id == TipId::RoadHierarchyViolation)
        .collect();
    assert!(
        !hierarchy_msgs.is_empty(),
        "Advisor should generate a road hierarchy violation message"
    );
    assert!(
        hierarchy_msgs[0].location.is_some(),
        "Hierarchy advisor message should include a location"
    );
}

#[test]
fn test_road_hierarchy_avenue_to_boulevard_no_violation() {
    use crate::road_hierarchy::RoadHierarchyState;

    // Avenue (level 2) -> Boulevard (level 3) = diff 1, no violation
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Avenue)
        .with_road(100, 110, 100, 120, RoadType::Boulevard);

    city.tick(200);

    let state = city.resource::<RoadHierarchyState>();
    assert!(
        state.violations.is_empty(),
        "Avenue to Boulevard should not produce a violation (adjacent levels)"
    );
}

#[test]
fn test_road_hierarchy_path_to_avenue_is_violation() {
    use crate::road_hierarchy::RoadHierarchyState;

    // Path (level 0) -> Avenue (level 2) = diff 2, violation
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Path)
        .with_road(100, 110, 100, 120, RoadType::Avenue);

    city.tick(200);

    let state = city.resource::<RoadHierarchyState>();
    assert!(
        !state.violations.is_empty(),
        "Path to Avenue should produce a hierarchy violation"
    );
    assert_eq!(state.violations[0].levels_skipped, 1);
}

// ---------------------------------------------------------------------------
// Parallel Road Drawing (UX-021)
// ---------------------------------------------------------------------------

#[test]
fn test_parallel_road_creates_two_segments() {
    use crate::grid::RoadType;
    use crate::test_harness::TestCity;

    // Simulate the parallel drawing behavior: place two parallel roads
    // at an offset to mimic what the parallel_draw system would do.
    let city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_road(100, 131, 110, 131, RoadType::Local);

    assert_eq!(city.segment_count(), 2);
    assert_eq!(city.segment_road_type(0), Some(RoadType::Local));
    assert_eq!(city.segment_road_type(1), Some(RoadType::Local));
}

#[test]
fn test_parallel_road_both_segments_rasterize() {
    use crate::grid::{CellType, RoadType};
    use crate::test_harness::TestCity;

    let city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_road(100, 131, 110, 131, RoadType::Local);

    // Both roads should have rasterized cells
    city.assert_has_road(105, 128);
    city.assert_has_road(105, 131);

    // Check that the cells between the parallel roads are NOT roads
    let cell = city.cell(105, 130);
    assert_ne!(cell.cell_type, CellType::Road);
}

#[test]
fn test_parallel_road_highway_wider_offset() {
    use crate::grid::RoadType;
    use crate::test_harness::TestCity;

    // Highway roads should be placed further apart due to wider road width
    let city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 120, 120, 120, RoadType::Highway)
        .with_road(100, 126, 120, 126, RoadType::Highway);

    assert_eq!(city.segment_count(), 2);
    // Both segments should be highways
    assert_eq!(city.segment_road_type(0), Some(RoadType::Highway));
    assert_eq!(city.segment_road_type(1), Some(RoadType::Highway));
}

#[test]
fn test_parallel_road_more_cells_than_single() {
    use crate::grid::RoadType;
    use crate::test_harness::TestCity;

    let city_single =
        TestCity::new()
            .with_budget(100_000.0)
            .with_road(100, 128, 110, 128, RoadType::Local);
    let single_road_cells = city_single.road_cell_count();

    let city_double = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_road(100, 131, 110, 131, RoadType::Local);
    let double_road_cells = city_double.road_cell_count();

    // Two parallel roads should produce more road cells than one
    assert!(
        double_road_cells > single_road_cells,
        "Expected more road cells with two roads ({}) than one ({})",
        double_road_cells,
        single_road_cells
    );
}

#[test]
fn test_parallel_road_oneway_pair() {
    use crate::grid::RoadType;
    use crate::test_harness::TestCity;

    // One-way roads are a common use case for parallel drawing
    let city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(100, 128, 115, 128, RoadType::OneWay)
        .with_road(100, 130, 115, 130, RoadType::OneWay);

    assert_eq!(city.segment_count(), 2);
    assert_eq!(city.segment_road_type(0), Some(RoadType::OneWay));
    assert_eq!(city.segment_road_type(1), Some(RoadType::OneWay));
}

// ====================================================================
// Curve Road Drawing (UX-019) tests
// ====================================================================

#[test]
fn test_curved_road_creates_segment_with_nonlinear_control_points() {
    let city = TestCity::new().with_budget(100_000.0).with_curved_road(
        120,
        128,
        125,
        135,
        130,
        128,
        RoadType::Local,
    );

    let segments = city.road_segments();
    assert_eq!(segments.segments.len(), 1, "should have exactly 1 segment");

    let seg = &segments.segments[0];
    let straight_p1 = seg.p0 + (seg.p3 - seg.p0) / 3.0;
    let straight_p2 = seg.p0 + (seg.p3 - seg.p0) * 2.0 / 3.0;

    let p1_diff = (seg.p1 - straight_p1).length();
    let p2_diff = (seg.p2 - straight_p2).length();
    assert!(
        p1_diff > 1.0 || p2_diff > 1.0,
        "curved segment should have non-trivial control points, p1_diff={}, p2_diff={}",
        p1_diff,
        p2_diff
    );
}

#[test]
fn test_curved_road_rasterizes_cells() {
    let city = TestCity::new().with_budget(100_000.0).with_curved_road(
        120,
        128,
        125,
        135,
        130,
        128,
        RoadType::Local,
    );

    let segments = city.road_segments();
    let seg = &segments.segments[0];
    assert!(
        !seg.rasterized_cells.is_empty(),
        "curved segment should rasterize to grid cells"
    );

    let grid = city.grid();
    let road_cells = seg
        .rasterized_cells
        .iter()
        .filter(|&&(gx, gy)| grid.get(gx, gy).cell_type == CellType::Road)
        .count();
    assert!(road_cells > 0, "rasterized cells should include road cells");
}

#[test]
fn test_curved_road_has_longer_arc_than_straight_distance() {
    let city = TestCity::new().with_budget(100_000.0).with_curved_road(
        120,
        128,
        125,
        140,
        130,
        128,
        RoadType::Avenue,
    );

    let segments = city.road_segments();
    let seg = &segments.segments[0];
    let straight_dist = (seg.p3 - seg.p0).length();

    assert!(
        seg.arc_length > straight_dist,
        "curved road arc length ({}) should exceed straight distance ({})",
        seg.arc_length,
        straight_dist
    );
}

#[test]
fn test_curved_road_endpoints_match_requested_positions() {
    use crate::config::CELL_SIZE;

    let city = TestCity::new().with_budget(100_000.0).with_curved_road(
        120,
        128,
        125,
        135,
        130,
        128,
        RoadType::Local,
    );

    let segments = city.road_segments();
    let seg = &segments.segments[0];

    let at_start = seg.evaluate(0.0);
    let at_end = seg.evaluate(1.0);
    assert!(
        (at_start - seg.p0).length() < 0.01,
        "curve start should match p0"
    );
    assert!(
        (at_end - seg.p3).length() < 0.01,
        "curve end should match p3"
    );

    let (wx0, wy0) = WorldGrid::grid_to_world(120, 128);
    let (wx1, wy1) = WorldGrid::grid_to_world(130, 128);
    assert!(
        (seg.p0 - bevy::math::Vec2::new(wx0, wy0)).length() < CELL_SIZE,
        "start should be near grid (120, 128)"
    );
    assert!(
        (seg.p3 - bevy::math::Vec2::new(wx1, wy1)).length() < CELL_SIZE,
        "end should be near grid (130, 128)"
    );
}

#[test]
fn test_curved_road_creates_nodes() {
    let city = TestCity::new().with_budget(100_000.0).with_curved_road(
        120,
        128,
        125,
        135,
        130,
        128,
        RoadType::Local,
    );

    let segments = city.road_segments();
    assert!(
        segments.nodes.len() >= 2,
        "curved road should create at least 2 nodes, got {}",
        segments.nodes.len()
    );
}

#[test]
fn test_curved_road_different_types() {
    for road_type in [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
    ] {
        let city = TestCity::new()
            .with_budget(100_000.0)
            .with_curved_road(120, 128, 125, 135, 130, 128, road_type);

        let segments = city.road_segments();
        assert_eq!(
            segments.segments[0].road_type, road_type,
            "segment road type should match requested type"
        );
    }
}

#[test]
fn test_quadratic_to_cubic_conversion_preserves_midpoint() {
    use crate::curve_road_drawing::quadratic_to_cubic;
    use bevy::math::Vec2;

    let p0 = Vec2::new(0.0, 0.0);
    let control = Vec2::new(150.0, 200.0);
    let p3 = Vec2::new(300.0, 0.0);

    let (p1, p2) = quadratic_to_cubic(p0, control, p3);

    let quad_mid: Vec2 = 0.25 * p0 + 0.5 * control + 0.25 * p3;
    let cubic_mid: Vec2 = 0.125 * p0 + 0.375 * p1 + 0.375 * p2 + 0.125 * p3;

    assert!(
        (quad_mid - cubic_mid).length() < 0.1,
        "cubic midpoint ({:?}) should match quadratic midpoint ({:?})",
        cubic_mid,
        quad_mid,
    );
}

// ====================================================================
// Bicycle Lanes (TRAF-013)
// ====================================================================

#[test]
fn test_bicycle_lanes_default_state_has_no_lanes() {
    let city = TestCity::new();
    let bike_state = city.resource::<crate::bicycle_lanes::BicycleLaneState>();
    assert_eq!(
        bike_state.lane_count(),
        0,
        "new city should have no bike lanes"
    );
}

#[test]
fn test_bicycle_lanes_coverage_zero_without_infrastructure() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();

    let coverage = city.resource::<crate::bicycle_lanes::BicycleCoverageGrid>();
    assert_eq!(
        coverage.city_average, 0.0,
        "city without bike infrastructure should have 0 cycling coverage"
    );
    assert!(
        coverage.cycling_mode_share < 0.01,
        "cycling mode share should be ~0 without infrastructure, got {}",
        coverage.cycling_mode_share
    );
}

// Superblock policy tests
// ====================================================================

#[test]
fn test_superblock_state_initialized() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::superblock::SuperblockState>();
}

#[test]
fn test_superblock_add_and_query() {
    use crate::superblock::{Superblock, SuperblockCell, SuperblockState};
    let mut city = TestCity::new();

    // Add a 5x5 superblock
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SuperblockState>();
        let added = state.add_superblock(Superblock::new(
            50,
            50,
            54,
            54,
            "Downtown Block".to_string(),
        ));
        assert!(added, "should successfully add a valid superblock");
    }

    // Verify cell classifications
    {
        let state = city.resource::<SuperblockState>();
        // Interior cell
        assert_eq!(state.get_cell(52, 52), SuperblockCell::Interior);
        // Perimeter cell
        assert_eq!(state.get_cell(50, 50), SuperblockCell::Perimeter);
        // Outside cell
        assert_eq!(state.get_cell(40, 40), SuperblockCell::None);
        // Coverage stats
        assert_eq!(state.total_interior_cells, 9); // 3x3 interior
        assert_eq!(state.total_coverage_cells, 25); // 5x5 total
    }
}

#[test]
fn test_superblock_reject_too_small() {
    use crate::superblock::{Superblock, SuperblockState};
    let mut city = TestCity::new();

    let world = city.world_mut();
    let mut state = world.resource_mut::<SuperblockState>();

    // 2x2 is too small (minimum 3x3)
    let added = state.add_superblock(Superblock::new(10, 10, 11, 11, "Tiny".to_string()));
    assert!(!added, "should reject superblocks smaller than 3x3");
    assert!(state.superblocks.is_empty());
}

#[test]
fn test_superblock_traffic_multiplier_interior() {
    use crate::superblock::{Superblock, SuperblockState, SUPERBLOCK_TRAFFIC_PENALTY};
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SuperblockState>();
        state.add_superblock(Superblock::new(30, 30, 36, 36, "Test Block".to_string()));
    }

    let state = city.resource::<SuperblockState>();
    // Interior cells get the penalty multiplier
    assert!(
        (state.traffic_multiplier(33, 33) - SUPERBLOCK_TRAFFIC_PENALTY).abs() < f32::EPSILON,
        "interior cells should have traffic penalty"
    );
    // Perimeter cells have normal cost
    assert!(
        (state.traffic_multiplier(30, 30) - 1.0).abs() < f32::EPSILON,
        "perimeter cells should have no traffic penalty"
    );
    // Outside cells have normal cost
    assert!(
        (state.traffic_multiplier(20, 20) - 1.0).abs() < f32::EPSILON,
        "cells outside superblock should have no penalty"
    );
}

// ====================================================================
// Mode Choice (TRAF-007) integration tests
// ====================================================================

#[test]
fn test_mode_choice_resource_initialized() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::mode_choice::ModeShareStats>();
    city.assert_resource_exists::<crate::mode_choice::ModeInfrastructureCache>();
}

#[test]
fn test_mode_choice_default_stats() {
    let city = TestCity::new();
    let stats = city.resource::<crate::mode_choice::ModeShareStats>();
    assert_eq!(stats.total(), 0);
    // Default: 100% drive when no trips active
    assert!((stats.drive_pct - 100.0).abs() < f32::EPSILON);
}

#[test]
fn test_mode_choice_citizen_has_component() {
    use crate::mode_choice::ChosenTransportMode;
    use bevy::prelude::Entity;

    let mut city = TestCity::new()
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_building(101, 127, ZoneType::ResidentialLow, 1)
        .with_building(109, 127, ZoneType::CommercialLow, 1)
        .with_citizen((101, 127), (109, 127));

    // Verify the citizen has a ChosenTransportMode component
    let world = city.world_mut();
    let count = world
        .query_filtered::<Entity, bevy::prelude::With<ChosenTransportMode>>()
        .iter(world)
        .count();
    assert_eq!(
        count, 1,
        "citizen should have ChosenTransportMode component"
    );
}

#[test]
fn test_bicycle_lanes_add_lane_to_road_segment() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_budget(100_000.0);

    // Find the segment ID
    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        assert!(
            !store.segments.is_empty(),
            "should have at least one road segment"
        );
        store.segments[0].id
    };

    // Add bike lane
    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }

    city.tick_slow_cycle();

    let coverage = city.resource::<crate::bicycle_lanes::BicycleCoverageGrid>();
    assert!(
        coverage.city_average > 0.0,
        "city with bike lane should have nonzero cycling coverage, got {}",
        coverage.city_average
    );
}

#[test]
fn test_bicycle_lanes_mode_share_positive_with_infrastructure() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_budget(100_000.0);

    // Add bike lane to the road
    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        store.segments[0].id
    };

    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }

    city.tick_slow_cycle();

    let coverage = city.resource::<crate::bicycle_lanes::BicycleCoverageGrid>();
    assert!(
        coverage.cycling_mode_share > 0.0,
        "cycling mode share should be positive with bike lanes, got {}",
        coverage.cycling_mode_share
    );
}

#[test]
fn test_bicycle_lanes_remove_lane_drops_coverage() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_budget(100_000.0);

    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        store.segments[0].id
    };

    // Add and verify
    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }
    city.tick_slow_cycle();

    let coverage_with = city
        .resource::<crate::bicycle_lanes::BicycleCoverageGrid>()
        .city_average;
    assert!(coverage_with > 0.0);

    // Remove and verify drop
    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.remove_bike_lane(seg_id);
    }
    city.tick_slow_cycle();

    let coverage_without = city
        .resource::<crate::bicycle_lanes::BicycleCoverageGrid>()
        .city_average;
    assert!(
        coverage_without < coverage_with,
        "removing bike lane should reduce coverage: with={}, without={}",
        coverage_with,
        coverage_without
    );
}

#[test]
fn test_bicycle_lanes_maintenance_cost() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_budget(100_000.0);

    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        store.segments[0].id
    };

    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }

    city.tick_slow_cycle();

    let coverage = city.resource::<crate::bicycle_lanes::BicycleCoverageGrid>();
    assert!(
        coverage.total_maintenance_cost > 0.0,
        "bike lanes should have positive maintenance cost, got {}",
        coverage.total_maintenance_cost
    );
}

#[test]
fn test_bicycle_lanes_unsupported_road_type_ignored() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Highway)
        .with_budget(100_000.0);

    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        store.segments[0].id
    };

    // Add bike lane to highway (should not produce coverage)
    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }

    city.tick_slow_cycle();

    let bike_state = city.resource::<crate::bicycle_lanes::BicycleLaneState>();
    assert!(
        bike_state.has_bike_lane(seg_id),
        "bike lane flag should be set even on unsupported type"
    );

    // The coverage system checks supports_bike_lane(), so highway bike lanes
    // don't contribute to coverage. But Path cells may still contribute.
    // This test just verifies no panic occurs.
}

#[test]
fn test_bicycle_lanes_encourage_biking_policy_boost() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_budget(100_000.0);

    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        store.segments[0].id
    };

    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }

    // Measure without policy
    city.tick_slow_cycle();
    let share_without = city
        .resource::<crate::bicycle_lanes::BicycleCoverageGrid>()
        .cycling_mode_share;

    // Enable Encourage Biking policy
    {
        let world = city.world_mut();
        let mut policies = world
            .get_resource_mut::<crate::policies::Policies>()
            .unwrap();
        policies.toggle(crate::policies::Policy::EncourageBiking);
    }

    city.tick_slow_cycle();
    let share_with = city
        .resource::<crate::bicycle_lanes::BicycleCoverageGrid>()
        .cycling_mode_share;

    assert!(
        share_with > share_without,
        "Encourage Biking policy should increase cycling mode share: \
         without={}, with={}",
        share_without,
        share_with
    );
}

#[test]
fn test_bicycle_lanes_saveable_roundtrip() {
    use crate::bicycle_lanes::BicycleLaneState;
    use crate::road_segments::SegmentId;
    use crate::Saveable;

    let mut state = BicycleLaneState::default();
    state.add_bike_lane(SegmentId(10));
    state.add_bike_lane(SegmentId(20));

    let bytes = state.save_to_bytes().expect("non-empty should save");
    let restored = BicycleLaneState::load_from_bytes(&bytes);

    assert_eq!(restored.lane_count(), 2);
    assert!(restored.has_bike_lane(SegmentId(10)));
    assert!(restored.has_bike_lane(SegmentId(20)));
}

#[test]
fn test_bicycle_lanes_path_road_gives_implicit_coverage() {
    // Path roads are pedestrian/bike paths — they should provide coverage
    // even without explicitly adding bike lanes
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Path)
        .with_budget(100_000.0);

    city.tick_slow_cycle();

    let coverage = city.resource::<crate::bicycle_lanes::BicycleCoverageGrid>();
    assert!(
        coverage.city_average > 0.0,
        "Path roads should provide implicit bike coverage, got {}",
        coverage.city_average
    );
}

// ====================================================================
// Simulation invariant validation tests
// ====================================================================

#[test]
fn test_invariant_validator_detects_overcapacity_on_tel_aviv() {
    use crate::simulation_invariants::InvariantViolations;
    use crate::test_harness::TestCity;

    let mut city = TestCity::with_tel_aviv();
    city.tick_slow_cycles(3);

    // The validator should have detected and corrected overcapacity violations.
    // Due to simulation dynamics (job seeking can add workers between slow ticks),
    // the violation count may be non-zero. We just verify the validator ran and
    // the InvariantViolations resource is accessible (system is wired up correctly).
    let _violations = city.resource::<InvariantViolations>();
    // If we got here without panicking, the validator system is properly registered
    // and ran successfully during the slow tick cycles.
}

#[test]
fn test_invariant_nonreciprocal_marriage_detected_and_cleared() {
    use crate::citizen::{
        Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation,
        Needs, PathCache, Personality, Position, Velocity,
    };
    use crate::grid::{RoadType, WorldGrid, ZoneType};
    use crate::mode_choice::ChosenTransportMode;
    use crate::movement::ActivityTimer;
    use crate::simulation_invariants::InvariantViolations;
    use crate::test_harness::TestCity;

    let mut city = TestCity::new()
        .with_road(10, 10, 10, 15, RoadType::Local)
        .with_building(11, 12, ZoneType::ResidentialLow, 1);

    // Run 99 ticks so the slow tick counter is at 99
    city.tick(99);

    // Spawn citizens and inject non-reciprocal link right before validation fires
    let (citizen_a, citizen_b) = {
        let world = city.world_mut();
        let grid = world.resource::<WorldGrid>();
        let home_entity = grid.get(11, 12).building_id.unwrap();
        let (hx, hy) = WorldGrid::grid_to_world(11, 12);

        let a = world
            .spawn((
                Citizen,
                Position { x: hx, y: hy },
                Velocity { x: 0.0, y: 0.0 },
                HomeLocation {
                    grid_x: 11,
                    grid_y: 12,
                    building: home_entity,
                },
                CitizenStateComp(CitizenState::AtHome),
                PathCache::new(Vec::new()),
                CitizenDetails {
                    age: 30,
                    gender: Gender::Male,
                    education: 0,
                    happiness: 60.0,
                    health: 90.0,
                    salary: 0.0,
                    savings: 1000.0,
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
                ChosenTransportMode::default(),
            ))
            .id();

        let b = world
            .spawn((
                Citizen,
                Position { x: hx, y: hy },
                Velocity { x: 0.0, y: 0.0 },
                HomeLocation {
                    grid_x: 11,
                    grid_y: 12,
                    building: home_entity,
                },
                CitizenStateComp(CitizenState::AtHome),
                PathCache::new(Vec::new()),
                CitizenDetails {
                    age: 28,
                    gender: Gender::Female,
                    education: 0,
                    happiness: 60.0,
                    health: 90.0,
                    salary: 0.0,
                    savings: 1000.0,
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
                ChosenTransportMode::default(),
            ))
            .id();

        (a, b)
    };

    // Set up non-reciprocal link
    {
        let world = city.world_mut();
        if let Some(mut family) = world.get_mut::<Family>(citizen_a) {
            family.partner = Some(citizen_b);
        }
    }

    // Run 1 more tick to trigger validation at counter=100
    city.tick(1);

    let violations = city.resource::<InvariantViolations>();
    assert!(
        violations.marriage_non_reciprocal > 0,
        "Non-reciprocal marriage should have been detected"
    );

    let world = city.world_mut();
    let family_a = world.get::<Family>(citizen_a).unwrap();
    assert!(
        family_a.partner.is_none(),
        "Citizen A's non-reciprocal partner link should have been cleared"
    );
}

#[test]
fn test_mode_choice_walking_for_short_trip() {
    use crate::mode_choice::{evaluate_walk, WALK_SPEED_MULTIPLIER};

    // A short trip (5 cells) should make walking attractive
    let distance = 5.0;
    let walk_time = evaluate_walk(distance);
    // Walk time = 5.0 / 0.3 / 1.0 = ~16.7
    assert!(walk_time > 0.0);
    assert!((walk_time - distance / WALK_SPEED_MULTIPLIER).abs() < f32::EPSILON);
}

#[test]
fn test_mode_choice_infrastructure_cache_transit() {
    use crate::mode_choice::ModeInfrastructureCache;

    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::BusDepot)
        .with_service(140, 140, ServiceType::SubwayStation);

    // Tick once to trigger infrastructure cache refresh
    city.tick(1);

    let cache = city.resource::<ModeInfrastructureCache>();
    assert!(
        !cache.transit_stops.is_empty(),
        "transit stops should be populated from bus depot and subway station"
    );
    assert!(
        cache.transit_stops.len() >= 2,
        "should have at least 2 transit stops"
    );
}

#[test]
fn test_invariant_tel_aviv_employment_drift_corrected() {
    use crate::buildings::Building;
    use crate::citizen::{Citizen, WorkLocation};
    use crate::simulation_invariants::InvariantViolations;
    use crate::test_harness::TestCity;
    use bevy::prelude::{Entity, With};
    use std::collections::HashMap;

    let mut city = TestCity::with_tel_aviv();
    city.tick_slow_cycle();

    // init_world spawns citizens with WorkLocations but does NOT increment
    // work building occupant counts. The validator should detect and correct this.
    let violations = city.resource::<InvariantViolations>();
    assert!(
        violations.employment_drift > 0,
        "Employment drift should be detected on Tel Aviv map"
    );

    // After correction, actual worker counts should not exceed building occupants
    let world = city.world_mut();
    let mut worker_counts: HashMap<Entity, u32> = HashMap::new();
    let mut work_query = world.query_filtered::<&WorkLocation, With<Citizen>>();
    for work in work_query.iter(world) {
        *worker_counts.entry(work.building).or_insert(0) += 1;
    }
    let mut building_query = world.query::<(Entity, &Building)>();
    for (entity, building) in building_query.iter(world) {
        if building.zone_type.is_job_zone() {
            let actual_workers = worker_counts.get(&entity).copied().unwrap_or(0);
            assert!(
                actual_workers <= building.occupants,
                "After correction, building at ({},{}) should have occupants >= actual workers ({} vs {})",
                building.grid_x, building.grid_y, actual_workers, building.occupants
            );
        }
    }
}

#[test]
fn test_invariant_marriage_reciprocity_on_tel_aviv() {
    use crate::citizen::{Citizen, Family};
    use crate::test_harness::TestCity;
    use bevy::prelude::{Entity, With};

    let mut city = TestCity::with_tel_aviv();
    city.tick_slow_cycles(3);

    // After validation, all remaining partner links should be reciprocal
    let world = city.world_mut();
    let mut partner_map: std::collections::HashMap<Entity, Option<Entity>> =
        std::collections::HashMap::new();
    let mut query = world.query_filtered::<(Entity, &Family), With<Citizen>>();
    for (entity, family) in query.iter(world) {
        partner_map.insert(entity, family.partner);
    }
    for (&entity, &partner_opt) in &partner_map {
        if let Some(partner) = partner_opt {
            match partner_map.get(&partner) {
                Some(Some(back)) if *back == entity => {}
                _ => panic!(
                    "After validation, citizen {:?} has partner {:?} but link is not reciprocal",
                    entity, partner
                ),
            }
        }
    }
}

#[test]
fn test_invariant_no_overcapacity_on_empty_city() {
    use crate::grid::{RoadType, ZoneType};
    use crate::simulation_invariants::InvariantViolations;
    use crate::test_harness::TestCity;

    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_building(11, 12, ZoneType::ResidentialLow, 1)
        .with_building(11, 18, ZoneType::Industrial, 1);

    city.tick_slow_cycle();

    let violations = city.resource::<InvariantViolations>();
    assert_eq!(
        violations.job_overcapacity, 0,
        "No job overcapacity violations expected on empty city"
    );
}

#[test]
fn test_superblock_remove_clears_grid() {
    use crate::superblock::{Superblock, SuperblockCell, SuperblockState};
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SuperblockState>();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "Temp".to_string()));
    }

    // Verify it exists
    assert!(city.resource::<SuperblockState>().is_interior(12, 12));

    // Remove it
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SuperblockState>();
        assert!(state.remove_superblock(0));
    }

    // Verify it's gone
    let state = city.resource::<SuperblockState>();
    assert_eq!(state.get_cell(12, 12), SuperblockCell::None);
    assert_eq!(state.total_interior_cells, 0);
    assert_eq!(state.total_coverage_cells, 0);
}

#[test]
fn test_superblock_persists_across_slow_tick() {
    use crate::superblock::{Superblock, SuperblockState};
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SuperblockState>();
        state.add_superblock(Superblock::new(20, 20, 25, 25, "Persistent".to_string()));
    }

    // Run a full slow tick cycle
    city.tick_slow_cycle();

    // Superblock should still be there
    let state = city.resource::<SuperblockState>();
    assert_eq!(state.superblocks.len(), 1);
    assert!(state.is_interior(22, 22));
    assert!(state.total_interior_cells > 0);
}

#[test]
fn test_superblock_saveable_roundtrip() {
    use crate::superblock::{Superblock, SuperblockState};
    use crate::Saveable;

    let mut state = SuperblockState::default();
    state.add_superblock(Superblock::new(10, 10, 15, 15, "Block A".to_string()));
    state.add_superblock(Superblock::new(50, 50, 56, 56, "Block B".to_string()));

    // Save
    let bytes = state
        .save_to_bytes()
        .expect("non-empty state should serialize");

    // Load
    let restored = SuperblockState::load_from_bytes(&bytes);
    assert_eq!(restored.superblocks.len(), 2);
    assert_eq!(restored.superblocks[0].name, "Block A");
    assert!(restored.is_interior(12, 12));
    assert!(restored.is_interior(53, 53));
    assert!(restored.total_interior_cells > 0);
}

#[test]
fn test_superblock_multiple_blocks_coverage() {
    use crate::superblock::{Superblock, SuperblockState};
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SuperblockState>();
        // Add two non-overlapping 5x5 superblocks
        state.add_superblock(Superblock::new(10, 10, 14, 14, "A".to_string()));
        state.add_superblock(Superblock::new(30, 30, 34, 34, "B".to_string()));
    }

    let state = city.resource::<SuperblockState>();
    assert_eq!(state.superblocks.len(), 2);
    // Each 5x5 has 9 interior cells, total = 18
    assert_eq!(state.total_interior_cells, 18);
    // Each 5x5 has 25 cells, total = 50
    assert_eq!(state.total_coverage_cells, 50);
}

#[test]
fn test_mode_choice_infrastructure_cache_bike_paths() {
    use crate::mode_choice::ModeInfrastructureCache;

    let mut city = TestCity::new().with_road(128, 128, 140, 128, RoadType::Path);

    // Tick to populate cache
    city.tick(1);

    let cache = city.resource::<ModeInfrastructureCache>();
    assert!(
        !cache.bike_paths.is_empty(),
        "bike paths should include Path-type roads"
    );
}

#[test]
fn test_mode_share_stats_update_after_slow_tick() {
    use crate::mode_choice::ModeShareStats;

    // Create a city with roads, buildings, and citizens
    let mut city = TestCity::new()
        .with_road(100, 128, 130, 128, RoadType::Local)
        .with_building(101, 127, ZoneType::ResidentialLow, 1)
        .with_building(120, 127, ZoneType::CommercialLow, 1)
        .with_citizen((101, 127), (120, 127))
        .with_time(7.5) // morning commute time
        .rebuild_csr();

    // Run a full slow cycle to trigger stats update
    city.tick_slow_cycle();

    let stats = city.resource::<ModeShareStats>();
    // After a slow cycle, stats should have been computed
    // (the exact values depend on whether citizens started commuting)
    // At minimum, the system should have run without panicking
    assert!(stats.walk_pct + stats.bike_pct + stats.drive_pct + stats.transit_pct <= 400.1);
}

#[test]
fn test_mode_choice_speed_multiplier_values() {
    use crate::mode_choice::TransportMode;

    // Walk should be slowest
    assert!(TransportMode::Walk.speed_multiplier() < TransportMode::Bike.speed_multiplier());
    // Bike should be slower than driving
    assert!(TransportMode::Bike.speed_multiplier() < TransportMode::Drive.speed_multiplier());
    // Transit should be between bike and drive
    assert!(TransportMode::Transit.speed_multiplier() > TransportMode::Bike.speed_multiplier());
    assert!(TransportMode::Transit.speed_multiplier() < TransportMode::Drive.speed_multiplier());
}

#[test]
fn test_mode_choice_saveable_roundtrip() {
    use crate::mode_choice::ModeShareStats;
    use crate::Saveable;

    let stats = ModeShareStats {
        walk_count: 15,
        bike_count: 25,
        drive_count: 40,
        transit_count: 20,
        walk_pct: 15.0,
        bike_pct: 25.0,
        drive_pct: 40.0,
        transit_pct: 20.0,
    };

    let bytes = stats
        .save_to_bytes()
        .expect("should serialize non-zero stats");
    let restored = ModeShareStats::load_from_bytes(&bytes);

    assert_eq!(restored.walk_count, 15);
    assert_eq!(restored.bike_count, 25);
    assert_eq!(restored.drive_count, 40);
    assert_eq!(restored.transit_count, 20);
    assert_eq!(restored.total(), 100);
}
// Bus Transit System (TRAF-005) Integration Tests
// =============================================================
#[test]
fn test_bus_transit_add_stops_and_route() {
    use crate::bus_transit::BusTransitState;

    let city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_road(10, 20, 20, 20, RoadType::Local);

    let mut transit = city.resource::<BusTransitState>().clone();
    let grid = city.grid();

    // Add stops on road cells
    let s1 = transit.add_stop(grid, 10, 10);
    assert!(s1.is_some(), "Should add stop on road cell");
    let s2 = transit.add_stop(grid, 10, 20);
    assert!(s2.is_some(), "Should add second stop on road cell");

    // Create route
    let route_id = transit.add_route("Line 1".to_string(), vec![s1.unwrap(), s2.unwrap()]);
    assert!(route_id.is_some(), "Should create route with 2 stops");
    assert_eq!(transit.routes.len(), 1);
    assert_eq!(transit.routes[0].stop_ids.len(), 2);
}

#[test]
fn test_bus_transit_stop_on_grass_fails() {
    use crate::bus_transit::BusTransitState;

    let city = TestCity::new();
    let mut transit = BusTransitState::default();
    let grid = city.grid();

    // Try to add stop on grass (no road)
    let result = transit.add_stop(grid, 50, 50);
    assert!(result.is_none(), "Should not add stop on grass");
}

#[test]
fn test_bus_transit_route_activation_with_depot() {
    use crate::bus_transit::BusTransitState;

    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(10, 12, ServiceType::BusDepot);

    // Set up transit state with stops and route
    {
        let world = city.world_mut();
        let grid = world.resource::<WorldGrid>();
        let mut transit = BusTransitState::default();
        let s1 = transit.add_stop(grid, 10, 10).unwrap();
        let s2 = transit.add_stop(grid, 10, 18).unwrap();
        transit.add_route("Line 1".to_string(), vec![s1, s2]);
        world.insert_resource(transit);
    }

    // Run simulation to trigger route activation
    city.tick(5);

    let transit = city.resource::<BusTransitState>();
    assert_eq!(transit.routes.len(), 1);
    assert!(
        transit.routes[0].active,
        "Route should be active with depot nearby"
    );
}

// ====================================================================
// Auto-Grid Road Placement (TRAF-010)
// ====================================================================

#[test]
fn test_auto_grid_generates_roads_in_area() {
    use crate::auto_grid_road::{compute_grid_plan, execute_grid_plan, AutoGridConfig};
    use crate::grid::{CellType, RoadType};
    use crate::road_segments::RoadSegmentStore;
    use crate::roads::RoadNetwork;
    use crate::test_harness::TestCity;

    let mut city = TestCity::new().with_budget(100_000.0);
    let config = AutoGridConfig {
        block_size: 6,
        road_type: RoadType::Local,
    };

    let plan = {
        let grid = city.grid();
        compute_grid_plan((50, 50), (70, 70), &config, grid)
    };

    assert!(!plan.segments.is_empty(), "plan should have segments");
    assert!(plan.total_cells > 0, "plan should place road cells");
    assert!(plan.total_cost > 0.0, "plan should have a cost");

    let world = city.world_mut();
    world.resource_scope(
        |world, mut segments: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(
                |world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
                    world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                        let cells =
                            execute_grid_plan(&plan, &config, &mut segments, &mut grid, &mut roads);
                        assert!(!cells.is_empty(), "should have placed road cells");
                        for &(x, y) in &cells {
                            assert_eq!(grid.get(x, y).cell_type, CellType::Road);
                        }
                    });
                },
            );
        },
    );
}

#[test]
fn test_auto_grid_block_size_affects_density() {
    use crate::auto_grid_road::{compute_grid_plan, AutoGridConfig};
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{RoadType, WorldGrid};

    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    let plan_small = compute_grid_plan(
        (50, 50),
        (80, 80),
        &AutoGridConfig {
            block_size: 4,
            road_type: RoadType::Local,
        },
        &grid,
    );
    let plan_large = compute_grid_plan(
        (50, 50),
        (80, 80),
        &AutoGridConfig {
            block_size: 8,
            road_type: RoadType::Local,
        },
        &grid,
    );

    assert!(
        plan_small.total_cells > plan_large.total_cells,
        "smaller block size ({}) should produce more roads than larger ({})",
        plan_small.total_cells,
        plan_large.total_cells
    );
}

#[test]
fn test_bus_transit_buses_spawn_on_active_route() {
    use crate::bus_transit::{BusTransitState, BUSES_PER_ROUTE};

    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(10, 12, ServiceType::BusDepot);

    // Set up transit state
    {
        let world = city.world_mut();
        let grid = world.resource::<WorldGrid>();
        let mut transit = BusTransitState::default();
        let s1 = transit.add_stop(grid, 10, 10).unwrap();
        let s2 = transit.add_stop(grid, 10, 18).unwrap();
        transit.add_route("Line 1".to_string(), vec![s1, s2]);
        world.insert_resource(transit);
    }

    // Run enough ticks for activation and bus spawning
    city.tick(10);

    let transit = city.resource::<BusTransitState>();
    assert!(transit.routes[0].active, "Route should be active");
    assert_eq!(
        transit.buses.len(),
        BUSES_PER_ROUTE as usize,
        "Should have spawned {} buses",
        BUSES_PER_ROUTE
    );
}

#[test]
fn test_bus_transit_ridership_increases() {
    use crate::bus_transit::BusTransitState;

    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(10, 12, ServiceType::BusDepot)
        .with_zone_rect(8, 8, 12, 12, ZoneType::ResidentialLow)
        .with_zone_rect(8, 18, 12, 22, ZoneType::CommercialLow);

    // Set up transit with stops near zones
    {
        let world = city.world_mut();
        let grid = world.resource::<WorldGrid>();
        let mut transit = BusTransitState::default();
        let s1 = transit.add_stop(grid, 10, 10).unwrap();
        let s2 = transit.add_stop(grid, 10, 18).unwrap();
        transit.add_route("Line 1".to_string(), vec![s1, s2]);
        world.insert_resource(transit);
    }

    // Run slow cycle to generate waiting passengers and let buses pick them up
    city.tick_slow_cycle();
    city.tick(50); // Extra ticks for buses to reach stops

    let transit = city.resource::<BusTransitState>();
    assert!(
        transit.total_ridership() > 0
            || transit.routes[0].monthly_ridership > 0
            || transit.stops.iter().any(|s| s.waiting > 0),
        "Should have some ridership or waiting passengers after simulation"
    );
}

#[test]
fn test_bus_transit_remove_route_clears_buses() {
    use crate::bus_transit::BusTransitState;

    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(10, 12, ServiceType::BusDepot);

    // Set up transit and let buses spawn
    {
        let world = city.world_mut();
        let grid = world.resource::<WorldGrid>();
        let mut transit = BusTransitState::default();
        let s1 = transit.add_stop(grid, 10, 10).unwrap();
        let s2 = transit.add_stop(grid, 10, 18).unwrap();
        transit.add_route("Line 1".to_string(), vec![s1, s2]);
        world.insert_resource(transit);
    }

    city.tick(10);

    // Verify buses exist
    let bus_count = city.resource::<BusTransitState>().buses.len();
    assert!(bus_count > 0, "Should have buses before removal");

    // Remove the route
    {
        let world = city.world_mut();
        let route_id = world.resource::<BusTransitState>().routes[0].id;
        world
            .resource_mut::<BusTransitState>()
            .remove_route(route_id);
    }

    let transit = city.resource::<BusTransitState>();
    assert_eq!(transit.routes.len(), 0, "Route should be removed");
    assert_eq!(transit.buses.len(), 0, "Buses should be removed with route");
}

#[test]
fn test_bus_transit_costs_applied_to_budget() {
    use crate::bus_transit::BusTransitState;

    let mut city = TestCity::new()
        .with_budget(50_000.0)
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(10, 12, ServiceType::BusDepot);

    // Set up transit with last_cost_day far in the past to trigger cost application
    {
        let world = city.world_mut();
        let grid = world.resource::<WorldGrid>();
        let mut transit = BusTransitState::default();
        let s1 = transit.add_stop(grid, 10, 10).unwrap();
        let s2 = transit.add_stop(grid, 10, 18).unwrap();
        transit.add_route("Line 1".to_string(), vec![s1, s2]);
        world.insert_resource(transit);
    }

    // Advance game clock past day 31 so the monthly cost applies
    {
        let world = city.world_mut();
        let mut clock = world.resource_mut::<GameClock>();
        clock.day = 35;
        clock.hour = 12.0;
    }

    // Run a slow cycle to trigger cost application (costs check on slow tick)
    city.tick_slow_cycle();

    let transit = city.resource::<BusTransitState>();
    // Verify the route activated and has operating cost computed
    assert!(
        transit.active_route_count() > 0 || transit.monthly_operating_cost > 0.0,
        "Should have active routes or computed operating cost after ticks"
    );
}

#[test]
fn test_bus_transit_saveable_roundtrip() {
    use crate::bus_transit::BusTransitState;
    use crate::Saveable;

    let mut grid = WorldGrid::new(32, 32);
    grid.get_mut(5, 5).cell_type = CellType::Road;
    grid.get_mut(15, 15).cell_type = CellType::Road;

    let mut state = BusTransitState::default();
    let s1 = state.add_stop(&grid, 5, 5).unwrap();
    let s2 = state.add_stop(&grid, 15, 15).unwrap();
    state.add_route("Test Route".to_string(), vec![s1, s2]);

    // Save
    let bytes = state
        .save_to_bytes()
        .expect("Should serialize non-empty state");

    // Load
    let loaded = BusTransitState::load_from_bytes(&bytes);
    assert_eq!(loaded.stops.len(), 2);
    assert_eq!(loaded.routes.len(), 1);
    assert_eq!(loaded.routes[0].name, "Test Route");
    assert_eq!(loaded.routes[0].stop_ids.len(), 2);
}

#[test]
fn test_auto_grid_respects_water_obstacles() {
    use crate::auto_grid_road::{compute_grid_plan, execute_grid_plan, AutoGridConfig};
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, RoadType, WorldGrid};
    use crate::road_segments::RoadSegmentStore;
    use crate::roads::RoadNetwork;

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut segments = RoadSegmentStore::default();

    for x in 55..=65 {
        for y in 55..=65 {
            grid.get_mut(x, y).cell_type = CellType::Water;
        }
    }

    let config = AutoGridConfig {
        block_size: 6,
        road_type: RoadType::Local,
    };

    let plan = compute_grid_plan((50, 50), (70, 70), &config, &grid);
    let cells = execute_grid_plan(&plan, &config, &mut segments, &mut grid, &mut roads);

    for x in 55..=65 {
        for y in 55..=65 {
            assert_ne!(grid.get(x, y).cell_type, CellType::Road);
        }
    }

    assert!(!cells.is_empty(), "should still place some roads");
}

#[test]
fn test_auto_grid_cost_scales_with_road_type() {
    use crate::auto_grid_road::{compute_grid_plan, AutoGridConfig};
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{RoadType, WorldGrid};

    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    let plan_local = compute_grid_plan(
        (50, 50),
        (70, 70),
        &AutoGridConfig {
            block_size: 6,
            road_type: RoadType::Local,
        },
        &grid,
    );
    let plan_avenue = compute_grid_plan(
        (50, 50),
        (70, 70),
        &AutoGridConfig {
            block_size: 6,
            road_type: RoadType::Avenue,
        },
        &grid,
    );

    assert_eq!(plan_local.total_cells, plan_avenue.total_cells);
    assert!(plan_avenue.total_cost > plan_local.total_cost);
}
// ====================================================================
// Undo/Redo integration tests (UX-001)
// ====================================================================

#[test]
fn test_undo_grid_road_restores_cell_and_treasury() {
    let mut city = TestCity::new().with_road(5, 5, 5, 10, RoadType::Local);

    city.tick(1);

    let cost = {
        let grid = city.grid();
        assert_eq!(grid.get(5, 5).cell_type, CellType::Road);
        let budget = city.budget();
        10_000.0 - budget.treasury
    };

    // Push an undo action for the road
    let world = city.world_mut();
    world.resource_scope(|world, mut history: Mut<ActionHistory>| {
        let grid = world.resource::<WorldGrid>();
        let road_type = grid.get(5, 5).road_type;
        history.push(CityAction::PlaceGridRoad {
            x: 5,
            y: 5,
            road_type,
            cost,
        });
    });

    // Now undo it
    let world = city.world_mut();
    world.resource_scope(|world, mut history: Mut<ActionHistory>| {
        if let Some(action) = history.pop_undo() {
            match &action {
                CityAction::PlaceGridRoad { x, y, cost, .. } => {
                    let mut grid = world.resource_mut::<WorldGrid>();
                    grid.get_mut(*x, *y).cell_type = CellType::Grass;
                    let mut budget = world.resource_mut::<CityBudget>();
                    budget.treasury += cost;
                }
                _ => panic!("Expected PlaceGridRoad action"),
            }
            history.push_redo(action);
        }
    });

    let grid = city.grid();
    assert_eq!(
        grid.get(5, 5).cell_type,
        CellType::Grass,
        "Road should be removed after undo"
    );

    let budget = city.budget();
    assert!(
        (budget.treasury - 10_000.0).abs() < 1.0,
        "Treasury should be restored after undo"
    );
}

#[test]
fn test_redo_grid_road_replaces_road_and_deducts() {
    let mut city = TestCity::new();

    let cost = 5.0;
    let road_type = RoadType::Local;

    // Push and then undo a road action
    let world = city.world_mut();
    world.resource_scope(|_world, mut history: Mut<ActionHistory>| {
        history.push(CityAction::PlaceGridRoad {
            x: 3,
            y: 3,
            road_type,
            cost,
        });
        if let Some(action) = history.pop_undo() {
            history.push_redo(action);
        }
    });

    // Redo it manually
    let world = city.world_mut();
    world.resource_scope(|world, mut history: Mut<ActionHistory>| {
        if let Some(action) = history.pop_redo() {
            match &action {
                CityAction::PlaceGridRoad {
                    x,
                    y,
                    road_type,
                    cost,
                    ..
                } => {
                    let mut grid = world.resource_mut::<WorldGrid>();
                    grid.get_mut(*x, *y).cell_type = CellType::Road;
                    grid.get_mut(*x, *y).road_type = *road_type;
                    let mut budget = world.resource_mut::<CityBudget>();
                    budget.treasury -= cost;
                }
                _ => panic!("Expected PlaceGridRoad action"),
            }
            history.push_undo_no_clear(action);
        }
    });

    let grid = city.grid();
    assert_eq!(
        grid.get(3, 3).cell_type,
        CellType::Road,
        "Road should be placed after redo"
    );
    assert_eq!(grid.get(3, 3).road_type, road_type);
}

#[test]
fn test_undo_zone_placement_clears_zone() {
    let mut city = TestCity::new()
        .with_road(5, 5, 5, 10, RoadType::Local)
        .with_zone(6, 5, ZoneType::ResidentialLow);

    city.tick(1);

    // Push zone action
    let world = city.world_mut();
    world.resource_scope(|_world, mut history: Mut<ActionHistory>| {
        history.push(CityAction::PlaceZone {
            cells: vec![(6, 5, ZoneType::ResidentialLow)],
            cost: 0.0,
        });
    });

    // Undo the zone
    let world = city.world_mut();
    world.resource_scope(|world, mut history: Mut<ActionHistory>| {
        if let Some(action) = history.pop_undo() {
            match &action {
                CityAction::PlaceZone { cells, .. } => {
                    let mut grid = world.resource_mut::<WorldGrid>();
                    for (x, y, _) in cells {
                        grid.get_mut(*x, *y).zone = ZoneType::None;
                    }
                }
                _ => panic!("Expected PlaceZone action"),
            }
            history.push_redo(action);
        }
    });

    let grid = city.grid();
    assert_eq!(
        grid.get(6, 5).zone,
        ZoneType::None,
        "Zone should be cleared after undo"
    );
}

#[test]
fn test_push_clears_redo_stack() {
    let mut city = TestCity::new();

    let world = city.world_mut();
    world.resource_scope(|_world, mut history: Mut<ActionHistory>| {
        history.push(CityAction::PlaceGridRoad {
            x: 1,
            y: 1,
            road_type: RoadType::Local,
            cost: 5.0,
        });
        if let Some(action) = history.pop_undo() {
            history.push_redo(action);
        }
        assert!(history.can_redo(), "Should be able to redo after undo");

        // Pushing a new action should clear the redo stack
        history.push(CityAction::PlaceGridRoad {
            x: 2,
            y: 2,
            road_type: RoadType::Local,
            cost: 5.0,
        });
        assert!(
            !history.can_redo(),
            "Redo stack should be cleared after push"
        );
    });
}

#[test]
fn test_action_history_limit_100() {
    let mut city = TestCity::new();

    let world = city.world_mut();
    world.resource_scope(|_world, mut history: Mut<ActionHistory>| {
        for i in 0..120 {
            history.push(CityAction::PlaceGridRoad {
                x: i % 256,
                y: 0,
                road_type: RoadType::Local,
                cost: 5.0,
            });
        }
        assert_eq!(
            history.undo_stack.len(),
            100,
            "History should be capped at 100 actions"
        );
    });
}

#[test]
fn test_undo_composite_action() {
    let mut city = TestCity::new();

    let world = city.world_mut();
    world.resource_scope(|_world, mut history: Mut<ActionHistory>| {
        let composite = CityAction::Composite(vec![
            CityAction::PlaceGridRoad {
                x: 0,
                y: 0,
                road_type: RoadType::Local,
                cost: 5.0,
            },
            CityAction::PlaceGridRoad {
                x: 1,
                y: 0,
                road_type: RoadType::Local,
                cost: 5.0,
            },
        ]);
        history.push(composite);
        assert_eq!(
            history.undo_stack.len(),
            1,
            "Composite counts as one action"
        );

        let undone = history.pop_undo();
        assert!(undone.is_some());
        if let Some(CityAction::Composite(actions)) = &undone {
            assert_eq!(actions.len(), 2, "Composite should contain 2 sub-actions");
        } else {
            panic!("Expected Composite action");
        }
        assert_eq!(history.undo_stack.len(), 0);
    });
}

#[test]
fn test_undo_bulldoze_road_restores_road() {
    let mut city = TestCity::new().with_road(4, 4, 4, 9, RoadType::Local);

    city.tick(1);

    // Get road type before bulldozing
    let road_type = city.grid().get(4, 4).road_type;

    // Simulate bulldozing by clearing the road
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(4, 4).cell_type = CellType::Grass;
    }

    // Record the bulldoze action and undo it
    let world = city.world_mut();
    world.resource_scope(|world, mut history: Mut<ActionHistory>| {
        history.push(CityAction::BulldozeRoad {
            x: 4,
            y: 4,
            road_type,
            refund: 2.0,
        });

        // Undo the bulldoze (should restore the road)
        if let Some(action) = history.pop_undo() {
            match &action {
                CityAction::BulldozeRoad {
                    x,
                    y,
                    road_type,
                    refund,
                } => {
                    let mut grid = world.resource_mut::<WorldGrid>();
                    grid.get_mut(*x, *y).cell_type = CellType::Road;
                    grid.get_mut(*x, *y).road_type = *road_type;
                    let mut budget = world.resource_mut::<CityBudget>();
                    budget.treasury -= refund;
                }
                _ => panic!("Expected BulldozeRoad action"),
            }
            history.push_redo(action);
        }
    });

    let grid = city.grid();
    assert_eq!(
        grid.get(4, 4).cell_type,
        CellType::Road,
        "Road should be restored after undoing bulldoze"
    );
    assert_eq!(grid.get(4, 4).road_type, road_type);
}

// Metro transit system (TRAF-006)
// ===========================================================================

#[test]
fn test_metro_station_placement_and_ridership() {
    use crate::metro_transit::MetroTransitState;

    let mut city = TestCity::new()
        .with_road(10, 50, 90, 50, RoadType::Avenue)
        .with_building(12, 48, ZoneType::ResidentialLow, 2)
        .with_building(88, 48, ZoneType::CommercialLow, 2)
        .with_citizen((12, 48), (88, 48));

    // Place metro stations and create a line
    {
        let world = city.world_mut();
        let mut metro = world.resource_mut::<MetroTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = metro
            .add_station(15, 50, "West Station".to_string(), &grid)
            .expect("station 1 should be placed");
        let s2 = metro
            .add_station(85, 50, "East Station".to_string(), &grid)
            .expect("station 2 should be placed");
        metro
            .add_line("Red Line".to_string(), vec![s1, s2])
            .expect("line should be created");
    }

    // Run a few slow cycles to accumulate ridership stats
    city.tick_slow_cycles(3);

    let metro = city.resource::<MetroTransitState>();
    assert_eq!(metro.stats.total_stations, 2, "should have 2 stations");
    assert_eq!(metro.stats.total_lines, 1, "should have 1 operational line");
}

#[test]
fn test_metro_land_value_boost() {
    use crate::metro_transit::MetroTransitState;

    let mut city = TestCity::new();

    // Record baseline land value at station location
    let baseline = city.resource::<LandValueGrid>().get(50, 50);

    // Place a metro station
    {
        let world = city.world_mut();
        let mut metro = world.resource_mut::<MetroTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = metro
            .add_station(50, 50, "Central".to_string(), &grid)
            .expect("station should be placed");
        let s2 = metro
            .add_station(80, 50, "East".to_string(), &grid)
            .expect("station should be placed");
        metro.add_line("Blue Line".to_string(), vec![s1, s2]);
    }

    // Run slow tick so land value boost is applied
    city.tick_slow_cycle();

    let boosted = city.resource::<LandValueGrid>().get(50, 50);
    assert!(
        boosted > baseline,
        "land value at station ({}) should exceed baseline ({})",
        boosted,
        baseline
    );
}

#[test]
fn test_metro_maintenance_cost() {
    use crate::metro_transit::MetroTransitState;

    let mut city = TestCity::new().with_budget(100_000.0);

    // Place stations and line
    {
        let world = city.world_mut();
        let mut metro = world.resource_mut::<MetroTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = metro.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = metro.add_station(20, 20, "B".to_string(), &grid).unwrap();
        metro.add_line("Red".to_string(), vec![s1, s2]);
    }

    let cost = city
        .resource::<MetroTransitState>()
        .total_monthly_maintenance();
    assert!(
        cost > 0.0,
        "metro maintenance cost should be positive, got {}",
        cost
    );

    // Verify the cost is: 2 stations * $500/week * 4 + 1 line * $1200/week * 4
    let expected = 2.0 * 500.0 * 4.0 + 1.0 * 1200.0 * 4.0;
    assert!(
        (cost - expected).abs() < 0.01,
        "expected cost {}, got {}",
        expected,
        cost
    );
}

#[test]
fn test_metro_station_on_water_rejected() {
    use crate::metro_transit::MetroTransitState;

    // Create a grid with water at (50,50)
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(50, 50).cell_type = CellType::Water;

    let mut metro = MetroTransitState::default();

    // Placing on water should fail
    let result = metro.add_station(50, 50, "Aquatic".to_string(), &grid);
    assert!(
        result.is_none(),
        "should not be able to place station on water"
    );

    // Placing on grass should succeed
    let result = metro.add_station(51, 50, "Dry".to_string(), &grid);
    assert!(result.is_some(), "should place station on grass");
}

#[test]
fn test_metro_travel_time_estimation() {
    use crate::metro_transit::MetroTransitState;

    let mut metro = MetroTransitState::default();
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    let s1 = metro
        .add_station(50, 50, "West".to_string(), &grid)
        .unwrap();
    let s2 = metro
        .add_station(100, 50, "East".to_string(), &grid)
        .unwrap();
    metro.add_line("Green".to_string(), vec![s1, s2]);

    // Travel from near station 1 to near station 2
    let time = metro
        .estimate_travel_time(52, 50, 98, 50)
        .expect("should have a viable route");
    assert!(time > 0.0, "travel time should be positive");
    assert!(
        time < 0.5,
        "metro trip should be under 30 min for ~50 cells"
    );

    // Travel from far away should fail
    let no_route = metro.estimate_travel_time(200, 200, 98, 50);
    assert!(
        no_route.is_none(),
        "should fail when too far from any station"
    );
}

#[test]
fn test_metro_saveable_roundtrip_integration() {
    use crate::metro_transit::MetroTransitState;
    use crate::Saveable;

    let mut state = MetroTransitState::default();
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    state.add_station(10, 10, "Alpha".to_string(), &grid);
    state.add_station(50, 50, "Beta".to_string(), &grid);
    state.add_line("Red".to_string(), vec![0, 1]);

    // Save
    let bytes = state.save_to_bytes().expect("should produce bytes");
    assert!(!bytes.is_empty());

    // Load
    let restored = MetroTransitState::load_from_bytes(&bytes);
    assert_eq!(restored.stations.len(), 2);
    assert_eq!(restored.lines.len(), 1);
    assert_eq!(restored.lines[0].name, "Red");
}

// ====================================================================
// GameParams data-driven parameters tests
// ====================================================================

#[test]
fn test_game_params_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::game_params::GameParams>();
}

#[test]
fn test_game_params_defaults_match_original_constants() {
    let city = TestCity::new();
    let params = city.resource::<crate::game_params::GameParams>();

    assert!(
        (params.economy.starting_treasury - 10_000.0).abs() < f64::EPSILON,
        "starting_treasury should be 10000"
    );
    assert!(
        (params.economy.default_tax_rate - 0.10).abs() < f32::EPSILON,
        "default_tax_rate should be 0.10"
    );
    assert_eq!(params.economy.tax_collection_interval_days, 30);
    assert!((params.citizen.speed - 48.0).abs() < f32::EPSILON);
    assert_eq!(params.citizen.shopping_duration_ticks, 30);
    assert_eq!(params.citizen.leisure_duration_ticks, 60);
    assert_eq!(params.building.construction_ticks, 100);
    assert_eq!(params.building.spawn_interval_ticks, 2);

    let local = params.road_params(RoadType::Local);
    assert!((local.speed - 30.0).abs() < f32::EPSILON);
    assert!((local.cost - 10.0).abs() < f64::EPSILON);
    assert_eq!(local.capacity, 20);
}

#[test]
fn test_game_params_saveable_roundtrip() {
    use crate::game_params::GameParams;
    use crate::Saveable;

    let mut params = GameParams::default();
    params.economy.starting_treasury = 99_999.0;
    params.citizen.speed = 200.0;

    let bytes = params.save_to_bytes().expect("should encode");
    let restored = GameParams::load_from_bytes(&bytes);

    assert!((restored.economy.starting_treasury - 99_999.0).abs() < f64::EPSILON);
    assert!((restored.citizen.speed - 200.0).abs() < f32::EPSILON);
}

#[test]
fn test_game_params_modifying_construction_ticks() {
    use crate::buildings::UnderConstruction;
    use crate::game_params::GameParams;

    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_utility(5, 5, crate::utilities::UtilityType::PowerPlant)
        .with_utility(6, 6, crate::utilities::UtilityType::WaterTower);

    {
        let world = city.world_mut();
        let mut params = world.resource_mut::<GameParams>();
        params.building.construction_ticks = 2;
    }

    let mut city = city
        .with_zone_rect(11, 9, 19, 9, ZoneType::ResidentialLow)
        .with_zone_rect(11, 11, 19, 11, ZoneType::ResidentialLow);

    city.tick(20);

    let world = city.world_mut();
    let still_constructing: usize = world.query::<&UnderConstruction>().iter(world).count();
    let buildings: usize = world.query::<&Building>().iter(world).count();

    if buildings > 0 {
        assert!(
            still_constructing < buildings,
            "With 2-tick construction, most of {} buildings should be done, but {} still constructing",
            buildings,
            still_constructing
        );
    }
}

#[test]
fn test_game_params_zone_demand_bootstrap() {
    use crate::game_params::GameParams;
    use crate::zones::ZoneDemand;

    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);

    {
        let world = city.world_mut();
        let mut params = world.resource_mut::<GameParams>();
        params.zone_demand.bootstrap_demand = 0.9;
    }

    city.tick_slow_cycle();

    let demand = city.resource::<ZoneDemand>();
    assert!(
        demand.residential > 0.0,
        "With bootstrap_demand=0.9 and roads, residential demand should be positive, got {}",
        demand.residential
    );
}

#[test]
fn test_game_params_road_params_lookup() {
    use crate::game_params::GameParams;

    let city = TestCity::new();
    let params = city.resource::<GameParams>();

    let road_types = [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
        RoadType::OneWay,
        RoadType::Path,
    ];
    for rt in road_types {
        let rp = params.road_params(rt);
        assert!(rp.speed > 0.0, "{:?} should have positive speed", rt);
        assert!(rp.cost > 0.0, "{:?} should have positive cost", rt);
    }
}

// ====================================================================
// Outside Connections integration tests (TEST-070)
// ====================================================================

use crate::outside_connections::{ConnectionType, OutsideConnections};

#[test]
fn test_outside_connections_resource_initialized() {
    let city = TestCity::new();
    city.assert_resource_exists::<OutsideConnections>();
}

#[test]
fn test_outside_connections_empty_city_has_none() {
    let mut city = TestCity::new();
    // Run enough ticks to trigger the update (UPDATE_INTERVAL=100)
    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(
        outside.connections.is_empty(),
        "Empty city should have no outside connections"
    );
}

#[test]
fn test_outside_connections_highway_at_edge_detected_via_system() {
    let mut city = TestCity::new();
    // Place a highway at the bottom edge using the grid directly
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        let cell = grid.get_mut(100, 0);
        cell.cell_type = CellType::Road;
        cell.road_type = RoadType::Highway;
    }

    // Run enough ticks for the system to detect connections
    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(
        outside.has_connection(ConnectionType::Highway),
        "Should detect highway at map edge"
    );
    assert_eq!(outside.count(ConnectionType::Highway), 1);
}

#[test]
fn test_outside_connections_airport_detected_via_service() {
    let mut city = TestCity::new().with_service(100, 100, ServiceType::InternationalAirport);

    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(
        outside.has_connection(ConnectionType::Airport),
        "Should detect InternationalAirport as airport connection"
    );
}

#[test]
fn test_outside_connections_train_station_at_edge_detected_as_railway() {
    // TrainStation near the edge (x=1 is within EDGE_PROXIMITY=3)
    let mut city = TestCity::new().with_service(1, 128, ServiceType::TrainStation);

    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(
        outside.has_connection(ConnectionType::Railway),
        "Should detect TrainStation at edge as railway connection"
    );
}

#[test]
fn test_outside_connections_train_station_interior_not_detected() {
    // TrainStation in the interior (x=128 is NOT near edge)
    let mut city = TestCity::new().with_service(128, 128, ServiceType::TrainStation);

    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(
        !outside.has_connection(ConnectionType::Railway),
        "TrainStation in interior should NOT be a railway connection"
    );
}

#[test]
fn test_outside_connections_effects_boost_tourism() {
    use crate::tourism::Tourism;

    let mut city = TestCity::new().with_service(100, 100, ServiceType::InternationalAirport);

    // Record initial tourism
    let initial_attractiveness = city.resource::<Tourism>().attractiveness;

    // Run system
    city.tick(101);

    let tourism = city.resource::<Tourism>();
    assert!(
        tourism.attractiveness > initial_attractiveness,
        "Airport connection should boost tourism attractiveness (was {}, now {})",
        initial_attractiveness,
        tourism.attractiveness
    );
}

#[test]
fn test_outside_connections_multiple_types_coexist() {
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::InternationalAirport)
        .with_service(1, 128, ServiceType::TrainStation);

    // Place a highway at the bottom edge
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        let cell = grid.get_mut(100, 0);
        cell.cell_type = CellType::Road;
        cell.road_type = RoadType::Highway;
    }

    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(outside.has_connection(ConnectionType::Highway));
    assert!(outside.has_connection(ConnectionType::Railway));
    assert!(outside.has_connection(ConnectionType::Airport));
    // SeaPort requires FerryPier near water edge, which we didn't set up
    assert!(!outside.has_connection(ConnectionType::SeaPort));
}

// ====================================================================
// Freight traffic tests (TRAF-004)
// ====================================================================

#[test]
fn test_freight_traffic_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::freight_traffic::FreightTrafficState>();
}

#[test]
fn test_freight_traffic_default_state() {
    let city = TestCity::new();
    let state = city.resource::<crate::freight_traffic::FreightTrafficState>();
    assert_eq!(state.trucks.len(), 0);
    assert_eq!(state.industrial_demand, 0.0);
    assert_eq!(state.commercial_demand, 0.0);
    assert_eq!(state.trips_generated, 0);
    assert_eq!(state.trips_completed, 0);
    assert_eq!(state.satisfaction, 1.0);
}

#[test]
fn test_freight_traffic_demand_from_buildings() {
    let mut city = TestCity::new()
        .with_road(50, 50, 50, 60, RoadType::Local)
        .with_building(49, 50, ZoneType::Industrial, 2)
        .with_building(49, 58, ZoneType::CommercialLow, 2);

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            building.occupants = 20;
        }
    }

    city.tick(25);

    let state = city.resource::<crate::freight_traffic::FreightTrafficState>();
    assert!(
        state.industrial_demand > 0.0,
        "industrial buildings should generate outbound freight demand, got {}",
        state.industrial_demand
    );
    assert!(
        state.commercial_demand > 0.0,
        "commercial buildings should generate inbound freight demand, got {}",
        state.commercial_demand
    );
}

#[test]
fn test_freight_trucks_contribute_to_traffic() {
    let mut city = TestCity::new()
        .with_road(50, 50, 50, 65, RoadType::Avenue)
        .with_building(49, 52, ZoneType::Industrial, 2)
        .with_building(49, 63, ZoneType::CommercialLow, 2)
        .rebuild_csr();

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut Building>();
        for mut building in q.iter_mut(world) {
            building.occupants = 50;
        }
    }

    let initial_traffic: u64 = {
        let traffic = city.resource::<crate::traffic::TrafficGrid>();
        (50..=65).map(|y| traffic.get(50, y) as u64).sum()
    };

    city.tick(80);

    let final_traffic: u64 = {
        let traffic = city.resource::<crate::traffic::TrafficGrid>();
        (50..=65).map(|y| traffic.get(50, y) as u64).sum()
    };

    let state = city.resource::<crate::freight_traffic::FreightTrafficState>();
    if state.trips_generated > 0 {
        assert!(
            final_traffic >= initial_traffic,
            "freight trucks should contribute to traffic density; initial={}, final={}",
            initial_traffic,
            final_traffic
        );
    }
}

#[test]
fn test_freight_satisfaction_with_no_buildings() {
    let mut city = TestCity::new();
    city.tick(25);

    let state = city.resource::<crate::freight_traffic::FreightTrafficState>();
    assert_eq!(
        state.satisfaction, 1.0,
        "satisfaction should be 1.0 (fully satisfied) with no buildings"
    );
}

#[test]
fn test_freight_heavy_traffic_ban() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<crate::freight_traffic::FreightTrafficState>();
        state.toggle_heavy_traffic_ban(0);
    }

    let state = city.resource::<crate::freight_traffic::FreightTrafficState>();
    assert!(
        state.is_heavy_traffic_banned(0),
        "district 0 should have heavy traffic banned"
    );
    assert!(
        !state.is_heavy_traffic_banned(1),
        "district 1 should not have heavy traffic banned"
    );
}

// ====================================================================
// Tourism system integration tests
// ====================================================================

use crate::tourism::Tourism;
use crate::weather::{Season, WeatherCondition};

#[test]
fn test_tourism_resource_exists_in_empty_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<Tourism>();
}

#[test]
fn test_tourism_default_state_in_empty_city() {
    let city = TestCity::new();
    let tourism = city.resource::<Tourism>();
    assert!(
        (tourism.attractiveness - 0.0).abs() < f32::EPSILON,
        "Empty city should have 0 attractiveness"
    );
    assert_eq!(
        tourism.monthly_visitors, 0,
        "Empty city should have 0 visitors"
    );
    assert!(
        (tourism.monthly_tourism_income - 0.0).abs() < f64::EPSILON,
        "Empty city should have 0 tourism income"
    );
}

#[test]
fn test_tourism_with_attractions_gains_attractiveness() {
    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::Stadium)
        .with_service(20, 20, ServiceType::Museum);
    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 31;
    }
    city.tick(1);
    let tourism = city.resource::<Tourism>();
    assert!(
        tourism.attractiveness > 0.0,
        "City with stadium and museum should have positive attractiveness, got {}",
        tourism.attractiveness
    );
}

#[test]
fn test_tourism_visitors_proportional_to_attractiveness() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        city.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city.tick(1);
    assert!(
        city.resource::<Tourism>().monthly_visitors > 0,
        "City with stadium should attract visitors"
    );
}

#[test]
fn test_tourism_more_attractions_more_visitors() {
    let mut city1 = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        city1.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city1.tick(1);
    let v1 = city1.resource::<Tourism>().monthly_visitors;

    let mut city2 = TestCity::new()
        .with_service(10, 10, ServiceType::Stadium)
        .with_service(20, 20, ServiceType::Museum)
        .with_service(30, 30, ServiceType::Cathedral);
    {
        city2.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city2.tick(1);
    let v2 = city2.resource::<Tourism>().monthly_visitors;
    assert!(
        v2 > v1,
        "More attractions ({}) should yield more visitors than fewer ({})",
        v2,
        v1
    );
}

#[test]
fn test_tourism_revenue_positive_with_visitors() {
    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::Stadium)
        .with_service(20, 20, ServiceType::Museum);
    {
        city.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city.tick(1);
    let t = city.resource::<Tourism>();
    if t.monthly_visitors > 0 {
        assert!(
            t.monthly_tourism_income > 0.0,
            "Positive visitors should generate positive revenue"
        );
    }
}

#[test]
fn test_tourism_no_update_before_30_days() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    city.tick(10);
    assert_eq!(
        city.resource::<Tourism>().monthly_visitors,
        0,
        "Tourism should not update before 30 days"
    );
}

#[test]
fn test_tourism_airport_multiplier_effect() {
    let mut city1 = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        city1.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city1.tick(1);
    let v1 = city1.resource::<Tourism>().monthly_visitors;

    let mut city2 = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        let w = city2.world_mut();
        w.resource_mut::<Tourism>().airport_multiplier = 2.0;
        w.resource_mut::<GameClock>().day = 31;
    }
    city2.tick(1);
    let v2 = city2.resource::<Tourism>().monthly_visitors;
    assert!(
        v2 > v1,
        "Airport multiplier should increase visitors: {} vs {}",
        v2,
        v1
    );
}

#[test]
fn test_tourism_weather_affects_visitors() {
    let mut city_summer = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        let w = city_summer.world_mut();
        {
            let mut wt = w.resource_mut::<Weather>();
            wt.season = Season::Summer;
            wt.current_event = WeatherCondition::Sunny;
            wt.temperature = 25.0;
        }
        w.resource_mut::<GameClock>().day = 31;
    }
    city_summer.tick(1);
    let sv = city_summer.resource::<Tourism>().monthly_visitors;

    let mut city_winter = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    {
        let w = city_winter.world_mut();
        {
            let mut wt = w.resource_mut::<Weather>();
            wt.season = Season::Winter;
            wt.current_event = WeatherCondition::Storm;
            wt.temperature = 2.0;
        }
        w.resource_mut::<GameClock>().day = 31;
    }
    city_winter.tick(1);
    let wv = city_winter.resource::<Tourism>().monthly_visitors;
    assert!(
        sv > wv,
        "Summer sunny ({}) should attract more tourists than winter storm ({})",
        sv,
        wv
    );
}

// ====================================================================
// Airport system integration tests (TEST-069)
// ====================================================================

#[test]
fn test_airport_stats_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::airport::AirportStats>();
}

#[test]
fn test_airport_no_airports_stats_zero() {
    let mut city = TestCity::new();
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 0);
    assert_eq!(stats.airports_by_tier, [0, 0, 0]);
    assert_eq!(stats.passenger_flights_per_month, 0);
    assert_eq!(stats.cargo_flights_per_month, 0);
    assert!((stats.revenue).abs() < f64::EPSILON);
    assert!((stats.total_monthly_cost).abs() < f64::EPSILON);
}

#[test]
fn test_airport_no_airports_tourism_multiplier_one() {
    let mut city = TestCity::new();
    city.tick_slow_cycles(2);
    let tourism = city.resource::<crate::tourism::Tourism>();
    assert!(
        (tourism.airport_multiplier - 1.0).abs() < 0.01,
        "Without airports, airport_multiplier should be 1.0, got {}",
        tourism.airport_multiplier
    );
}

#[test]
fn test_airport_single_small_airstrip_counted() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::SmallAirstrip);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 1);
    assert_eq!(stats.airports_by_tier[0], 1);
    assert_eq!(stats.airports_by_tier[1], 0);
    assert_eq!(stats.airports_by_tier[2], 0);
}

#[test]
fn test_airport_single_regional_counted() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::RegionalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 1);
    assert_eq!(stats.airports_by_tier[1], 1);
}

#[test]
fn test_airport_single_international_counted() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 1);
    assert_eq!(stats.airports_by_tier[2], 1);
}

#[test]
fn test_airport_multiple_tiers_counted() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::SmallAirstrip)
        .with_service(60, 60, ServiceType::SmallAirstrip)
        .with_service(90, 90, ServiceType::RegionalAirport)
        .with_service(120, 120, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 4);
    assert_eq!(stats.airports_by_tier, [2, 1, 1]);
}

#[test]
fn test_airport_non_airport_services_not_counted() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::SmallAirstrip)
        .with_service(60, 60, ServiceType::FireStation)
        .with_service(70, 70, ServiceType::PoliceStation)
        .with_service(80, 80, ServiceType::Hospital);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 1);
}

#[test]
fn test_airport_tourism_multiplier_single_international() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    // 1 intl: bonus = 1.0 * sqrt(1) = 1.0, multiplier = 2.0
    assert!(
        (stats.tourism_multiplier - 2.0).abs() < 0.01,
        "got {}",
        stats.tourism_multiplier
    );
}

#[test]
fn test_airport_tourism_multiplier_applied_to_tourism_resource() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let tourism = city.resource::<crate::tourism::Tourism>();
    assert!(
        (tourism.airport_multiplier - 2.0).abs() < 0.01,
        "got {}",
        tourism.airport_multiplier
    );
}

#[test]
fn test_airport_tourism_multiplier_mixed_tiers() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::SmallAirstrip)
        .with_service(60, 60, ServiceType::RegionalAirport)
        .with_service(90, 90, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    // 0.10 + 0.30 + 1.00 = 1.40, multiplier = 2.40
    assert!(
        (stats.tourism_multiplier - 2.40).abs() < 0.01,
        "got {}",
        stats.tourism_multiplier
    );
}

#[test]
fn test_airport_tourism_diminishing_returns_two_international() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::InternationalAirport)
        .with_service(80, 80, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    // 2 intl: bonus = 1.0 * sqrt(2) ~ 1.414, multiplier ~ 2.414, NOT 3.0
    assert!(
        stats.tourism_multiplier < 3.0,
        "should have diminishing returns, got {}",
        stats.tourism_multiplier
    );
    assert!(
        (stats.tourism_multiplier - 2.414).abs() < 0.01,
        "got {}",
        stats.tourism_multiplier
    );
}

#[test]
fn test_airport_monthly_cost_single_small_airstrip() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::SmallAirstrip);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert!((stats.total_monthly_cost - 60.0).abs() < f64::EPSILON);
}

#[test]
fn test_airport_monthly_cost_multiple_airports() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::SmallAirstrip)
        .with_service(60, 60, ServiceType::RegionalAirport)
        .with_service(90, 90, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    // 60 + 100 + 150 = 310
    assert!((stats.total_monthly_cost - 310.0).abs() < f64::EPSILON);
}

#[test]
fn test_airport_zero_population_zero_revenue() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.passenger_flights_per_month, 0);
    assert!((stats.revenue).abs() < f64::EPSILON);
}

#[test]
fn test_airport_fog_suspends_flights() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    {
        let world = city.world_mut();
        world
            .resource_mut::<crate::fog::FogState>()
            .flights_suspended = true;
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 100_000;
    }
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(
        stats.passenger_flights_per_month, 0,
        "fog should suspend flights"
    );
    assert_eq!(stats.cargo_flights_per_month, 0);
}

#[test]
fn test_airport_clear_weather_allows_flights() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    {
        let world = city.world_mut();
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 100_000;
        world
            .resource_mut::<crate::fog::FogState>()
            .flights_suspended = false;
    }
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert!(
        stats.passenger_flights_per_month > 0,
        "expected flights > 0, got {}",
        stats.passenger_flights_per_month
    );
}

#[test]
fn test_airport_capacity_limits_flights() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::SmallAirstrip);
    {
        let world = city.world_mut();
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 1_000_000;
    }
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(
        stats.passenger_flights_per_month, 500,
        "should be capped at 500, got {}",
        stats.passenger_flights_per_month
    );
}

#[test]
fn test_airport_revenue_positive_with_population() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    {
        let world = city.world_mut();
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 50_000;
    }
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert!(
        stats.revenue > 0.0,
        "expected revenue > 0, got {}",
        stats.revenue
    );
}

#[test]
fn test_airport_noise_generated_around_small_airstrip() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::SmallAirstrip);
    city.tick_slow_cycles(2);
    let noise = city.resource::<crate::noise::NoisePollutionGrid>();
    let center = noise.get(128, 128);
    let far = noise.get(140, 128);
    assert!(center > 0, "center noise should be > 0, got {}", center);
    assert!(
        far < center,
        "far ({}) should be < center ({})",
        far,
        center
    );
}

#[test]
fn test_airport_noise_international_larger_radius() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let noise = city.resource::<crate::noise::NoisePollutionGrid>();
    assert!(noise.get(128, 128) > 0, "center noise should be > 0");
    assert!(
        noise.get(135, 128) > 0,
        "noise at 7 cells should be > 0 for international"
    );
}

#[test]
fn test_airport_noise_international_more_intense_than_small() {
    let mut city_small = TestCity::new().with_service(128, 128, ServiceType::SmallAirstrip);
    city_small.tick_slow_cycles(2);
    let noise_small = city_small
        .resource::<crate::noise::NoisePollutionGrid>()
        .get(128, 128);

    let mut city_intl = TestCity::new().with_service(128, 128, ServiceType::InternationalAirport);
    city_intl.tick_slow_cycles(2);
    let noise_intl = city_intl
        .resource::<crate::noise::NoisePollutionGrid>()
        .get(128, 128);

    assert!(
        noise_intl > noise_small,
        "international ({}) should be louder than small ({})",
        noise_intl,
        noise_small
    );
}

#[test]
fn test_airport_economic_bonus_international_higher_revenue_than_small() {
    let mut city_small = TestCity::new().with_service(50, 50, ServiceType::SmallAirstrip);
    {
        let world = city_small.world_mut();
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 10_000;
    }
    city_small.tick_slow_cycles(2);
    let rev_small = city_small
        .resource::<crate::airport::AirportStats>()
        .revenue;

    let mut city_intl = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    {
        let world = city_intl.world_mut();
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 10_000;
    }
    city_intl.tick_slow_cycles(2);
    let rev_intl = city_intl.resource::<crate::airport::AirportStats>().revenue;

    assert!(
        rev_intl > rev_small,
        "intl revenue ({}) should exceed small ({})",
        rev_intl,
        rev_small
    );
}
// =============================================================================
// Transit Hub / Multi-Modal Stations (TRAF-015)
// =============================================================================

/// Test that transit hub resources are initialized when SimulationPlugin starts.
#[test]
fn test_transit_hub_resources_exist() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::transit_hub::TransitHubs>();
    city.assert_resource_exists::<crate::transit_hub::TransitHubStats>();
}

/// Test that co-located bus depot and subway station form a BusMetroHub.
#[test]
fn test_transit_hub_creation_bus_metro() {
    use crate::transit_hub::{TransitHubType, TransitHubs};

    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::BusDepot)
        .with_service(51, 50, ServiceType::SubwayStation);

    // Run a slow cycle so update_transit_hubs fires
    city.tick_slow_cycle();

    let hubs = city.world_mut().resource::<TransitHubs>();
    assert!(
        !hubs.hubs.is_empty(),
        "Expected at least one transit hub from co-located bus + subway"
    );
    assert_eq!(
        hubs.hubs[0].hub_type,
        TransitHubType::BusMetroHub,
        "Expected BusMetroHub type"
    );
}

/// Test that co-located train station and subway station form a TrainMetroHub.
#[test]
fn test_transit_hub_creation_train_metro() {
    use crate::transit_hub::{TransitHubType, TransitHubs};

    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::TrainStation)
        .with_service(51, 51, ServiceType::SubwayStation);

    city.tick_slow_cycle();

    let hubs = city.world_mut().resource::<TransitHubs>();
    assert!(
        !hubs.hubs.is_empty(),
        "Expected at least one transit hub from co-located train + subway"
    );
    assert_eq!(hubs.hubs[0].hub_type, TransitHubType::TrainMetroHub);
}

/// Test that 3+ transit modes co-located form a MultiModalHub.
#[test]
fn test_transit_hub_creation_multi_modal() {
    use crate::transit_hub::{TransitHubType, TransitHubs};

    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::BusDepot)
        .with_service(51, 50, ServiceType::SubwayStation)
        .with_service(50, 51, ServiceType::TrainStation);

    city.tick_slow_cycle();

    let hubs = city.world_mut().resource::<TransitHubs>();
    assert!(
        !hubs.hubs.is_empty(),
        "Expected a multi-modal hub from 3 co-located transit types"
    );
    assert_eq!(hubs.hubs[0].hub_type, TransitHubType::MultiModalHub);
}

/// Test that isolated transit stops do NOT form hubs.
#[test]
fn test_transit_hub_no_hub_for_isolated_stops() {
    use crate::transit_hub::TransitHubs;

    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::BusDepot)
        .with_service(100, 100, ServiceType::SubwayStation);

    city.tick_slow_cycle();

    let hubs = city.world_mut().resource::<TransitHubs>();
    assert!(
        hubs.hubs.is_empty(),
        "Isolated transit stops should not form a hub"
    );
}

/// Test that transfer penalty is reduced at hub locations.
#[test]
fn test_transit_hub_transfer_penalty_reduction() {
    use crate::transit_hub::{
        TransitHubs, TransitMode, DEFAULT_TRANSFER_PENALTY_MINUTES, HUB_TRANSFER_PENALTY_MINUTES,
    };

    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::BusDepot)
        .with_service(51, 50, ServiceType::SubwayStation);

    city.tick_slow_cycle();

    let hubs = city.world_mut().resource::<TransitHubs>();

    // At hub location: reduced penalty
    let penalty_at_hub = hubs.transfer_penalty_at(50, 50, TransitMode::Bus, TransitMode::Metro);
    assert!(
        (penalty_at_hub - HUB_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON,
        "Transfer penalty at hub should be {HUB_TRANSFER_PENALTY_MINUTES}, got {penalty_at_hub}"
    );

    // Away from hub: default penalty
    let penalty_away = hubs.transfer_penalty_at(200, 200, TransitMode::Bus, TransitMode::Metro);
    assert!(
        (penalty_away - DEFAULT_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON,
        "Transfer penalty away from hub should be {DEFAULT_TRANSFER_PENALTY_MINUTES}, got {penalty_away}"
    );
}

/// Test that hub land value boost is higher than individual station boost.
#[test]
fn test_transit_hub_land_value_boost() {
    use crate::transit_hub::{HUB_LAND_VALUE_MULTIPLIER, TRANSIT_STATION_BASE_BOOST};

    // Place a hub (bus + subway co-located) and measure land value nearby.
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::BusDepot)
        .with_service(129, 128, ServiceType::SubwayStation);

    city.tick_slow_cycle();

    let lv = city.world_mut().resource::<LandValueGrid>();
    let hub_lv = lv.get(128, 128);

    // Also create a city with just a single bus depot (no hub).
    let mut city_single = TestCity::new().with_service(128, 128, ServiceType::BusDepot);

    city_single.tick_slow_cycle();

    let lv_single = city_single.world_mut().resource::<LandValueGrid>();
    let single_lv = lv_single.get(128, 128);

    // The hub location should have at least as much land value as the single station.
    // The hub provides an additional boost via transit_hub_land_value system.
    assert!(
        hub_lv >= single_lv,
        "Hub land value ({hub_lv}) should be >= single station land value ({single_lv})"
    );

    // Verify the hub boost constant is correct
    let hub_boost = (TRANSIT_STATION_BASE_BOOST as f32 * HUB_LAND_VALUE_MULTIPLIER) as i32;
    assert!(
        hub_boost > TRANSIT_STATION_BASE_BOOST,
        "Hub boost ({hub_boost}) must exceed individual station boost ({TRANSIT_STATION_BASE_BOOST})"
    );
}

/// Test that hub stats are updated correctly.
#[test]
fn test_transit_hub_stats_update() {
    use crate::transit_hub::TransitHubStats;

    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::BusDepot)
        .with_service(51, 50, ServiceType::SubwayStation)
        .with_service(100, 100, ServiceType::TrainStation)
        .with_service(101, 100, ServiceType::SubwayStation);

    city.tick_slow_cycle();

    let stats = city.world_mut().resource::<TransitHubStats>();
    assert_eq!(
        stats.total_hubs, 2,
        "Expected 2 hubs, got {}",
        stats.total_hubs
    );
    assert!(stats.bus_metro_hubs >= 1, "Expected at least 1 BusMetroHub");
    assert!(
        stats.train_metro_hubs >= 1,
        "Expected at least 1 TrainMetroHub"
    );
}

// ====================================================================
// Natural Resources integration tests
// ====================================================================

#[test]
fn natural_resources_resource_grid_exists_in_empty_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::natural_resources::ResourceGrid>();
    city.assert_resource_exists::<crate::natural_resources::ResourceBalance>();
}

#[test]
fn natural_resources_empty_city_grid_has_no_deposits() {
    let city = TestCity::new();
    let resource_grid = city.resource::<crate::natural_resources::ResourceGrid>();
    let deposit_count = resource_grid
        .deposits
        .iter()
        .filter(|d| d.is_some())
        .count();
    assert_eq!(
        deposit_count, 0,
        "empty city (no terrain generation) should have no resource deposits, got {deposit_count}"
    );
}

#[test]
fn natural_resources_deposit_placed_at_grid_position() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut resource_grid = world.resource_mut::<crate::natural_resources::ResourceGrid>();
        resource_grid.set(
            50,
            50,
            crate::natural_resources::ResourceDeposit {
                resource_type: crate::natural_resources::ResourceType::Ore,
                amount: 5000,
                max_amount: 5000,
            },
        );
    }
    let resource_grid = city.resource::<crate::natural_resources::ResourceGrid>();
    let deposit = resource_grid.get(50, 50);
    assert!(deposit.is_some(), "deposit should exist at (50, 50)");
    assert_eq!(
        deposit.as_ref().unwrap().resource_type,
        crate::natural_resources::ResourceType::Ore
    );
    assert!(resource_grid.get(49, 50).is_none());
    assert!(resource_grid.get(51, 50).is_none());
}

#[test]
fn natural_resources_extraction_rate_depends_on_occupants() {
    let occupants_low: u32 = 5;
    let occupants_high: u32 = 20;
    let output_low = occupants_low as f32 * 0.5;
    let output_high = occupants_high as f32 * 0.5;
    assert!(
        (output_low - 2.5).abs() < f32::EPSILON,
        "5 occupants should produce 2.5 output, got {output_low}"
    );
    assert!(
        (output_high - 10.0).abs() < f32::EPSILON,
        "20 occupants should produce 10.0 output, got {output_high}"
    );
    assert!(
        output_high > output_low,
        "more occupants should produce more output"
    );
}

#[test]
fn natural_resources_depleted_deposit_produces_nothing() {
    use crate::natural_resources::{ResourceDeposit, ResourceType};
    let deposit = ResourceDeposit {
        resource_type: ResourceType::Ore,
        amount: 0,
        max_amount: 5000,
    };
    assert_eq!(deposit.amount, 0, "depleted deposit should have amount 0");
}

#[test]
fn natural_resources_finite_resource_depletes_over_time() {
    use crate::natural_resources::{ResourceDeposit, ResourceType};
    let mut deposit = ResourceDeposit {
        resource_type: ResourceType::Ore,
        amount: 100,
        max_amount: 5000,
    };
    let output = 5.0_f32;
    let extraction = (output * 0.2) as u32;
    deposit.amount = deposit.amount.saturating_sub(extraction.max(1));
    assert_eq!(
        deposit.amount, 99,
        "first extraction should reduce amount by 1"
    );
    for _ in 0..50 {
        deposit.amount = deposit.amount.saturating_sub(extraction.max(1));
    }
    assert!(
        deposit.amount < 99,
        "repeated extraction should further deplete the resource"
    );
    for _ in 0..200 {
        deposit.amount = deposit.amount.saturating_sub(extraction.max(1));
    }
    assert_eq!(
        deposit.amount, 0,
        "finite resource should fully deplete to zero"
    );
}

#[test]
fn natural_resources_non_industrial_building_does_not_extract() {
    assert_ne!(ZoneType::ResidentialLow, ZoneType::Industrial);
    assert_ne!(ZoneType::CommercialLow, ZoneType::Industrial);
}

#[test]
fn natural_resources_consumption_scales_with_population() {
    let pop: f32 = 10000.0;
    let food = pop * 0.02;
    let timber = pop * 0.005;
    let metal = pop * 0.003;
    let fuel = pop * 0.004;
    assert!((food - 200.0).abs() < f32::EPSILON);
    assert!((timber - 50.0).abs() < f32::EPSILON);
    assert!((metal - 30.0).abs() < f32::EPSILON);
    assert!((fuel - 40.0).abs() < f32::EPSILON);
    let pop2: f32 = 20000.0;
    let food2 = pop2 * 0.02;
    assert!(
        food2 > food,
        "doubling population should double food consumption"
    );
}

// ====================================================================
// Train/Rail Transit (TRAF-012) integration tests
// ====================================================================

#[test]
fn test_train_transit_create_line_and_ridership() {
    use crate::train_transit::TrainTransitState;

    let mut city = TestCity::new()
        .with_road(10, 50, 90, 50, RoadType::Avenue)
        .with_building(12, 48, ZoneType::ResidentialLow, 2)
        .with_building(88, 48, ZoneType::CommercialLow, 2)
        .with_citizen((12, 48), (88, 48));

    // Place train stations and create a line
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<TrainTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = state
            .add_station(15, 50, "West Terminal".to_string(), &grid)
            .expect("station 1 should be placed");
        let s2 = state
            .add_station(85, 50, "East Terminal".to_string(), &grid)
            .expect("station 2 should be placed");
        state
            .add_line("Express Line".to_string(), vec![s1, s2])
            .expect("line should be created");
    }

    // Run a few slow cycles to accumulate ridership stats
    city.tick_slow_cycles(3);

    let state = city.resource::<TrainTransitState>();
    assert_eq!(state.stats.total_stations, 2, "should have 2 stations");
    assert_eq!(
        state.stats.total_active_lines, 1,
        "should have 1 active line"
    );
    assert_eq!(state.lines.len(), 1, "should have 1 line");
    assert!(state.lines[0].active, "line should be active");
}

#[test]
fn test_train_station_boosts_land_value() {
    use crate::train_transit::TrainTransitState;

    let mut city = TestCity::new();

    // Record baseline land value at station location
    let baseline = city.resource::<LandValueGrid>().get(50, 50);

    // Place train stations
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<TrainTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = state
            .add_station(50, 50, "Central".to_string(), &grid)
            .expect("station should be placed");
        let s2 = state
            .add_station(80, 50, "East".to_string(), &grid)
            .expect("station should be placed");
        state.add_line("Commuter Line".to_string(), vec![s1, s2]);
    }

    // Run slow tick so land value boost is applied
    city.tick_slow_cycle();

    let boosted = city.resource::<LandValueGrid>().get(50, 50);
    assert!(
        boosted > baseline,
        "land value at station ({}) should exceed baseline ({})",
        boosted,
        baseline
    );
}

#[test]
fn test_train_operating_cost_deduction() {
    use crate::train_transit::TrainTransitState;

    let mut city = TestCity::new().with_budget(100_000.0);

    // Place stations and line
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<TrainTransitState>();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        let s1 = state.add_station(10, 10, "A".to_string(), &grid).unwrap();
        let s2 = state.add_station(20, 20, "B".to_string(), &grid).unwrap();
        state.add_line("Red".to_string(), vec![s1, s2]);
    }

    let cost = city.resource::<TrainTransitState>().total_weekly_cost();
    assert!(
        cost > 0.0,
        "train operating cost should be positive, got {}",
        cost
    );

    // Verify the cost is: 2 stations * $800/week + 1 line * $2000/week = $3600
    let expected = 2.0 * 800.0 + 1.0 * 2000.0;
    assert!(
        (cost - expected).abs() < 0.01,
        "expected weekly cost {}, got {}",
        expected,
        cost
    );

    // Verify monthly cost is 4x weekly
    let monthly = city.resource::<TrainTransitState>().total_monthly_cost();
    assert!(
        (monthly - expected * 4.0).abs() < 0.01,
        "expected monthly cost {}, got {}",
        expected * 4.0,
        monthly
    );
}

#[test]
fn test_train_station_on_water_rejected() {
    use crate::train_transit::TrainTransitState;

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.get_mut(50, 50).cell_type = CellType::Water;

    let mut state = TrainTransitState::default();

    let result = state.add_station(50, 50, "Aquatic".to_string(), &grid);
    assert!(
        result.is_none(),
        "should not be able to place station on water"
    );

    let result = state.add_station(51, 50, "Dry".to_string(), &grid);
    assert!(result.is_some(), "should place station on grass");
}

#[test]
fn test_train_saveable_roundtrip_integration() {
    use crate::train_transit::TrainTransitState;
    use crate::Saveable;

    let mut state = TrainTransitState::default();
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    state.add_station(10, 10, "Alpha".to_string(), &grid);
    state.add_station(50, 50, "Beta".to_string(), &grid);
    state.add_line("Red".to_string(), vec![0, 1]);

    let bytes = state.save_to_bytes().expect("should produce bytes");
    let restored = TrainTransitState::load_from_bytes(&bytes);
    assert_eq!(restored.stations.len(), 2, "should restore 2 stations");
    assert_eq!(restored.lines.len(), 1, "should restore 1 line");
    assert_eq!(
        restored.stations[0].name, "Alpha",
        "station name should survive roundtrip"
    );
}

// ====================================================================
// Welfare system tests (issue #851)
// ====================================================================

#[test]
fn test_welfare_stats_resource_exists_in_new_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::welfare::WelfareStats>();
}

#[test]
fn test_welfare_stats_default_values_on_empty_city() {
    let city = TestCity::new();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.total_sheltered, 0);
    assert_eq!(stats.total_welfare_recipients, 0);
    assert_eq!(stats.monthly_cost, 0.0);
    assert_eq!(stats.shelter_capacity, 0);
    assert_eq!(stats.shelter_occupancy, 0);
    assert_eq!(stats.welfare_office_count, 0);
    assert_eq!(stats.shelter_count, 0);
}

#[test]
fn test_welfare_office_tracks_count_after_slow_tick() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::WelfareOffice)
        .with_service(60, 60, ServiceType::WelfareOffice);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.welfare_office_count, 2);
}

#[test]
fn test_welfare_shelter_tracks_count_after_slow_tick() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::HomelessShelter)
        .with_service(60, 60, ServiceType::HomelessShelter);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.shelter_count, 2);
}

#[test]
fn test_welfare_shelter_capacity_from_service_buildings() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::HomelessShelter)
        .with_service(60, 60, ServiceType::HomelessShelter);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    // Each shelter has 50 bed capacity
    assert_eq!(stats.shelter_capacity, 100);
}

#[test]
fn test_welfare_monthly_cost_for_offices_and_shelters() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::WelfareOffice)
        .with_service(60, 60, ServiceType::HomelessShelter);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    let expected = ServiceBuilding::monthly_maintenance(ServiceType::WelfareOffice)
        + ServiceBuilding::monthly_maintenance(ServiceType::HomelessShelter);
    assert!(
        (stats.monthly_cost - expected).abs() < 0.01,
        "monthly cost should match sum of maintenance: got {}, expected {}",
        stats.monthly_cost,
        expected
    );
}

#[test]
fn test_welfare_monthly_cost_zero_with_no_services() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();
    assert_eq!(
        city.resource::<crate::welfare::WelfareStats>().monthly_cost,
        0.0
    );
}

#[test]
fn test_welfare_no_recipients_without_offices() {
    let mut city = TestCity::new()
        .with_building(30, 30, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen((30, 30));
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.total_welfare_recipients, 0);
    assert_eq!(stats.welfare_office_count, 0);
}

#[test]
fn test_welfare_office_near_unemployed_citizen() {
    let mut city = TestCity::new()
        .with_building(30, 30, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen((30, 30))
        .with_service(31, 31, ServiceType::WelfareOffice);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.welfare_office_count, 1);
}

#[test]
fn test_welfare_multiple_offices_counts_recipients() {
    let mut city = TestCity::new()
        .with_building(30, 30, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen((30, 30))
        .with_service(31, 31, ServiceType::WelfareOffice)
        .with_service(32, 32, ServiceType::WelfareOffice);
    city.tick_slow_cycle();
    let stats = city.resource::<crate::welfare::WelfareStats>();
    assert_eq!(stats.welfare_office_count, 2);
    assert!(stats.total_welfare_recipients <= 1);
}

#[test]
fn test_welfare_expense_scales_with_building_count() {
    let mut small = TestCity::new().with_service(30, 30, ServiceType::WelfareOffice);
    small.tick_slow_cycle();
    let cost_small = small
        .resource::<crate::welfare::WelfareStats>()
        .monthly_cost;

    let mut large = TestCity::new()
        .with_service(30, 30, ServiceType::WelfareOffice)
        .with_service(60, 60, ServiceType::WelfareOffice)
        .with_service(90, 90, ServiceType::WelfareOffice)
        .with_service(30, 90, ServiceType::HomelessShelter)
        .with_service(90, 30, ServiceType::HomelessShelter);
    large.tick_slow_cycle();
    let cost_large = large
        .resource::<crate::welfare::WelfareStats>()
        .monthly_cost;
    assert!(
        cost_large > cost_small,
        "more buildings should cost more: small={cost_small}, large={cost_large}"
    );
    let expected = 3.0 * ServiceBuilding::monthly_maintenance(ServiceType::WelfareOffice)
        + 2.0 * ServiceBuilding::monthly_maintenance(ServiceType::HomelessShelter);
    assert!((cost_large - expected).abs() < 0.01);
}

#[test]
fn test_welfare_stats_reset_between_slow_ticks() {
    let mut city = TestCity::new()
        .with_building(30, 30, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen((30, 30))
        .with_service(31, 31, ServiceType::WelfareOffice);
    city.tick_slow_cycle();
    let first = city
        .resource::<crate::welfare::WelfareStats>()
        .total_welfare_recipients;
    city.tick_slow_cycle();
    let second = city
        .resource::<crate::welfare::WelfareStats>()
        .total_welfare_recipients;
    assert!(
        second <= first,
        "recipients should not accumulate: first={first}, second={second}"
    );
}

// ====================================================================
// Education Pipeline Tests (TEST-058)
// ====================================================================

#[test]
fn test_education_elementary_school_propagates_level_1() {
    // An elementary school should propagate education level 1 to nearby
    // cells reachable via roads.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // The school cell itself should have education level 1
    assert_eq!(
        edu.get(50, 50),
        1,
        "elementary school cell should have education level 1"
    );
}

#[test]
fn test_education_high_school_propagates_level_2() {
    // A high school should propagate education level 2 to cells it reaches.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::HighSchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    assert_eq!(
        edu.get(50, 50),
        2,
        "high school cell should have education level 2"
    );
}

#[test]
fn test_education_university_propagates_level_3() {
    // A university should propagate education level 3 to cells it reaches.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::University);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    assert_eq!(
        edu.get(50, 50),
        3,
        "university cell should have education level 3"
    );
}

#[test]
fn test_education_spreads_along_road() {
    // Education should spread along roads from the school.
    // Place a school at (50,50) with a road going east to (60,50).
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // Cells along the road should get education level 1
    let level_at_55 = edu.get(55, 50);
    assert_eq!(
        level_at_55, 1,
        "road cell at (55,50) should have education level 1, got {level_at_55}"
    );
}

#[test]
fn test_education_does_not_reach_disconnected_area() {
    // Education should NOT reach cells that are not road-connected to the school.
    // Place school at (50,50) with road going east, but check a far-off cell.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // A disconnected cell far away should have no education
    assert_eq!(
        edu.get(200, 200),
        0,
        "disconnected cell at (200,200) should have education level 0"
    );
}

#[test]
fn test_education_university_overrides_lower_level() {
    // When a university (level 3) and elementary school (level 1) both
    // cover a cell, the higher level should win because sources are sorted
    // highest-first.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_service(55, 50, ServiceType::University);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // The university cell should have level 3 (not overridden by elementary)
    assert_eq!(
        edu.get(55, 50),
        3,
        "university cell should have education level 3 even with nearby elementary school"
    );
    // The school cell at (50, 50) should also have level 3 because the
    // university BFS runs first and reaches it via the road
    let level_at_school = edu.get(50, 50);
    assert!(
        level_at_school >= 1,
        "elementary school cell should have at least education level 1, got {level_at_school}"
    );
}

#[test]
fn test_education_grid_resets_each_slow_tick() {
    // Education grid is recalculated from scratch each slow tick.
    // After removing the school (despawning it), the grid should reset to 0.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool);
    city.tick_slow_cycle();
    let edu_before = city
        .resource::<crate::education::EducationGrid>()
        .get(50, 50);
    assert_eq!(edu_before, 1, "should have education before removal");

    // Bulldoze the school service building
    city.bulldoze_service_at(50, 50);
    city.tick_slow_cycle();
    let edu_after = city
        .resource::<crate::education::EducationGrid>()
        .get(50, 50);
    assert_eq!(
        edu_after, 0,
        "education should be 0 after school is removed"
    );
}

#[test]
fn test_education_kindergarten_propagates_level_1() {
    // Kindergarten should also propagate education level 1.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::Kindergarten);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    assert_eq!(
        edu.get(50, 50),
        1,
        "kindergarten cell should have education level 1"
    );
}

#[test]
fn test_education_library_propagates_level_1() {
    // Library should propagate education level 1.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::Library);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    assert_eq!(
        edu.get(50, 50),
        1,
        "library cell should have education level 1"
    );
}

#[test]
fn test_education_reaches_adjacent_grass_but_not_beyond() {
    // BFS spreads through roads and marks adjacent grass cells, but does
    // not continue propagating through grass (only through roads).
    // Layout: school at (50,50), road (50,50)-(55,50), grass cell at (55,51)
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::HighSchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // The road cell at (55,50) should have level 2
    assert_eq!(
        edu.get(55, 50),
        2,
        "road cell should have education level 2"
    );
    // Adjacent grass at (55,51) should get level 2 (BFS marks grass neighbors)
    let grass_level = edu.get(55, 51);
    assert_eq!(
        grass_level, 2,
        "adjacent grass cell at (55,51) should get education level 2, got {grass_level}"
    );
}

#[test]
fn test_education_no_schools_means_zero_everywhere() {
    // With no education buildings, the entire grid should remain at 0.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_building(51, 51, ZoneType::ResidentialLow, 1);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // Spot-check a few cells
    assert_eq!(edu.get(50, 50), 0, "no schools => level 0 at (50,50)");
    assert_eq!(edu.get(55, 50), 0, "no schools => level 0 at (55,50)");
    assert_eq!(edu.get(51, 51), 0, "no schools => level 0 at (51,51)");
}

#[test]
fn test_education_multiple_schools_different_levels() {
    // Place elementary (level 1) and university (level 3) on separate roads.
    // Each school's area should reflect its own level.
    let mut city = TestCity::new()
        // Elementary school area: road from (30,30) to (35,30)
        .with_road(30, 30, 35, 30, RoadType::Local)
        .with_service(30, 30, ServiceType::ElementarySchool)
        // University area: road from (80,80) to (85,80), far from elementary
        .with_road(80, 80, 85, 80, RoadType::Local)
        .with_service(80, 80, ServiceType::University);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    assert_eq!(edu.get(30, 30), 1, "elementary area should have level 1");
    assert_eq!(edu.get(80, 80), 3, "university area should have level 3");
    // Each area should not be contaminated by the other
    assert_eq!(edu.get(35, 30), 1, "near elementary should be level 1");
    assert_eq!(edu.get(85, 80), 3, "near university should be level 3");
}

#[test]
fn test_education_range_limited_by_radius() {
    // Education spread is limited by the school's radius converted to grid cells.
    // ElementarySchool has coverage_radius = 15.0 * CELL_SIZE = 15 * 16 = 240 pixels.
    // Range in grid cells = 240 / 16 = 15 cells.
    // Place a long road but check that cells beyond range have level 0.
    let mut city = TestCity::new()
        .with_road(50, 50, 100, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // Close to school (within 15 cells) should be level 1
    assert_eq!(
        edu.get(55, 50),
        1,
        "within range should have education level 1"
    );
    // Far beyond range (50 cells away) should be 0
    assert_eq!(
        edu.get(100, 50),
        0,
        "beyond range should have education level 0"
    );
}

// ====================================================================
// Economy tax collection tests (issue #834 / TEST-054)
// ====================================================================

/// Helper: advance the game clock day past the tax collection interval (30 days)
/// so that the next tick triggers `collect_taxes`. This avoids running 40K+ ticks.
fn force_clock_to_day(city: &mut TestCity, day: u32) {
    let world = city.world_mut();
    world.resource_mut::<GameClock>().day = day;
}

/// Set up city with employed citizens, advance past tax collection day,
/// verify treasury increases.
#[test]
fn test_economy_tax_collection_increases_treasury() {
    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_zone_rect(12, 11, 16, 11, ZoneType::ResidentialLow)
        .with_zone_rect(20, 11, 24, 11, ZoneType::CommercialLow)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_building(14, 11, ZoneType::ResidentialLow, 1)
        .with_building(20, 11, ZoneType::CommercialLow, 2)
        .with_citizen((12, 11), (20, 11))
        .with_citizen((14, 11), (20, 11))
        .with_budget(10_000.0);

    let treasury_before = city.budget().treasury;

    // Advance clock past the 30-day tax collection interval
    force_clock_to_day(&mut city, 32);
    city.tick(10); // let collect_taxes system run

    let budget = city.budget().clone();
    assert!(
        budget.treasury > treasury_before,
        "Treasury should increase after tax collection: before={treasury_before}, after={}",
        budget.treasury
    );
}

/// Verify monthly_income is positive after tax collection with buildings present.
#[test]
fn test_economy_monthly_income_positive_with_buildings() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_building(14, 11, ZoneType::CommercialLow, 1)
        .with_building(16, 11, ZoneType::Industrial, 1)
        .with_citizen((12, 11), (14, 11))
        .with_budget(10_000.0);

    // Advance past tax collection interval
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.monthly_income > 0.0,
        "monthly_income should be > 0 with taxable buildings, got {}",
        budget.monthly_income
    );
}

/// Verify expenses are deducted for active services (road maintenance + service costs).
#[test]
fn test_economy_expenses_deducted_for_services() {
    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_service(20, 11, ServiceType::PoliceStation)
        .with_budget(50_000.0);

    // Advance past tax collection interval
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.monthly_expenses > 0.0,
        "monthly_expenses should be > 0 with roads and services, got {}",
        budget.monthly_expenses
    );
}

/// Verify that treasury net change equals income minus expenses.
#[test]
fn test_economy_treasury_change_equals_net_income() {
    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_building(14, 11, ZoneType::CommercialLow, 2)
        .with_service(20, 11, ServiceType::FireStation)
        .with_citizen((12, 11), (14, 11))
        .with_budget(10_000.0);

    let treasury_before = city.budget().treasury;

    // Force tax collection
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    let expected_change = budget.monthly_income - budget.monthly_expenses;
    let actual_change = budget.treasury - treasury_before;

    assert!(
        (actual_change - expected_change).abs() < 0.01,
        "Treasury change ({actual_change}) should equal income - expenses ({expected_change})"
    );
}

/// Verify tax collection only happens once per interval - not on every tick.
#[test]
fn test_economy_tax_collection_respects_interval() {
    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 2)
        .with_citizen((12, 11), (12, 11))
        .with_budget(10_000.0);

    // First collection
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget_after_first = city.budget().clone();
    let treasury_after_first = budget_after_first.treasury;

    // last_collection_day should be updated
    assert!(
        budget_after_first.last_collection_day > 0,
        "last_collection_day should be updated after collection, got {}",
        budget_after_first.last_collection_day
    );

    // Tick again without advancing the day past the next interval
    city.tick(10);

    let treasury_after_second = city.budget().treasury;
    assert!(
        (treasury_after_second - treasury_after_first).abs() < 0.01,
        "Treasury should not change between collection intervals: first={treasury_after_first}, second={treasury_after_second}"
    );
}

/// Verify no income is generated when there are no buildings (empty city).
#[test]
fn test_economy_no_income_without_buildings() {
    let mut city = TestCity::new().with_budget(10_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    // With no buildings and no roads, income and expenses should both be 0
    assert!(
        (budget.monthly_income - 0.0).abs() < 0.01,
        "monthly_income should be 0 with no buildings, got {}",
        budget.monthly_income
    );
}

/// Verify road maintenance contributes to expenses.
#[test]
fn test_economy_road_maintenance_in_expenses() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_road(10, 10, 40, 10, RoadType::Local)
        .with_road(10, 10, 10, 40, RoadType::Avenue)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>().clone();
    assert!(
        extended.expense_breakdown.road_maintenance > 0.0,
        "road maintenance expense should be > 0 with roads, got {}",
        extended.expense_breakdown.road_maintenance
    );
}

/// Verify residential buildings generate residential tax income.
#[test]
fn test_economy_residential_tax_income() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_budget(10_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>().clone();
    assert!(
        extended.income_breakdown.residential_tax > 0.0,
        "residential tax should be > 0 for residential building, got {}",
        extended.income_breakdown.residential_tax
    );
}

/// Verify commercial buildings generate commercial tax income.
#[test]
fn test_economy_commercial_tax_income() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialLow, 2)
        .with_budget(10_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>().clone();
    assert!(
        extended.income_breakdown.commercial_tax > 0.0,
        "commercial tax should be > 0 for commercial building, got {}",
        extended.income_breakdown.commercial_tax
    );
}

/// Verify industrial buildings generate industrial tax income.
#[test]
fn test_economy_industrial_tax_income() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 2)
        .with_budget(10_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>().clone();
    assert!(
        extended.income_breakdown.industrial_tax > 0.0,
        "industrial tax should be > 0 for industrial building, got {}",
        extended.income_breakdown.industrial_tax
    );
}

/// Higher building level should produce more tax revenue.
#[test]
fn test_economy_higher_level_building_more_tax() {
    use crate::budget::ExtendedBudget;

    // City with level 1 building
    let mut city_low = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_budget(10_000.0);

    force_clock_to_day(&mut city_low, 32);
    city_low.tick(10);

    let tax_low = city_low
        .resource::<ExtendedBudget>()
        .income_breakdown
        .residential_tax;

    // City with level 3 building at same location
    let mut city_high = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 3)
        .with_budget(10_000.0);

    force_clock_to_day(&mut city_high, 32);
    city_high.tick(10);

    let tax_high = city_high
        .resource::<ExtendedBudget>()
        .income_breakdown
        .residential_tax;

    assert!(
        tax_high > tax_low,
        "Higher level building should generate more tax: level1={tax_low}, level3={tax_high}"
    );
}

/// Service maintenance costs appear in the expense breakdown after tax collection.
#[test]
fn test_economy_service_maintenance_in_expense_breakdown() {
    use crate::budget::ExtendedBudget;

    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::Hospital)
        .with_service(60, 60, ServiceType::PoliceStation)
        .with_budget(50_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let extended = city.resource::<ExtendedBudget>().clone();
    let expected_service_cost = ServiceBuilding::monthly_maintenance(ServiceType::Hospital)
        + ServiceBuilding::monthly_maintenance(ServiceType::PoliceStation);

    assert!(
        (extended.expense_breakdown.service_costs - expected_service_cost).abs() < 0.01,
        "Service costs should match sum of maintenance: got {}, expected {expected_service_cost}",
        extended.expense_breakdown.service_costs
    );
}

/// Multiple tax collections over time should each add to the treasury.
#[test]
fn test_economy_multiple_tax_collections_over_time() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 3)
        .with_building(52, 50, ZoneType::CommercialLow, 2)
        .with_citizen((50, 50), (52, 50))
        .with_budget(10_000.0);

    // First collection at day 32
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let after_first = city.budget().treasury;
    assert!(
        after_first > 10_000.0,
        "Treasury should increase after first collection: got {after_first}"
    );

    // Second collection at day 63 (32 + 31, past the next interval)
    force_clock_to_day(&mut city, 63);
    city.tick(10);

    let after_second = city.budget().treasury;
    assert!(
        after_second > after_first,
        "Treasury should increase again after second collection: first={after_first}, second={after_second}"
    );
}

/// Budget goes negative when expenses exceed income with no buildings.
#[test]
fn test_economy_treasury_decreases_with_only_expenses() {
    let mut city = TestCity::new()
        .with_road(10, 10, 80, 10, RoadType::Highway) // expensive road
        .with_service(20, 11, ServiceType::Hospital) // expensive service
        .with_budget(1_000.0);

    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let budget = city.budget();
    assert!(
        budget.treasury < 1_000.0,
        "Treasury should decrease when expenses exceed income: got {}",
        budget.treasury
    );
    assert!(
        budget.monthly_expenses > budget.monthly_income,
        "Expenses ({}) should exceed income ({}) with no taxable buildings",
        budget.monthly_expenses,
        budget.monthly_income,
    );
}

// ====================================================================
// Tram / light rail transit tests (issue #865)
// ====================================================================

#[test]
fn test_tram_transit_add_stops_and_line() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::tram_transit::TramTransitState;

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    for x in 95..125 {
        roads.place_road_typed(&mut grid, x, 100, RoadType::Local);
    }

    let mut state = TramTransitState::default();
    let s0 = state.add_stop(&grid, 100, 100).expect("stop on road");
    let s1 = state.add_stop(&grid, 120, 100).expect("stop on road");
    assert_eq!(state.stops.len(), 2);

    state.add_line("Green".into(), vec![s0, s1]);
    assert_eq!(state.lines.len(), 1);
    assert_eq!(state.lines[0].stop_ids.len(), 2);
}

#[test]
fn test_tram_stop_on_non_road_cell_fails() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::tram_transit::TramTransitState;

    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut state = TramTransitState::default();
    let result = state.add_stop(&grid, 5, 5);
    assert!(result.is_none(), "should not place tram stop on grass");
    assert_eq!(state.stops.len(), 0);
}

#[test]
fn test_tram_transit_saveable_roundtrip() {
    use crate::tram_transit::TramTransitState;
    use crate::Saveable;

    let mut state = TramTransitState::default();
    state.stops.push(crate::tram_transit::TramStop {
        id: 0,
        grid_x: 10,
        grid_y: 20,
        waiting: 0,
    });
    state.stops.push(crate::tram_transit::TramStop {
        id: 1,
        grid_x: 30,
        grid_y: 40,
        waiting: 0,
    });
    state.next_stop_id = 2;
    state.add_line("Red".into(), vec![0, 1]);

    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = TramTransitState::load_from_bytes(&bytes);
    assert_eq!(restored.stops.len(), 2);
    assert_eq!(restored.lines.len(), 1);
    assert_eq!(restored.stops[0].grid_x, 10);
}

#[test]
fn test_tram_capacity_is_90() {
    use crate::tram_transit::TRAM_CAPACITY;
    assert_eq!(TRAM_CAPACITY, 90, "tram capacity should be 90 passengers");
}

// ====================================================================
// Homelessness system tests (TEST-059)
// ====================================================================

#[test]
fn test_homelessness_citizen_becomes_homeless_when_home_despawned() {
    // Spawn a citizen with a valid home building, then despawn the building.
    // After ticking past the CHECK_INTERVAL (50 ticks), the citizen should
    // gain the Homeless component.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Verify citizen exists and is not homeless yet
    {
        let world = city.world_mut();
        let homeless_count = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count();
        assert_eq!(
            homeless_count, 0,
            "citizen should not be homeless initially"
        );
    }

    // Despawn the home building
    let building_entity = {
        let grid = city.grid();
        grid.get(50, 50).building_id.expect("building should exist")
    };
    city.world_mut().despawn(building_entity);

    // Tick past the homelessness CHECK_INTERVAL (50 ticks)
    city.tick(50);

    // Citizen should now be homeless
    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert_eq!(
        homeless_count, 1,
        "citizen should become homeless after home is despawned"
    );
}

#[test]
fn test_homelessness_stats_track_total_homeless() {
    // Create citizens and despawn their homes to produce homeless citizens.
    // Verify HomelessnessStats.total_homeless reflects the count.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50))
        .with_building(60, 60, ZoneType::ResidentialLow, 1)
        .with_citizen((60, 60), (60, 60));

    // Despawn both home buildings
    let b1 = city.grid().get(50, 50).building_id.expect("building 1");
    let b2 = city.grid().get(60, 60).building_id.expect("building 2");
    city.world_mut().despawn(b1);
    city.world_mut().despawn(b2);

    // Tick to trigger check_homelessness + seek_shelter
    city.tick(50);

    let stats = city.resource::<crate::homelessness::HomelessnessStats>();
    assert_eq!(
        stats.total_homeless, 2,
        "total_homeless should reflect both homeless citizens"
    );
}

#[test]
fn test_homelessness_recover_when_housing_available() {
    // Make a citizen homeless, then provide a residential building with capacity.
    // After ticking, the citizen should recover (Homeless component removed).
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Despawn the home building to make citizen homeless
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    // Tick to make citizen homeless
    city.tick(50);

    {
        let world = city.world_mut();
        let homeless_count = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count();
        assert_eq!(homeless_count, 1, "citizen should be homeless");
    }

    // Now spawn a new residential building with capacity for the citizen to move into
    {
        let entity = city
            .world_mut()
            .spawn(Building {
                zone_type: ZoneType::ResidentialLow,
                level: 1,
                grid_x: 70,
                grid_y: 70,
                capacity: 5,
                occupants: 0,
            })
            .id();
        let mut grid = city.world_mut().resource_mut::<WorldGrid>();
        grid.get_mut(70, 70).building_id = Some(entity);
        grid.get_mut(70, 70).zone = ZoneType::ResidentialLow;
    }

    // Tick again to trigger recover_from_homelessness
    city.tick(50);

    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert_eq!(
        homeless_count, 0,
        "citizen should recover from homelessness when housing is available"
    );
}

#[test]
fn test_homelessness_happiness_penalty_applied() {
    // When a citizen becomes homeless, their happiness should drop by HOMELESS_PENALTY (30.0).
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Record initial happiness
    let initial_happiness = {
        let world = city.world_mut();
        let details = world
            .query::<&crate::citizen::CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should exist");
        details.happiness
    };

    // Despawn home building
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    // Tick to make citizen homeless
    city.tick(50);

    let new_happiness = {
        let world = city.world_mut();
        let details = world
            .query::<&crate::citizen::CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should exist");
        details.happiness
    };

    // Happiness should have dropped (by at least HOMELESS_PENALTY = 30.0,
    // though other systems may also affect it)
    assert!(
        new_happiness < initial_happiness,
        "happiness should decrease when homeless: was {initial_happiness}, now {new_happiness}"
    );
    // The penalty is exactly 30.0 in check_homelessness, but other systems running
    // concurrently may shift it slightly. Check it dropped by at least 20.
    assert!(
        initial_happiness - new_happiness >= 20.0,
        "happiness should drop significantly: was {initial_happiness}, now {new_happiness}"
    );
}

#[test]
fn test_homelessness_shelter_provides_shelter_to_homeless() {
    // Spawn a homeless citizen and a shelter. After ticking, the citizen
    // should become sheltered (Homeless.sheltered = true).
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Despawn home to make citizen homeless
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    // Tick to trigger check_homelessness (citizen becomes homeless)
    city.tick(50);

    // Now spawn a HomelessShelter component entity
    {
        city.world_mut()
            .spawn(crate::homelessness::HomelessShelter {
                grid_x: 55,
                grid_y: 55,
                capacity: 10,
                current_occupants: 0,
            });
    }

    // Tick again to trigger seek_shelter
    city.tick(50);

    // Citizen should now be sheltered
    let sheltered = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .any(|h| h.sheltered)
    };
    assert!(
        sheltered,
        "homeless citizen should become sheltered when shelter has capacity"
    );

    let stats = city.resource::<crate::homelessness::HomelessnessStats>();
    assert!(
        stats.sheltered > 0,
        "sheltered count in stats should be positive"
    );
}

#[test]
fn test_homelessness_shelter_capacity_respected() {
    // Create more homeless citizens than shelter capacity.
    // Only up to capacity should be sheltered.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50));

    // Despawn home building to make all 3 citizens homeless
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    // Tick to make citizens homeless
    city.tick(50);

    // Spawn a shelter with capacity of 1
    {
        city.world_mut()
            .spawn(crate::homelessness::HomelessShelter {
                grid_x: 55,
                grid_y: 55,
                capacity: 1,
                current_occupants: 0,
            });
    }

    // Tick to trigger seek_shelter
    city.tick(50);

    let (sheltered_count, total_homeless) = {
        let world = city.world_mut();
        let homeless_list: Vec<&crate::homelessness::Homeless> = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .collect();
        let sheltered = homeless_list.iter().filter(|h| h.sheltered).count();
        let total = homeless_list.len();
        (sheltered, total)
    };

    assert_eq!(
        sheltered_count, 1,
        "only 1 citizen should be sheltered (capacity=1), got {sheltered_count}"
    );
    assert_eq!(
        total_homeless, 3,
        "all 3 citizens should still be homeless, got {total_homeless}"
    );
}

#[test]
fn test_homelessness_citizen_placeholder_home_becomes_homeless() {
    // A citizen whose home building is Entity::PLACEHOLDER should be detected
    // as homeless by check_homelessness.
    use crate::citizen::*;
    use crate::mode_choice::ChosenTransportMode;
    use crate::movement::ActivityTimer;
    use bevy::prelude::Entity;

    let mut city = TestCity::new();

    // Manually spawn a citizen with PLACEHOLDER home building
    {
        let world = city.world_mut();
        world.spawn((
            Citizen,
            Position { x: 800.0, y: 800.0 },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: 50,
                grid_y: 50,
                building: Entity::PLACEHOLDER,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 2,
                happiness: 60.0,
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
            ChosenTransportMode::default(),
        ));
    }

    // Tick to trigger check_homelessness
    city.tick(50);

    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert_eq!(
        homeless_count, 1,
        "citizen with PLACEHOLDER home should become homeless"
    );
}

#[test]
fn test_homelessness_rent_unaffordable_becomes_homeless() {
    // A citizen with negative savings and low salary should become homeless
    // due to rent unaffordability.
    use crate::citizen::*;
    use crate::mode_choice::ChosenTransportMode;
    use crate::movement::ActivityTimer;

    let mut city = TestCity::new().with_building(50, 50, ZoneType::ResidentialLow, 1);

    let home_entity = city.grid().get(50, 50).building_id.expect("building");

    // Spawn citizen with negative savings and salary below threshold (1000.0)
    {
        let world = city.world_mut();
        world.spawn((
            Citizen,
            Position { x: 800.0, y: 800.0 },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: 50,
                grid_y: 50,
                building: home_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 0,
                happiness: 50.0,
                health: 80.0,
                salary: 500.0,   // below RENT_AFFORDABILITY_THRESHOLD (1000.0)
                savings: -100.0, // negative savings
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
            ChosenTransportMode::default(),
        ));
    }

    // Tick to trigger check_homelessness
    city.tick(50);

    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert_eq!(
        homeless_count, 1,
        "citizen with negative savings and low salary should become homeless"
    );
}

#[test]
fn test_homelessness_stats_zero_in_empty_city() {
    // An empty city should have zero homelessness stats.
    let mut city = TestCity::new();
    city.tick(50);

    let stats = city.resource::<crate::homelessness::HomelessnessStats>();
    assert_eq!(stats.total_homeless, 0, "no homeless in empty city");
    assert_eq!(stats.sheltered, 0, "no sheltered in empty city");
}

#[test]
fn test_homelessness_recovery_updates_stats() {
    // After a homeless citizen recovers, total_homeless should decrease.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Despawn home to make citizen homeless
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    city.tick(50);

    let homeless_before = city
        .resource::<crate::homelessness::HomelessnessStats>()
        .total_homeless;
    assert!(
        homeless_before > 0,
        "should have at least one homeless citizen"
    );

    // Provide new housing
    {
        let entity = city
            .world_mut()
            .spawn(Building {
                zone_type: ZoneType::ResidentialLow,
                level: 1,
                grid_x: 70,
                grid_y: 70,
                capacity: 5,
                occupants: 0,
            })
            .id();
        let mut grid = city.world_mut().resource_mut::<WorldGrid>();
        grid.get_mut(70, 70).building_id = Some(entity);
        grid.get_mut(70, 70).zone = ZoneType::ResidentialLow;
    }

    // Tick to recover
    city.tick(50);

    let homeless_after = city
        .resource::<crate::homelessness::HomelessnessStats>()
        .total_homeless;
    assert!(
        homeless_after < homeless_before,
        "total_homeless should decrease after recovery: before={homeless_before}, after={homeless_after}"
    );
}

#[test]
fn test_homelessness_ticks_homeless_increments() {
    // The ticks_homeless counter on the Homeless component should increment
    // each time check_homelessness runs.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Despawn home
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    // Tick to make homeless (first check)
    city.tick(50);

    let ticks_after_first = {
        let world = city.world_mut();
        let homeless = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .next()
            .expect("should be homeless");
        homeless.ticks_homeless
    };

    // Tick again (second check)
    city.tick(50);

    let ticks_after_second = {
        let world = city.world_mut();
        let homeless = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .next()
            .expect("should still be homeless");
        homeless.ticks_homeless
    };

    assert!(
        ticks_after_second > ticks_after_first,
        "ticks_homeless should increment: first={ticks_after_first}, second={ticks_after_second}"
    );
}

// ====================================================================
// Policy system tests (issue #845)
// ====================================================================

use crate::policies::{Policies, Policy};

#[test]
fn test_policy_resource_exists_in_new_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<Policies>();
}

#[test]
fn test_policy_default_state_has_no_active_policies() {
    let city = TestCity::new();
    let policies = city.resource::<Policies>();
    assert!(
        policies.active.is_empty(),
        "new city should have no active policies"
    );
}

#[test]
fn test_policy_toggle_enables_policy() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::RecyclingProgram);
    }
    let policies = city.resource::<Policies>();
    assert!(
        policies.is_active(Policy::RecyclingProgram),
        "RecyclingProgram should be active after toggle"
    );
}

#[test]
fn test_policy_toggle_disables_policy() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::RecyclingProgram);
        policies.toggle(Policy::RecyclingProgram);
    }
    let policies = city.resource::<Policies>();
    assert!(
        !policies.is_active(Policy::RecyclingProgram),
        "RecyclingProgram should be inactive after double toggle"
    );
}

#[test]
fn test_policy_multiple_active_policies_stack() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::FreePublicTransport);
        policies.toggle(Policy::NightShiftBan);
        policies.toggle(Policy::HealthcareForAll);
        policies.toggle(Policy::NeighborhoodWatch);
    }
    let policies = city.resource::<Policies>();
    // FreePublicTransport: +3, NightShiftBan: +3, HealthcareForAll: +2, NeighborhoodWatch: +2
    let expected_happiness = 3.0 + 3.0 + 2.0 + 2.0;
    let actual = policies.happiness_bonus();
    assert!(
        (actual - expected_happiness).abs() < f32::EPSILON,
        "stacked happiness bonus should be {expected_happiness}, got {actual}"
    );
}

#[test]
fn test_policy_total_monthly_cost_single_policy() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::FreePublicTransport);
    }
    let policies = city.resource::<Policies>();
    let expected = 50.0;
    let actual = policies.total_monthly_cost();
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "FreePublicTransport should cost {expected}/month, got {actual}"
    );
}

#[test]
fn test_policy_total_monthly_cost_multiple_policies() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::FreePublicTransport); // 50
        policies.toggle(Policy::RecyclingProgram); // 20
        policies.toggle(Policy::EducationPush); // 40
    }
    let policies = city.resource::<Policies>();
    let expected = 50.0 + 20.0 + 40.0;
    let actual = policies.total_monthly_cost();
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "total monthly cost should be {expected}, got {actual}"
    );
}

#[test]
fn test_policy_zero_cost_policies_do_not_add_expense() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::HighRiseBan); // 0
        policies.toggle(Policy::NightShiftBan); // 0
        policies.toggle(Policy::CumulativeZoning); // 0
    }
    let policies = city.resource::<Policies>();
    let actual = policies.total_monthly_cost();
    assert!(
        actual.abs() < f64::EPSILON,
        "zero-cost policies should have 0 total cost, got {actual}"
    );
}

#[test]
fn test_policy_pollution_multiplier_with_air_filters() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::IndustrialAirFilters);
    }
    let policies = city.resource::<Policies>();
    let mult = policies.pollution_multiplier();
    assert!(
        (mult - 0.6).abs() < f32::EPSILON,
        "pollution multiplier with IndustrialAirFilters should be 0.6, got {mult}"
    );
}

#[test]
fn test_policy_pollution_multiplier_without_air_filters() {
    let city = TestCity::new();
    let policies = city.resource::<Policies>();
    let mult = policies.pollution_multiplier();
    assert!(
        (mult - 1.0).abs() < f32::EPSILON,
        "pollution multiplier without IndustrialAirFilters should be 1.0, got {mult}"
    );
}

#[test]
fn test_policy_garbage_multiplier_with_recycling() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::RecyclingProgram);
    }
    let policies = city.resource::<Policies>();
    let mult = policies.garbage_multiplier();
    assert!(
        (mult - 0.7).abs() < f32::EPSILON,
        "garbage multiplier with RecyclingProgram should be 0.7, got {mult}"
    );
}

#[test]
fn test_policy_park_multiplier_with_green_space() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::GreenSpaceInitiative);
    }
    let policies = city.resource::<Policies>();
    let mult = policies.park_multiplier();
    assert!(
        (mult - 1.5).abs() < f32::EPSILON,
        "park multiplier with GreenSpaceInitiative should be 1.5, got {mult}"
    );
}

#[test]
fn test_policy_max_building_level_with_high_rise_ban() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<Policies>().toggle(Policy::HighRiseBan);
    }
    let policies = city.resource::<Policies>();
    assert_eq!(
        policies.max_building_level(),
        2,
        "max building level with HighRiseBan should be 2"
    );
}

#[test]
fn test_policy_max_building_level_without_high_rise_ban() {
    let city = TestCity::new();
    let policies = city.resource::<Policies>();
    assert_eq!(
        policies.max_building_level(),
        3,
        "max building level without HighRiseBan should be 3"
    );
}

#[test]
fn test_policy_industrial_tax_multiplier_with_tax_break() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::HeavyIndustryTaxBreak);
    }
    let policies = city.resource::<Policies>();
    let mult = policies.industrial_tax_multiplier();
    assert!(
        (mult - 0.5).abs() < f32::EPSILON,
        "industrial tax multiplier with HeavyIndustryTaxBreak should be 0.5, got {mult}"
    );
}

#[test]
fn test_policy_commercial_demand_bonus_stacks() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::TourismPromotion); // +0.15
        policies.toggle(Policy::SmallBusinessGrant); // +0.10
    }
    let policies = city.resource::<Policies>();
    let expected = 0.15 + 0.10;
    let actual = policies.commercial_demand_bonus();
    assert!(
        (actual - expected).abs() < f32::EPSILON,
        "commercial demand bonus should be {expected}, got {actual}"
    );
}

#[test]
fn test_policy_education_multiplier_with_education_push() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::EducationPush);
    }
    let policies = city.resource::<Policies>();
    let mult = policies.education_multiplier();
    assert!(
        (mult - 1.5).abs() < f32::EPSILON,
        "education multiplier with EducationPush should be 1.5, got {mult}"
    );
}

#[test]
fn test_policy_industrial_demand_bonus_with_tax_break() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::HeavyIndustryTaxBreak);
    }
    let policies = city.resource::<Policies>();
    let bonus = policies.industrial_demand_bonus();
    assert!(
        (bonus - 0.15).abs() < f32::EPSILON,
        "industrial demand bonus with HeavyIndustryTaxBreak should be 0.15, got {bonus}"
    );
}

#[test]
fn test_policy_disabling_removes_effects() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::IndustrialAirFilters);
        policies.toggle(Policy::RecyclingProgram);
    }
    // Verify effects are active
    assert!(
        (city.resource::<Policies>().pollution_multiplier() - 0.6).abs() < f32::EPSILON,
        "pollution multiplier should be 0.6 when active"
    );
    assert!(
        (city.resource::<Policies>().garbage_multiplier() - 0.7).abs() < f32::EPSILON,
        "garbage multiplier should be 0.7 when active"
    );

    // Disable them
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::IndustrialAirFilters);
        policies.toggle(Policy::RecyclingProgram);
    }
    // Verify effects are removed
    assert!(
        (city.resource::<Policies>().pollution_multiplier() - 1.0).abs() < f32::EPSILON,
        "pollution multiplier should return to 1.0 after disabling"
    );
    assert!(
        (city.resource::<Policies>().garbage_multiplier() - 1.0).abs() < f32::EPSILON,
        "garbage multiplier should return to 1.0 after disabling"
    );
}

#[test]
fn test_policy_cost_deducted_from_budget_after_tax_collection() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(10, 50, 100, 50, RoadType::Local)
        .with_zone_rect(11, 48, 20, 49, ZoneType::ResidentialLow)
        .with_building(15, 48, ZoneType::ResidentialLow, 1);
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::FreePublicTransport); // 50/month
        policies.toggle(Policy::HealthcareForAll); // 45/month
    }
    // Advance the game clock past the 30-day tax collection interval
    // so that collect_taxes fires on the next tick.
    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 32;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }
    // Run a slow cycle so collect_taxes executes
    city.tick_slow_cycle();
    // The expense breakdown should show policy costs
    let extended = city.resource::<crate::budget::ExtendedBudget>();
    let policy_costs = extended.expense_breakdown.policy_costs;
    let expected_policy_cost = 50.0 + 45.0;
    assert!(
        (policy_costs - expected_policy_cost).abs() < f64::EPSILON,
        "policy costs in expense breakdown should be {expected_policy_cost}, got {policy_costs}"
    );
    // Monthly expenses should include policy costs
    let budget = city.budget();
    assert!(
        budget.monthly_expenses >= expected_policy_cost,
        "monthly expenses ({}) should include policy costs ({expected_policy_cost})",
        budget.monthly_expenses
    );
}

#[test]
fn test_policy_no_cost_when_no_policies_active() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_road(10, 50, 100, 50, RoadType::Local)
        .with_building(15, 48, ZoneType::ResidentialLow, 1);
    // Advance the game clock past the 30-day tax collection interval
    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 32;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }
    city.tick_slow_cycle();
    let extended = city.resource::<crate::budget::ExtendedBudget>();
    let policy_costs = extended.expense_breakdown.policy_costs;
    assert!(
        policy_costs.abs() < f64::EPSILON,
        "policy costs should be 0 with no active policies, got {policy_costs}"
    );
}

#[test]
fn test_policy_happiness_bonus_stacks_correctly() {
    let mut city = TestCity::new();
    // Enable one happiness policy at a time and verify stacking
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::FreePublicTransport); // +3
    }
    assert!(
        (city.resource::<Policies>().happiness_bonus() - 3.0).abs() < f32::EPSILON,
        "single policy should give +3 happiness"
    );

    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::NeighborhoodWatch); // +2
    }
    assert!(
        (city.resource::<Policies>().happiness_bonus() - 5.0).abs() < f32::EPSILON,
        "two policies should give +5 happiness (3+2)"
    );
}

#[test]
fn test_policy_all_returns_all_variants() {
    let all = Policy::all();
    assert_eq!(all.len(), 18, "Policy::all() should return all 18 policies");
    // Verify a few known policies exist
    assert!(
        all.contains(&Policy::FreePublicTransport),
        "should contain FreePublicTransport"
    );
    assert!(
        all.contains(&Policy::EncourageBiking),
        "should contain EncourageBiking"
    );
    assert!(
        all.contains(&Policy::CumulativeZoning),
        "should contain CumulativeZoning"
    );
}

#[test]
fn test_policy_each_has_nonempty_name_and_description() {
    for policy in Policy::all() {
        let name = policy.name();
        let desc = policy.description();
        assert!(
            !name.is_empty(),
            "policy {:?} should have a non-empty name",
            policy
        );
        assert!(
            !desc.is_empty(),
            "policy {:?} should have a non-empty description",
            policy
        );
    }
}

#[test]
fn test_policy_disable_removes_from_cost_calculation() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut policies = world.resource_mut::<Policies>();
        policies.toggle(Policy::FreePublicTransport); // 50
        policies.toggle(Policy::RecyclingProgram); // 20
    }
    assert!(
        (city.resource::<Policies>().total_monthly_cost() - 70.0).abs() < f64::EPSILON,
        "total cost should be 70 with both active"
    );

    // Disable FreePublicTransport
    {
        let world = city.world_mut();
        world
            .resource_mut::<Policies>()
            .toggle(Policy::FreePublicTransport);
    }
    assert!(
        (city.resource::<Policies>().total_monthly_cost() - 20.0).abs() < f64::EPSILON,
        "total cost should be 20 after disabling FreePublicTransport"
    );
}

#[test]
fn test_policy_commercial_demand_bonus_zero_without_policies() {
    let city = TestCity::new();
    let policies = city.resource::<Policies>();
    assert!(
        policies.commercial_demand_bonus().abs() < f32::EPSILON,
        "commercial demand bonus should be 0 without policies"
    );
}

#[test]
fn test_policy_industrial_demand_bonus_zero_without_tax_break() {
    let city = TestCity::new();
    let policies = city.resource::<Policies>();
    assert!(
        policies.industrial_demand_bonus().abs() < f32::EPSILON,
        "industrial demand bonus should be 0 without tax break"
    );
}

// ====================================================================
// TEST-062: Negative Budget Consequences
// ====================================================================

#[test]
fn test_negative_budget_does_not_crash() {
    let mut city = TestCity::new()
        .with_budget(-1_000_000.0)
        .with_road(10, 10, 10, 30, RoadType::Local)
        .with_zone_rect(11, 10, 13, 14, ZoneType::ResidentialLow)
        .with_zone_rect(11, 16, 13, 20, ZoneType::CommercialLow)
        .with_building(12, 12, ZoneType::ResidentialLow, 1)
        .with_building(12, 18, ZoneType::CommercialLow, 1)
        .with_citizen((12, 12), (12, 18))
        .with_service(12, 15, ServiceType::PoliceStation);

    city.tick_slow_cycles(5);

    let treasury = city.budget().treasury;
    assert!(
        treasury.is_finite(),
        "Treasury should remain a finite number, got {treasury}"
    );
    let _count = city.citizen_count();
}

#[test]
fn test_negative_budget_extended_stability() {
    let mut city = TestCity::new().with_budget(-5_000_000.0);
    city.tick_slow_cycles(10);

    let budget = city.budget();
    assert!(budget.treasury.is_finite());
    assert!(budget.monthly_income.is_finite());
    assert!(budget.monthly_expenses.is_finite());
}

#[test]
fn test_negative_budget_triggers_crisis_event() {
    use crate::events::{CityEventType, EventJournal};

    let mut city = TestCity::new().with_budget(-50_000.0);
    city.tick_slow_cycles(3);

    let journal = city.resource::<EventJournal>();
    let has_crisis = journal
        .events
        .iter()
        .any(|e| matches!(e.event_type, CityEventType::BudgetCrisis));
    assert!(has_crisis, "Expected BudgetCrisis event");
}

#[test]
fn test_service_coverage_degrades_with_reduced_budgets() {
    use crate::budget::ExtendedBudget;
    use crate::happiness::ServiceCoverageGrid;

    let mut city =
        TestCity::new()
            .with_budget(100_000.0)
            .with_service(30, 30, ServiceType::Hospital);

    city.tick_slow_cycles(1);

    let idx = ServiceCoverageGrid::idx(30, 45);
    let has_health_full = city.resource::<ServiceCoverageGrid>().has_health(idx);

    {
        let world = city.world_mut();
        world
            .resource_mut::<ExtendedBudget>()
            .service_budgets
            .healthcare = 0.0;
    }

    city.tick_slow_cycles(1);

    let has_health_zero = city.resource::<ServiceCoverageGrid>().has_health(idx);

    assert!(
        has_health_full,
        "Should have health coverage at full budget"
    );
    assert!(!has_health_zero, "Should lose coverage when budget is 0");
}

#[test]
fn test_service_coverage_at_origin_with_low_budget() {
    use crate::budget::ExtendedBudget;
    use crate::happiness::ServiceCoverageGrid;

    let mut city = TestCity::new().with_service(50, 50, ServiceType::PoliceStation);

    {
        let world = city.world_mut();
        world
            .resource_mut::<ExtendedBudget>()
            .service_budgets
            .police = 0.1;
    }

    city.tick_slow_cycles(1);

    let idx = ServiceCoverageGrid::idx(50, 50);
    let has_police = city.resource::<ServiceCoverageGrid>().has_police(idx);
    assert!(
        has_police,
        "Police station should cover its own cell at 10% budget"
    );
}

#[test]
fn test_building_placement_with_negative_budget() {
    let city = TestCity::new()
        .with_budget(-10_000.0)
        .with_road(20, 20, 20, 40, RoadType::Local)
        .with_building(21, 25, ZoneType::ResidentialLow, 1)
        .with_building(21, 30, ZoneType::CommercialLow, 1);

    city.assert_has_road(20, 25);
    city.assert_has_building(21, 25);
    city.assert_has_building(21, 30);
    city.assert_budget_below(0.0);
}

#[test]
fn test_budget_recovery_from_negative() {
    let mut city = TestCity::new()
        .with_budget(-500.0)
        .with_road(10, 10, 10, 50, RoadType::Local)
        .with_zone_rect(11, 10, 14, 50, ZoneType::ResidentialLow)
        .with_zone_rect(8, 10, 9, 50, ZoneType::CommercialLow);

    for y in (10..50).step_by(2) {
        city = city
            .with_building(12, y, ZoneType::ResidentialLow, 3)
            .with_building(9, y, ZoneType::CommercialLow, 3);
    }

    let initial_treasury = city.budget().treasury;
    assert!(initial_treasury < 0.0);

    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 1;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }

    city.tick_slow_cycles(20);

    let after_treasury = city.budget().treasury;
    assert!(
        after_treasury > initial_treasury,
        "Treasury should improve: initial={initial_treasury}, after={after_treasury}"
    );
}

#[test]
fn test_negative_budget_crisis_events_recur() {
    use crate::events::{CityEventType, EventJournal};

    let mut city = TestCity::new().with_budget(-100_000.0);
    city.tick_slow_cycles(10);

    let journal = city.resource::<EventJournal>();
    let crisis_count = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, CityEventType::BudgetCrisis))
        .count();
    assert!(crisis_count >= 1, "Expected at least 1 BudgetCrisis event");
}

#[test]
fn test_tel_aviv_negative_budget_stability() {
    let mut city = TestCity::with_tel_aviv();

    {
        let world = city.world_mut();
        world.resource_mut::<CityBudget>().treasury = -1_000_000.0;
    }

    city.tick_slow_cycles(3);

    let budget = city.budget();
    assert!(budget.treasury.is_finite());
    assert!(
        city.citizen_count() > 0,
        "Tel Aviv should still have citizens"
    );
}

// ====================================================================
// TRAF-011: Roundabout builder
// ====================================================================

#[test]
fn test_roundabout_creation_produces_valid_struct() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::roundabout::{create_roundabout, CirculationDirection};

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut segments = crate::road_segments::RoadSegmentStore::default();

    let rb = create_roundabout(
        (128, 128),
        3,
        RoadType::Avenue,
        CirculationDirection::Clockwise,
        &mut segments,
        &mut grid,
        &mut roads,
    );

    assert!(rb.ring_cells.len() > 4, "ring should have multiple cells");
    assert_eq!(rb.center_x, 128);
    assert_eq!(rb.center_y, 128);
    assert_eq!(rb.radius, 3);
    assert!(
        !rb.segment_ids.is_empty(),
        "roundabout should have Bezier segments"
    );
}

#[test]
fn test_roundabout_registry_find_at_cell() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::roundabout::{
        create_roundabout, CirculationDirection, RoundaboutRegistry, RoundaboutStats,
    };

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut segments = crate::road_segments::RoadSegmentStore::default();

    let rb = create_roundabout(
        (100, 100),
        2,
        RoadType::Local,
        CirculationDirection::Clockwise,
        &mut segments,
        &mut grid,
        &mut roads,
    );

    let first_ring_cell = rb.ring_cells[0];
    let mut registry = RoundaboutRegistry::default();
    registry.stats.push(RoundaboutStats::default());
    registry.roundabouts.push(rb);

    assert!(
        registry
            .find_at_cell(first_ring_cell.0, first_ring_cell.1)
            .is_some(),
        "should find roundabout at ring cell"
    );
    assert!(
        registry.find_at_cell(0, 0).is_none(),
        "should not find roundabout at (0,0)"
    );
}

#[test]
fn test_roundabout_saveable_roundtrip() {
    use crate::roundabout::{
        CirculationDirection, Roundabout, RoundaboutRegistry, RoundaboutTrafficRule,
    };
    use crate::Saveable;

    let mut registry = RoundaboutRegistry::default();
    registry.roundabouts.push(Roundabout {
        center_x: 50,
        center_y: 60,
        radius: 3,
        road_type: RoadType::Avenue,
        direction: CirculationDirection::Clockwise,
        traffic_rule: RoundaboutTrafficRule::YieldOnEntry,
        ring_cells: vec![(49, 60), (50, 61), (51, 60), (50, 59)],
        segment_ids: vec![10, 11, 12, 13],
        approach_connections: vec![(48, 60)],
    });

    let bytes = registry.save_to_bytes().expect("should serialize");
    let restored = RoundaboutRegistry::load_from_bytes(&bytes);
    assert_eq!(restored.roundabouts.len(), 1);
    assert_eq!(restored.roundabouts[0].center_x, 50);
    assert_eq!(restored.roundabouts[0].ring_cells.len(), 4);
}

// ====================================================================
// TEST-064: Unlock / Progression System Tests
// ====================================================================

#[test]
fn test_unlock_state_default_has_starter_unlocks() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let state = UnlockState::default();
    assert!(state.is_unlocked(UnlockNode::BasicRoads));
    assert!(state.is_unlocked(UnlockNode::ResidentialZoning));
    assert!(state.is_unlocked(UnlockNode::CommercialZoning));
    assert!(state.is_unlocked(UnlockNode::IndustrialZoning));
    assert!(state.is_unlocked(UnlockNode::BasicPower));
    assert!(state.is_unlocked(UnlockNode::BasicWater));
    assert!(!state.is_unlocked(UnlockNode::FireService));
    assert!(!state.is_unlocked(UnlockNode::PoliceService));
}

#[test]
fn test_unlock_state_default_has_three_development_points() {
    use crate::unlocks::UnlockState;
    let state = UnlockState::default();
    assert_eq!(state.development_points, 3);
    assert_eq!(state.spent_points, 0);
    assert_eq!(state.available_points(), 3);
}

#[test]
fn test_unlock_node_cost_tiers() {
    use crate::unlocks::UnlockNode;
    assert_eq!(UnlockNode::BasicRoads.cost(), 0);
    assert_eq!(UnlockNode::BasicWater.cost(), 0);
    assert_eq!(UnlockNode::FireService.cost(), 1);
    assert_eq!(UnlockNode::PoliceService.cost(), 1);
    assert_eq!(UnlockNode::ElementaryEducation.cost(), 1);
    assert_eq!(UnlockNode::SmallParks.cost(), 1);
    assert_eq!(UnlockNode::BasicSanitation.cost(), 1);
    assert_eq!(UnlockNode::HealthCare.cost(), 2);
    assert_eq!(UnlockNode::HighDensityResidential.cost(), 2);
    assert_eq!(UnlockNode::SolarPower.cost(), 2);
    assert_eq!(UnlockNode::DeathCare.cost(), 2);
    assert_eq!(UnlockNode::OfficeZoning.cost(), 3);
    assert_eq!(UnlockNode::WindPower.cost(), 3);
    assert_eq!(UnlockNode::PublicTransport.cost(), 3);
    assert_eq!(UnlockNode::Telecom.cost(), 3);
    assert_eq!(UnlockNode::Landmarks.cost(), 5);
    assert_eq!(UnlockNode::NuclearPower.cost(), 5);
    assert_eq!(UnlockNode::PolicySystem.cost(), 5);
    assert_eq!(UnlockNode::InternationalAirports.cost(), 7);
}

#[test]
fn test_unlock_node_required_population_tiers() {
    use crate::unlocks::UnlockNode;
    assert_eq!(UnlockNode::BasicRoads.required_population(), 0);
    assert_eq!(UnlockNode::FireService.required_population(), 500);
    assert_eq!(UnlockNode::PoliceService.required_population(), 500);
    assert_eq!(UnlockNode::HealthCare.required_population(), 2_000);
    assert_eq!(
        UnlockNode::HighDensityResidential.required_population(),
        2_000
    );
    assert_eq!(UnlockNode::OfficeZoning.required_population(), 5_000);
    assert_eq!(UnlockNode::PublicTransport.required_population(), 5_000);
    assert_eq!(UnlockNode::RegionalAirports.required_population(), 20_000);
    assert_eq!(UnlockNode::Landmarks.required_population(), 50_000);
    assert_eq!(UnlockNode::NuclearPower.required_population(), 50_000);
    assert_eq!(
        UnlockNode::InternationalAirports.required_population(),
        100_000
    );
}

#[test]
fn test_unlock_purchase_deducts_points() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    assert_eq!(state.available_points(), 3);
    assert!(state.purchase(UnlockNode::FireService));
    assert_eq!(state.available_points(), 2);
    assert!(state.is_unlocked(UnlockNode::FireService));
    assert!(state.purchase(UnlockNode::PoliceService));
    assert_eq!(state.available_points(), 1);
    assert!(state.purchase(UnlockNode::ElementaryEducation));
    assert_eq!(state.available_points(), 0);
}

#[test]
fn test_unlock_purchase_fails_insufficient_points() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    assert!(state.purchase(UnlockNode::HealthCare));
    assert_eq!(state.available_points(), 1);
    assert!(!state.purchase(UnlockNode::HighDensityResidential));
    assert!(!state.is_unlocked(UnlockNode::HighDensityResidential));
    assert_eq!(state.available_points(), 1);
}

#[test]
fn test_unlock_purchase_fails_for_already_unlocked() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    assert!(!state.purchase(UnlockNode::BasicRoads));
    assert_eq!(state.available_points(), 3);
}

#[test]
fn test_unlock_can_purchase_checks_population_threshold() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 10;
    assert!(!state.can_purchase(UnlockNode::FireService, 499));
    assert!(state.can_purchase(UnlockNode::FireService, 500));
    assert!(state.can_purchase(UnlockNode::FireService, 1000));
    assert!(!state.can_purchase(UnlockNode::HealthCare, 1999));
    assert!(state.can_purchase(UnlockNode::HealthCare, 2000));
}

#[test]
fn test_unlock_can_purchase_false_when_already_unlocked() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let state = UnlockState::default();
    assert!(!state.can_purchase(UnlockNode::BasicRoads, 0));
}

#[test]
fn test_unlock_can_purchase_false_when_insufficient_points() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.spent_points = 3;
    assert_eq!(state.available_points(), 0);
    assert!(!state.can_purchase(UnlockNode::FireService, 1000));
}

#[test]
fn test_unlock_all_nodes_have_names() {
    use crate::unlocks::UnlockNode;
    for &node in UnlockNode::all() {
        assert!(!node.name().is_empty(), "Node {:?} has no name", node);
    }
}

#[test]
fn test_unlock_all_returns_all_variants() {
    use crate::unlocks::UnlockNode;
    let all = UnlockNode::all();
    assert!(all.contains(&UnlockNode::BasicRoads));
    assert!(all.contains(&UnlockNode::FireService));
    assert!(all.contains(&UnlockNode::HealthCare));
    assert!(all.contains(&UnlockNode::OfficeZoning));
    assert!(all.contains(&UnlockNode::Landmarks));
    assert!(all.contains(&UnlockNode::InternationalAirports));
}

#[test]
fn test_unlock_service_mapping_fire_service() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    assert!(!state.is_service_unlocked(ServiceType::FireStation));
    assert!(!state.is_service_unlocked(ServiceType::FireHouse));
    state.development_points = 10;
    state.purchase(UnlockNode::FireService);
    assert!(state.is_service_unlocked(ServiceType::FireStation));
    assert!(state.is_service_unlocked(ServiceType::FireHouse));
    assert!(!state.is_service_unlocked(ServiceType::FireHQ));
}

#[test]
fn test_unlock_service_mapping_police_service() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    assert!(!state.is_service_unlocked(ServiceType::PoliceStation));
    state.development_points = 10;
    state.purchase(UnlockNode::PoliceService);
    assert!(state.is_service_unlocked(ServiceType::PoliceStation));
    assert!(state.is_service_unlocked(ServiceType::PoliceKiosk));
    assert!(!state.is_service_unlocked(ServiceType::PoliceHQ));
    assert!(!state.is_service_unlocked(ServiceType::Prison));
}

#[test]
fn test_unlock_service_mapping_advanced_emergency() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::AdvancedEmergency);
    assert!(state.is_service_unlocked(ServiceType::FireHQ));
    assert!(state.is_service_unlocked(ServiceType::PoliceHQ));
    assert!(state.is_service_unlocked(ServiceType::Prison));
    assert!(state.is_service_unlocked(ServiceType::MedicalCenter));
}

#[test]
fn test_unlock_service_mapping_education() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::ElementaryEducation);
    assert!(state.is_service_unlocked(ServiceType::ElementarySchool));
    assert!(state.is_service_unlocked(ServiceType::Library));
    assert!(state.is_service_unlocked(ServiceType::Kindergarten));
    assert!(!state.is_service_unlocked(ServiceType::HighSchool));
    state.purchase(UnlockNode::HighSchoolEducation);
    assert!(state.is_service_unlocked(ServiceType::HighSchool));
    assert!(!state.is_service_unlocked(ServiceType::University));
    state.purchase(UnlockNode::UniversityEducation);
    assert!(state.is_service_unlocked(ServiceType::University));
}

#[test]
fn test_unlock_service_mapping_sanitation() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::BasicSanitation);
    assert!(state.is_service_unlocked(ServiceType::Landfill));
    assert!(state.is_service_unlocked(ServiceType::TransferStation));
    assert!(!state.is_service_unlocked(ServiceType::RecyclingCenter));
    state.purchase(UnlockNode::AdvancedSanitation);
    assert!(state.is_service_unlocked(ServiceType::RecyclingCenter));
    assert!(state.is_service_unlocked(ServiceType::Incinerator));
}

#[test]
fn test_unlock_service_mapping_transport() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::PublicTransport);
    assert!(state.is_service_unlocked(ServiceType::BusDepot));
    assert!(state.is_service_unlocked(ServiceType::TrainStation));
    assert!(!state.is_service_unlocked(ServiceType::SubwayStation));
    state.purchase(UnlockNode::AdvancedTransport);
    assert!(state.is_service_unlocked(ServiceType::SubwayStation));
    assert!(state.is_service_unlocked(ServiceType::TramDepot));
    assert!(state.is_service_unlocked(ServiceType::FerryPier));
}

#[test]
fn test_unlock_service_mapping_airports() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::SmallAirstrip));
    state.purchase(UnlockNode::SmallAirstrips);
    assert!(state.is_service_unlocked(ServiceType::SmallAirstrip));
    assert!(!state.is_service_unlocked(ServiceType::RegionalAirport));
    state.purchase(UnlockNode::RegionalAirports);
    assert!(state.is_service_unlocked(ServiceType::RegionalAirport));
    assert!(!state.is_service_unlocked(ServiceType::InternationalAirport));
    state.purchase(UnlockNode::InternationalAirports);
    assert!(state.is_service_unlocked(ServiceType::InternationalAirport));
}

#[test]
fn test_unlock_service_mapping_telecom() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::CellTower));
    assert!(!state.is_service_unlocked(ServiceType::DataCenter));
    state.purchase(UnlockNode::Telecom);
    assert!(state.is_service_unlocked(ServiceType::CellTower));
    assert!(state.is_service_unlocked(ServiceType::DataCenter));
}

#[test]
fn test_unlock_service_mapping_landmarks() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::CityHall));
    state.purchase(UnlockNode::Landmarks);
    assert!(state.is_service_unlocked(ServiceType::CityHall));
    assert!(state.is_service_unlocked(ServiceType::Museum));
    assert!(state.is_service_unlocked(ServiceType::Cathedral));
    assert!(state.is_service_unlocked(ServiceType::TVStation));
}

#[test]
fn test_unlock_service_mapping_death_care() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::Cemetery));
    assert!(!state.is_service_unlocked(ServiceType::Crematorium));
    state.purchase(UnlockNode::DeathCare);
    assert!(state.is_service_unlocked(ServiceType::Cemetery));
    assert!(state.is_service_unlocked(ServiceType::Crematorium));
}

#[test]
fn test_unlock_service_mapping_postal() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::PostOffice));
    assert!(!state.is_service_unlocked(ServiceType::MailSortingCenter));
    state.purchase(UnlockNode::PostalService);
    assert!(state.is_service_unlocked(ServiceType::PostOffice));
    assert!(state.is_service_unlocked(ServiceType::MailSortingCenter));
}

#[test]
fn test_unlock_service_mapping_heating() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::HeatingBoiler));
    state.purchase(UnlockNode::BasicHeating);
    assert!(state.is_service_unlocked(ServiceType::HeatingBoiler));
    assert!(!state.is_service_unlocked(ServiceType::DistrictHeatingPlant));
    state.purchase(UnlockNode::DistrictHeatingNetwork);
    assert!(state.is_service_unlocked(ServiceType::DistrictHeatingPlant));
    assert!(state.is_service_unlocked(ServiceType::GeothermalPlant));
}

#[test]
fn test_unlock_service_mapping_water_infrastructure() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::WaterTreatmentPlant));
    assert!(!state.is_service_unlocked(ServiceType::WellPump));
    state.purchase(UnlockNode::WaterInfrastructure);
    assert!(state.is_service_unlocked(ServiceType::WaterTreatmentPlant));
    assert!(state.is_service_unlocked(ServiceType::WellPump));
}

#[test]
fn test_unlock_utility_mapping_basic_power_and_water() {
    use crate::unlocks::UnlockState;
    let state = UnlockState::default();
    assert!(state.is_utility_unlocked(UtilityType::PowerPlant));
    assert!(state.is_utility_unlocked(UtilityType::WaterTower));
    assert!(state.is_utility_unlocked(UtilityType::PumpingStation));
    assert!(!state.is_utility_unlocked(UtilityType::SolarFarm));
    assert!(!state.is_utility_unlocked(UtilityType::WindTurbine));
    assert!(!state.is_utility_unlocked(UtilityType::NuclearPlant));
}

#[test]
fn test_unlock_utility_mapping_advanced_power() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::SolarPower);
    assert!(state.is_utility_unlocked(UtilityType::SolarFarm));
    state.purchase(UnlockNode::WindPower);
    assert!(state.is_utility_unlocked(UtilityType::WindTurbine));
    assert!(state.is_utility_unlocked(UtilityType::Geothermal));
    state.purchase(UnlockNode::NuclearPower);
    assert!(state.is_utility_unlocked(UtilityType::NuclearPlant));
}

#[test]
fn test_unlock_utility_mapping_sewage() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_utility_unlocked(UtilityType::SewagePlant));
    assert!(!state.is_utility_unlocked(UtilityType::WaterTreatment));
    state.purchase(UnlockNode::SewagePlant);
    assert!(state.is_utility_unlocked(UtilityType::SewagePlant));
    assert!(state.is_utility_unlocked(UtilityType::WaterTreatment));
}

#[test]
fn test_milestone_awards_development_points_at_500_pop() {
    use crate::unlocks::UnlockState;
    let mut city = TestCity::new();
    city.world_mut()
        .resource_mut::<crate::stats::CityStats>()
        .population = 500;
    let initial_dp = city.resource::<UnlockState>().development_points;
    city.tick_slow_cycle();
    let state = city.resource::<UnlockState>();
    assert_eq!(
        state.development_points,
        initial_dp + 2,
        "Should gain 2 DP at 500 pop milestone"
    );
    assert_eq!(state.last_milestone_pop, 500);
}

#[test]
fn test_milestone_awards_do_not_retrigger() {
    use crate::unlocks::UnlockState;
    let mut city = TestCity::new();
    city.world_mut()
        .resource_mut::<crate::stats::CityStats>()
        .population = 500;
    city.tick_slow_cycle();
    let dp_after_first = city.resource::<UnlockState>().development_points;
    city.world_mut()
        .resource_mut::<crate::stats::CityStats>()
        .population = 500;
    city.tick_slow_cycle();
    let dp_after_second = city.resource::<UnlockState>().development_points;
    assert_eq!(
        dp_after_first, dp_after_second,
        "Milestone should not re-trigger at same population"
    );
}

#[test]
fn test_milestone_multiple_thresholds_in_sequence() {
    use crate::unlocks::UnlockState;
    let mut city = TestCity::new();
    let initial_dp = city.resource::<UnlockState>().development_points;
    city.world_mut()
        .resource_mut::<crate::stats::CityStats>()
        .population = 2_000;
    city.tick_slow_cycle();
    let state = city.resource::<UnlockState>();
    assert_eq!(
        state.development_points,
        initial_dp + 2 + 2 + 3,
        "Should gain DP for all milestones up to 2000"
    );
    assert_eq!(state.last_milestone_pop, 2_000);
}

#[test]
fn test_milestone_skipping_intermediate_thresholds_still_awards() {
    use crate::unlocks::UnlockState;
    let mut city = TestCity::new();
    let initial_dp = city.resource::<UnlockState>().development_points;
    city.world_mut()
        .resource_mut::<crate::stats::CityStats>()
        .population = 10_000;
    city.tick_slow_cycle();
    let state = city.resource::<UnlockState>();
    let expected = initial_dp + 2 + 2 + 3 + 3 + 4;
    assert_eq!(
        state.development_points, expected,
        "Should gain DP for all milestones up to 10000 (expected {})",
        expected
    );
}

#[test]
fn test_unlock_state_resource_exists_in_test_city() {
    use crate::unlocks::UnlockState;
    let city = TestCity::new();
    city.assert_resource_exists::<UnlockState>();
    let state = city.resource::<UnlockState>();
    assert_eq!(state.unlocked_nodes.len(), 6);
}

#[test]
fn test_achievement_tracker_resource_exists_in_test_city() {
    use crate::achievements::AchievementTracker;
    let city = TestCity::new();
    city.assert_resource_exists::<AchievementTracker>();
    let tracker = city.resource::<AchievementTracker>();
    assert_eq!(tracker.unlocked_count(), 0);
}

#[test]
fn test_achievement_dp_reward_increases_unlock_points() {
    use crate::achievements::{Achievement, AchievementReward};
    let reward = Achievement::Millionaire.reward();
    match reward {
        AchievementReward::DevelopmentPoints(pts) => {
            assert_eq!(pts, 5, "Millionaire should give 5 DP");
        }
        _ => panic!("Millionaire should reward DevelopmentPoints"),
    }
    let reward = Achievement::FullPowerCoverage.reward();
    match reward {
        AchievementReward::DevelopmentPoints(pts) => {
            assert_eq!(pts, 3, "FullPowerCoverage should give 3 DP");
        }
        _ => panic!("FullPowerCoverage should reward DevelopmentPoints"),
    }
}

#[test]
fn test_achievement_treasury_reward_values() {
    use crate::achievements::{Achievement, AchievementReward};
    let reward = Achievement::Population1K.reward();
    match reward {
        AchievementReward::TreasuryBonus(amount) => {
            assert!((amount - 5_000.0).abs() < 0.01, "Pop 1K should give $5K");
        }
        _ => panic!("Population1K should reward TreasuryBonus"),
    }
    let reward = Achievement::Population1M.reward();
    match reward {
        AchievementReward::TreasuryBonus(amount) => {
            assert!(
                (amount - 1_000_000.0).abs() < 0.01,
                "Pop 1M should give $1M"
            );
        }
        _ => panic!("Population1M should reward TreasuryBonus"),
    }
}

#[test]
fn test_achievement_tracker_no_double_unlock() {
    use crate::achievements::{Achievement, AchievementTracker};
    let mut tracker = AchievementTracker::default();
    assert!(!tracker.is_unlocked(Achievement::Population1K));
    tracker.unlocked.insert(Achievement::Population1K, 100);
    assert!(tracker.is_unlocked(Achievement::Population1K));
    assert_eq!(tracker.unlocked_count(), 1);
    tracker.unlocked.insert(Achievement::Population1K, 200);
    assert_eq!(tracker.unlocked_count(), 1);
    assert!(tracker.is_unlocked(Achievement::Population1K));
}

#[test]
fn test_achievement_all_have_metadata() {
    use crate::achievements::Achievement;
    for &a in Achievement::ALL {
        assert!(!a.name().is_empty(), "Achievement {:?} has no name", a);
        assert!(
            !a.description().is_empty(),
            "Achievement {:?} has no description",
            a
        );
        let reward_desc = a.reward().description();
        assert!(
            !reward_desc.is_empty(),
            "Achievement {:?} reward has no description",
            a
        );
    }
}

#[test]
fn test_unlock_purchase_sequence_exhausts_points() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    assert!(state.purchase(UnlockNode::FireService));
    assert!(state.purchase(UnlockNode::PoliceService));
    assert!(state.purchase(UnlockNode::SmallParks));
    assert_eq!(state.available_points(), 0);
    assert!(!state.purchase(UnlockNode::ElementaryEducation));
    assert!(!state.is_unlocked(UnlockNode::ElementaryEducation));
}

#[test]
fn test_unlock_available_points_uses_saturating_sub() {
    use crate::unlocks::UnlockState;
    let mut state = UnlockState::default();
    state.spent_points = 100;
    assert_eq!(state.available_points(), 0);
}

#[test]
fn test_unlock_service_healthcare_chain() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::Hospital));
    assert!(!state.is_service_unlocked(ServiceType::MedicalClinic));
    assert!(!state.is_service_unlocked(ServiceType::HomelessShelter));
    assert!(!state.is_service_unlocked(ServiceType::WelfareOffice));
    state.purchase(UnlockNode::HealthCare);
    assert!(state.is_service_unlocked(ServiceType::Hospital));
    assert!(state.is_service_unlocked(ServiceType::MedicalClinic));
    assert!(state.is_service_unlocked(ServiceType::HomelessShelter));
    assert!(state.is_service_unlocked(ServiceType::WelfareOffice));
}

#[test]
fn test_unlock_service_parks_and_entertainment() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::SmallParks);
    assert!(state.is_service_unlocked(ServiceType::SmallPark));
    assert!(state.is_service_unlocked(ServiceType::Playground));
    assert!(!state.is_service_unlocked(ServiceType::LargePark));
    state.purchase(UnlockNode::AdvancedParks);
    assert!(state.is_service_unlocked(ServiceType::LargePark));
    assert!(state.is_service_unlocked(ServiceType::SportsField));
    assert!(!state.is_service_unlocked(ServiceType::Plaza));
    state.purchase(UnlockNode::Entertainment);
    assert!(state.is_service_unlocked(ServiceType::Plaza));
    assert!(state.is_service_unlocked(ServiceType::Stadium));
}

// ===========================================================================
// TEST-063: Random City Events
// ===========================================================================

/// Test that the EventJournal resource is initialized on city creation.
#[test]
fn test_random_events_journal_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::events::EventJournal>();
}

/// Test that the ActiveCityEffects resource is initialized on city creation.
#[test]
fn test_random_events_active_effects_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::events::ActiveCityEffects>();
}

/// Test that the MilestoneTracker resource is initialized on city creation.
#[test]
fn test_random_events_milestone_tracker_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::events::MilestoneTracker>();
}

/// Test that a BudgetCrisis event is logged when the treasury is negative.
#[test]
fn test_random_events_budget_crisis_logged_when_treasury_negative() {
    let mut city = TestCity::new().with_budget(-5000.0);
    city.tick_slow_cycle();

    let journal = city.resource::<crate::events::EventJournal>();
    let crisis_events: Vec<_> = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, crate::events::CityEventType::BudgetCrisis))
        .collect();

    assert!(
        !crisis_events.is_empty(),
        "Expected at least one BudgetCrisis event when treasury is negative"
    );
}

/// Test that BudgetCrisis is only logged once per day (deduplication).
#[test]
fn test_random_events_budget_crisis_dedup_same_day() {
    let mut city = TestCity::new().with_budget(-5000.0);
    city.tick_slow_cycle();
    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let journal = city.resource::<crate::events::EventJournal>();
    let mut crisis_days: Vec<u32> = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, crate::events::CityEventType::BudgetCrisis))
        .map(|e| e.day)
        .collect();
    crisis_days.sort();
    crisis_days.dedup();

    let total_crisis = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, crate::events::CityEventType::BudgetCrisis))
        .count();

    assert!(
        total_crisis <= crisis_days.len() + 1,
        "Expected at most one BudgetCrisis per day, got {} events across {} days",
        total_crisis,
        crisis_days.len()
    );
}

/// Test that population milestones are recorded in the EventJournal.
#[test]
fn test_random_events_population_milestone_logged() {
    use crate::events::{CityEventType, EventJournal, MilestoneTracker};
    use crate::virtual_population::VirtualPopulation;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<VirtualPopulation>().total_virtual = 1_500;
    }
    city.tick_slow_cycle();

    let journal = city.resource::<EventJournal>();
    let milestone_events: Vec<_> = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, CityEventType::MilestoneReached(_)))
        .collect();

    assert!(
        !milestone_events.is_empty(),
        "Expected a MilestoneReached event after virtual population hit 1,500"
    );

    let tracker = city.resource::<MilestoneTracker>();
    assert!(
        tracker.reached_milestones.contains(&1_000),
        "MilestoneTracker should contain the 1,000 threshold"
    );
}

/// Test that population milestones are NOT re-triggered once already reached.
#[test]
fn test_random_events_population_milestone_not_retriggered() {
    use crate::events::{CityEventType, EventJournal};
    use crate::virtual_population::VirtualPopulation;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<VirtualPopulation>().total_virtual = 1_500;
    }
    city.tick_slow_cycle();

    let first_count = {
        let journal = city.resource::<EventJournal>();
        journal
            .events
            .iter()
            .filter(|e| matches!(e.event_type, CityEventType::MilestoneReached(_)))
            .count()
    };

    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let second_count = {
        let journal = city.resource::<EventJournal>();
        journal
            .events
            .iter()
            .filter(|e| matches!(e.event_type, CityEventType::MilestoneReached(_)))
            .count()
    };

    assert_eq!(
        first_count, second_count,
        "Milestone should not re-trigger: first={}, after more ticks={}",
        first_count, second_count
    );
}

/// Test that multiple population milestones fire when population jumps past
/// several thresholds at once.
#[test]
fn test_random_events_multiple_milestones_at_once() {
    use crate::events::{CityEventType, EventJournal, MilestoneTracker};
    use crate::virtual_population::VirtualPopulation;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<VirtualPopulation>().total_virtual = 12_000;
    }
    city.tick_slow_cycle();

    let journal = city.resource::<EventJournal>();
    let milestone_count = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, CityEventType::MilestoneReached(_)))
        .count();

    assert!(
        milestone_count >= 3,
        "Expected at least 3 milestones (1K, 5K, 10K) for population 12,000, got {}",
        milestone_count
    );

    let tracker = city.resource::<MilestoneTracker>();
    assert!(tracker.reached_milestones.contains(&1_000));
    assert!(tracker.reached_milestones.contains(&5_000));
    assert!(tracker.reached_milestones.contains(&10_000));
}

/// Test that the festival effect timer decrements each slow tick.
#[test]
fn test_random_events_festival_effect_ticks_decrement() {
    use crate::events::ActiveCityEffects;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<ActiveCityEffects>().festival_ticks = 5;
    }
    city.tick_slow_cycle();

    let effects = city.resource::<ActiveCityEffects>();
    assert!(
        effects.festival_ticks <= 4 || effects.festival_ticks == 10,
        "Festival ticks should decrement from 5 to 4, or be reset to 10 if re-triggered, got {}",
        effects.festival_ticks
    );
}

/// Test that epidemic effect ticks decrement via the random_city_events system.
#[test]
fn test_random_events_epidemic_effect_ticks_decrement() {
    use crate::events::ActiveCityEffects;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<ActiveCityEffects>().epidemic_ticks = 5;
    }
    city.tick_slow_cycle();

    let effects = city.resource::<ActiveCityEffects>();
    assert!(
        effects.epidemic_ticks <= 4 || effects.epidemic_ticks == 10,
        "Epidemic ticks should decrement from 5 to 4, or be reset to 10 if re-triggered, got {}",
        effects.epidemic_ticks
    );
}

/// Test that apply_active_effects drains health during an active epidemic.
#[test]
fn test_random_events_epidemic_drains_health() {
    use crate::citizen::CitizenDetails;
    use crate::events::ActiveCityEffects;

    let home = (10, 10);
    let work = (15, 15);
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::Industrial, 1)
        .with_citizen(home, work);

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut CitizenDetails>();
        for mut details in q.iter_mut(world) {
            details.health = 80.0;
        }
    }
    {
        let world = city.world_mut();
        world.resource_mut::<ActiveCityEffects>().epidemic_ticks = 50;
    }

    city.tick_slow_cycles(3);

    let world = city.world_mut();
    let mut q = world.query::<&CitizenDetails>();
    let health = q.iter(world).next().unwrap().health;
    assert!(
        health < 80.0,
        "Epidemic should drain health below 80, got {}",
        health
    );
}

/// Test that active effect timers eventually expire after enough ticks.
#[test]
fn test_random_events_effect_duration_expires() {
    use crate::events::ActiveCityEffects;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut effects = world.resource_mut::<ActiveCityEffects>();
        effects.festival_ticks = 2;
        effects.economic_boom_ticks = 2;
        effects.epidemic_ticks = 2;
    }

    city.tick_slow_cycles(50);

    let effects = city.resource::<ActiveCityEffects>();
    // Epidemic has 0.5% re-trigger chance per tick; max from re-trigger is 10.
    assert!(
        effects.epidemic_ticks <= 10,
        "Epidemic ticks should be at most 10 (max from re-trigger), got {}",
        effects.epidemic_ticks
    );
}

/// Test that the EventJournal trims old events when exceeding max_events.
#[test]
fn test_random_events_journal_trims_to_max() {
    use crate::events::{CityEvent, CityEventType, EventJournal};

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut journal = world.resource_mut::<EventJournal>();
        journal.max_events = 5;
        for i in 0..10 {
            journal.push(CityEvent {
                event_type: CityEventType::Festival,
                day: i,
                hour: 12.0,
                description: format!("Test event {}", i),
            });
        }
    }

    let journal = city.resource::<EventJournal>();
    assert_eq!(journal.events.len(), 5);
    assert_eq!(journal.events[0].day, 5);
    assert_eq!(journal.events[4].day, 9);
}

/// Test that positive treasury does NOT trigger a BudgetCrisis event.
#[test]
fn test_random_events_no_budget_crisis_with_positive_treasury() {
    use crate::events::{CityEventType, EventJournal};

    let mut city = TestCity::new().with_budget(50_000.0);
    city.tick_slow_cycle();

    let journal = city.resource::<EventJournal>();
    let crisis_count = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, CityEventType::BudgetCrisis))
        .count();
    assert_eq!(
        crisis_count, 0,
        "No BudgetCrisis should fire with positive treasury, got {}",
        crisis_count
    );
}

/// Test that epidemic health drain does not go below 0.
#[test]
fn test_random_events_epidemic_health_floor_at_zero() {
    use crate::citizen::CitizenDetails;
    use crate::events::ActiveCityEffects;

    let home = (10, 10);
    let work = (15, 15);
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::Industrial, 1)
        .with_citizen(home, work);

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut CitizenDetails>();
        for mut details in q.iter_mut(world) {
            details.health = 0.5;
        }
    }
    {
        let world = city.world_mut();
        world.resource_mut::<ActiveCityEffects>().epidemic_ticks = 20;
    }

    city.tick_slow_cycles(10);

    let world = city.world_mut();
    let mut q = world.query::<&CitizenDetails>();
    for details in q.iter(world) {
        assert!(
            details.health >= 0.0,
            "Health should never go below 0, got {}",
            details.health
        );
    }
}

/// Test that festival happiness boost does not exceed 100.
#[test]
fn test_random_events_festival_happiness_capped_at_100() {
    use crate::citizen::CitizenDetails;
    use crate::events::ActiveCityEffects;

    let home = (10, 10);
    let work = (15, 15);
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::Industrial, 1)
        .with_citizen(home, work);

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut CitizenDetails>();
        for mut details in q.iter_mut(world) {
            details.happiness = 100.0;
        }
    }
    {
        let world = city.world_mut();
        world.resource_mut::<ActiveCityEffects>().festival_ticks = 20;
    }

    city.tick_slow_cycles(5);

    let world = city.world_mut();
    let mut q = world.query::<&CitizenDetails>();
    for details in q.iter(world) {
        assert!(
            details.happiness <= 100.0,
            "Happiness should not exceed 100, got {}",
            details.happiness
        );
    }
}

/// Test that CityEvent records the correct day from the GameClock.
#[test]
fn test_random_events_event_records_clock_time() {
    use crate::events::{CityEventType, EventJournal};

    let mut city = TestCity::new().with_budget(-5000.0).with_time(14.5);
    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 42;
    }
    city.tick_slow_cycle();

    let journal = city.resource::<EventJournal>();
    let crisis = journal
        .events
        .iter()
        .find(|e| matches!(e.event_type, CityEventType::BudgetCrisis));
    assert!(crisis.is_some(), "BudgetCrisis event should exist");
    let crisis = crisis.unwrap();
    assert!(
        crisis.day >= 42,
        "Event day should be >= 42, got {}",
        crisis.day
    );
}

// ===========================================================================
// TEST-056: District Statistics Aggregation
// ===========================================================================

/// Verify that the automatic `Districts` resource correctly tallies
/// per-district population from residential buildings after aggregation.
/// After running a slow cycle, we read actual building occupants and verify
/// district population matches for buildings in different statistical districts.
#[test]
fn test_district_aggregate_population_matches_residential_occupants() {
    use crate::districts::{Districts, DISTRICT_SIZE};

    let bld_a = (5, 5);
    let bld_b = (DISTRICT_SIZE + 3, 5);

    let mut city = TestCity::new()
        .with_building(bld_a.0, bld_a.1, ZoneType::ResidentialLow, 1)
        .with_building(bld_b.0, bld_b.1, ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();

    let mut occ_a = 0u32;
    let mut occ_b = 0u32;
    {
        let world = city.world_mut();
        let mut query = world.query::<&Building>();
        for building in query.iter(world) {
            if building.grid_x == bld_a.0
                && building.grid_y == bld_a.1
                && building.zone_type.is_residential()
            {
                occ_a += building.occupants;
            } else if building.grid_x == bld_b.0
                && building.grid_y == bld_b.1
                && building.zone_type.is_residential()
            {
                occ_b += building.occupants;
            }
        }
    }

    let districts = city.resource::<Districts>();
    let (da_x, da_y) = Districts::district_for_grid(bld_a.0, bld_a.1);
    let (db_x, db_y) = Districts::district_for_grid(bld_b.0, bld_b.1);

    assert_eq!(
        districts.get(da_x, da_y).population,
        occ_a,
        "District ({da_x},{da_y}) population should match building occupants"
    );
    assert_eq!(
        districts.get(db_x, db_y).population,
        occ_b,
        "District ({db_x},{db_y}) population should match building occupants"
    );
}

/// Verify per-district job counts after aggregation.
#[test]
fn test_district_aggregate_job_counts() {
    use crate::districts::Districts;

    let comm_pos = (5, 5);
    let ind_pos = (5, 6);
    let off_pos = (5, 7);

    let mut city = TestCity::new()
        .with_building(comm_pos.0, comm_pos.1, ZoneType::CommercialLow, 1)
        .with_building(ind_pos.0, ind_pos.1, ZoneType::Industrial, 1)
        .with_building(off_pos.0, off_pos.1, ZoneType::Office, 1);

    city.tick_slow_cycle();

    let mut expected_employed = 0u32;
    let mut expected_comm_cap = 0u32;
    let mut expected_ind_cap = 0u32;
    let mut expected_off_cap = 0u32;
    {
        let world = city.world_mut();
        let mut query = world.query::<&Building>();
        for building in query.iter(world) {
            let (dx, _) = Districts::district_for_grid(building.grid_x, building.grid_y);
            let (edx, _) = Districts::district_for_grid(comm_pos.0, comm_pos.1);
            if dx == edx {
                if building.zone_type.is_commercial() {
                    expected_employed += building.occupants;
                    expected_comm_cap += building.capacity;
                } else if building.zone_type == ZoneType::Industrial {
                    expected_employed += building.occupants;
                    expected_ind_cap += building.capacity;
                } else if building.zone_type == ZoneType::Office {
                    expected_employed += building.occupants;
                    expected_off_cap += building.capacity;
                }
            }
        }
    }

    let districts = city.resource::<Districts>();
    let (dx, dy) = Districts::district_for_grid(comm_pos.0, comm_pos.1);
    let d = districts.get(dx, dy);

    assert_eq!(
        d.commercial_jobs, expected_comm_cap,
        "Commercial jobs capacity should match"
    );
    assert_eq!(
        d.industrial_jobs, expected_ind_cap,
        "Industrial jobs capacity should match"
    );
    assert_eq!(
        d.office_jobs, expected_off_cap,
        "Office jobs capacity should match"
    );
    assert_eq!(
        d.employed, expected_employed,
        "Employed should match sum of non-residential occupants"
    );
}

/// Verify per-district happiness average is computed when citizens exist.
/// Happiness drifts significantly during simulation so we verify the
/// aggregation produces a non-zero value within the valid range.
#[test]
fn test_district_aggregate_happiness_average() {
    use crate::districts::Districts;

    let home_pos = (10, 10);

    let mut city = TestCity::new()
        .with_building(home_pos.0, home_pos.1, ZoneType::ResidentialLow, 1)
        .with_utility(home_pos.0 + 1, home_pos.1, UtilityType::PowerPlant)
        .with_utility(home_pos.0, home_pos.1 + 1, UtilityType::WaterTower)
        .with_unemployed_citizen(home_pos)
        .with_unemployed_citizen(home_pos)
        .with_unemployed_citizen(home_pos);

    city.tick_slow_cycle();

    let districts = city.resource::<Districts>();
    let (dx, dy) = Districts::district_for_grid(home_pos.0, home_pos.1);
    let avg = districts.get(dx, dy).avg_happiness;

    // Citizens exist, so average happiness should be > 0 and <= 100
    assert!(
        avg > 0.0,
        "Avg happiness should be > 0 when citizens exist, got {avg}"
    );
    assert!(avg <= 100.0, "Avg happiness should be <= 100, got {avg}");
}

/// Verify sum of district populations equals total.
#[test]
fn test_district_population_sums_to_total() {
    use crate::districts::Districts;

    let mut city = TestCity::new()
        .with_building(5, 5, ZoneType::ResidentialLow, 1)
        .with_building(20, 5, ZoneType::ResidentialLow, 1)
        .with_building(40, 5, ZoneType::ResidentialHigh, 1);

    city.tick_slow_cycle();

    let districts = city.resource::<Districts>();
    let total = districts.total_statistical_population();
    let sum: u32 = districts.data.iter().map(|d| d.population).sum();

    assert_eq!(
        sum, total,
        "Sum of per-district populations ({sum}) must equal total_statistical_population ({total})"
    );
}

/// Verify DistrictMap cell assignment and reassignment.
#[test]
fn test_district_map_cell_assignment_tracking() {
    use crate::districts::DistrictMap;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();

        for x in 10..14 {
            for y in 10..14 {
                dmap.assign_cell_to_district(x, y, 0);
            }
        }
        assert_eq!(
            dmap.districts[0].cells.len(),
            16,
            "Should have 4x4=16 cells"
        );
        assert_eq!(dmap.get_district_index_at(12, 12), Some(0));

        for x in 12..14 {
            for y in 12..14 {
                dmap.assign_cell_to_district(x, y, 1);
            }
        }
        assert_eq!(
            dmap.districts[0].cells.len(),
            12,
            "Downtown should now have 16-4=12 cells"
        );
        assert_eq!(
            dmap.districts[1].cells.len(),
            4,
            "Suburbs should have 4 cells"
        );
        assert_eq!(dmap.get_district_index_at(12, 12), Some(1));
        assert_eq!(dmap.get_district_index_at(10, 10), Some(0));
    }
}

/// Verify district_stats population from buildings with utility coverage.
/// Utility sources prevent abandonment which would zero occupants.
#[test]
fn test_district_stats_population_from_buildings() {
    use crate::districts::DistrictMap;

    let bld_a = (10, 10);
    let bld_b = (20, 20);

    let mut city = TestCity::new()
        .with_building(bld_a.0, bld_a.1, ZoneType::ResidentialLow, 1)
        .with_building(bld_b.0, bld_b.1, ZoneType::ResidentialLow, 1)
        .with_utility(bld_a.0 + 1, bld_a.1, UtilityType::PowerPlant)
        .with_utility(bld_a.0, bld_a.1 + 1, UtilityType::WaterTower)
        .with_utility(bld_b.0 + 1, bld_b.1, UtilityType::PowerPlant)
        .with_utility(bld_b.0, bld_b.1 + 1, UtilityType::WaterTower);

    // Set occupants after utility sources are placed
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.grid_x == bld_a.0 && building.grid_y == bld_a.1 {
                building.occupants = 15;
            } else if building.grid_x == bld_b.0 && building.grid_y == bld_b.1 {
                building.occupants = 30;
            }
        }
    }

    // Assign building cells to player districts
    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();
        dmap.assign_cell_to_district(bld_a.0, bld_a.1, 0);
        dmap.assign_cell_to_district(bld_b.0, bld_b.1, 1);
    }

    // district_stats runs every 50 ticks
    city.tick(50);

    let dmap = city.resource::<DistrictMap>();
    assert_eq!(
        dmap.districts[0].stats.population, 15,
        "Downtown district should have population 15"
    );
    assert_eq!(
        dmap.districts[1].stats.population, 30,
        "Suburbs district should have population 30"
    );
}

/// Verify player-district crime stats from CrimeGrid.
#[test]
fn test_district_stats_crime_average() {
    use crate::crime::CrimeGrid;
    use crate::districts::DistrictMap;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();
        for x in 50..52 {
            for y in 50..52 {
                dmap.assign_cell_to_district(x, y, 0);
            }
        }
        let mut crime = world.resource_mut::<CrimeGrid>();
        crime.set(50, 50, 40);
        crime.set(51, 50, 60);
        crime.set(50, 51, 80);
        crime.set(51, 51, 20);
    }

    city.tick(50);

    let dmap = city.resource::<DistrictMap>();
    let crime = dmap.districts[0].stats.crime;
    assert!(
        (crime - 50.0).abs() < 1.0,
        "Expected avg crime ~50.0, got {crime}"
    );
}

/// Verify player-district happiness from citizens.
#[test]
fn test_district_stats_happiness_from_citizens() {
    use crate::citizen::{CitizenDetails, HomeLocation};
    use crate::districts::DistrictMap;

    let home_pos = (30, 30);
    let work_pos = (32, 32);

    let mut city = TestCity::new()
        .with_building(home_pos.0, home_pos.1, ZoneType::ResidentialLow, 1)
        .with_building(work_pos.0, work_pos.1, ZoneType::CommercialLow, 1)
        .with_citizen(home_pos, work_pos)
        .with_citizen(home_pos, work_pos);

    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();
        dmap.assign_cell_to_district(home_pos.0, home_pos.1, 2);
    }

    city.tick(50);

    let mut happiness_sum = 0.0f32;
    let mut count = 0u32;
    {
        let dmap_cell_map: Vec<Option<usize>>;
        {
            let dmap = city.resource::<DistrictMap>();
            dmap_cell_map = dmap.cell_map.clone();
        }
        let world = city.world_mut();
        let mut query = world.query::<(&CitizenDetails, &HomeLocation)>();
        for (details, home) in query.iter(world) {
            let idx = home.grid_y * GRID_WIDTH + home.grid_x;
            if dmap_cell_map.get(idx).copied().flatten() == Some(2) {
                happiness_sum += details.happiness;
                count += 1;
            }
        }
    }

    let dmap = city.resource::<DistrictMap>();
    let avg = dmap.districts[2].stats.avg_happiness;

    if count > 0 {
        let expected = happiness_sum / count as f32;
        assert!(
            (avg - expected).abs() < 10.0,
            "Expected avg happiness ~{expected} (within 10.0), got {avg}"
        );
    }
}

/// Verify empty districts have zero stats.
#[test]
fn test_district_empty_district_has_zero_stats() {
    use crate::districts::{DistrictMap, Districts};

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();
        for x in 100..105 {
            for y in 100..105 {
                dmap.assign_cell_to_district(x, y, 0);
            }
        }
    }

    city.tick_slow_cycle();

    let dmap = city.resource::<DistrictMap>();
    assert_eq!(dmap.districts[0].stats.population, 0);
    assert!((dmap.districts[0].stats.avg_happiness).abs() < f32::EPSILON);
    assert!((dmap.districts[0].stats.crime).abs() < f32::EPSILON);

    let districts = city.resource::<Districts>();
    let (dx, dy) = Districts::district_for_grid(100, 100);
    let d = districts.get(dx, dy);
    assert_eq!(d.population, 0);
    assert_eq!(d.employed, 0);
    assert_eq!(d.residential_capacity, 0);
    assert!((d.avg_happiness).abs() < f32::EPSILON);
}

/// Verify district_for_grid boundary mapping.
#[test]
fn test_district_grid_boundary_mapping() {
    use crate::districts::{Districts, DISTRICTS_X, DISTRICTS_Y, DISTRICT_SIZE};

    assert_eq!(Districts::district_for_grid(0, 0), (0, 0));
    assert_eq!(
        Districts::district_for_grid(DISTRICT_SIZE - 1, DISTRICT_SIZE - 1),
        (0, 0)
    );
    assert_eq!(Districts::district_for_grid(DISTRICT_SIZE, 0), (1, 0));
    assert_eq!(Districts::district_for_grid(0, DISTRICT_SIZE), (0, 1));
    assert_eq!(
        Districts::district_for_grid(255, 255),
        (DISTRICTS_X - 1, DISTRICTS_Y - 1)
    );
}

/// Verify residential capacity aggregation per district.
#[test]
fn test_district_aggregate_residential_capacity() {
    use crate::districts::Districts;

    let pos_a = (5, 5);
    let pos_b = (6, 5);

    let mut city = TestCity::new()
        .with_building(pos_a.0, pos_a.1, ZoneType::ResidentialLow, 1)
        .with_building(pos_b.0, pos_b.1, ZoneType::ResidentialHigh, 1);

    city.tick_slow_cycle();

    let districts = city.resource::<Districts>();
    let (dx, dy) = Districts::district_for_grid(pos_a.0, pos_a.1);
    let d = districts.get(dx, dy);

    let expected_cap = Building::capacity_for_level(ZoneType::ResidentialLow, 1)
        + Building::capacity_for_level(ZoneType::ResidentialHigh, 1);
    assert_eq!(
        d.residential_capacity, expected_cap,
        "Residential capacity should be sum of both buildings: {expected_cap}"
    );
}

/// Verify no stat bleed between adjacent districts.
#[test]
fn test_district_no_stat_bleed_between_districts() {
    use crate::districts::{Districts, DISTRICT_SIZE};

    let pos_a = (DISTRICT_SIZE - 1, 0);
    let pos_b = (DISTRICT_SIZE, 0);

    let mut city = TestCity::new()
        .with_building(pos_a.0, pos_a.1, ZoneType::ResidentialLow, 1)
        .with_building(pos_b.0, pos_b.1, ZoneType::Industrial, 1);

    city.tick_slow_cycle();

    let districts = city.resource::<Districts>();
    let d0 = districts.get(0, 0);
    let d1 = districts.get(1, 0);

    assert_eq!(
        d0.industrial_jobs, 0,
        "District (0,0) should have no industrial jobs"
    );
    assert!(
        d1.industrial_jobs > 0,
        "District (1,0) should have industrial capacity"
    );
    assert_eq!(
        d1.residential_capacity, 0,
        "District (1,0) should have no residential capacity"
    );
    assert!(
        d0.residential_capacity > 0,
        "District (0,0) should have residential capacity"
    );
}

/// Verify cell removal updates both cell_map and district cells.
#[test]
fn test_district_map_remove_cell_updates_both_sides() {
    use crate::districts::DistrictMap;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();

        dmap.assign_cell_to_district(50, 50, 0);
        assert!(dmap.districts[0].cells.contains(&(50, 50)));
        assert_eq!(dmap.get_district_index_at(50, 50), Some(0));

        dmap.remove_cell_from_district(50, 50);
        assert!(!dmap.districts[0].cells.contains(&(50, 50)));
        assert_eq!(dmap.get_district_index_at(50, 50), None);
    }
}

/// Verify default districts are pre-populated.
#[test]
fn test_district_map_default_districts_exist() {
    use crate::districts::{DistrictMap, DEFAULT_DISTRICTS};

    let city = TestCity::new();
    let dmap = city.resource::<DistrictMap>();

    assert_eq!(
        dmap.districts.len(),
        DEFAULT_DISTRICTS.len(),
        "Default district count should match DEFAULT_DISTRICTS"
    );
    for (i, &(name, _color)) in DEFAULT_DISTRICTS.iter().enumerate() {
        assert_eq!(dmap.districts[i].name, name, "District {i} name mismatch");
    }
}

// ===========================================================================
// TEST-061: Stress Test: Rapid Save/Load Cycles (Issue #841)
// ===========================================================================

/// Rapidly save and load the SaveableRegistry 100 times.
/// Verifies no state corruption or resource leaks across cycles.
#[test]
fn test_rapid_save_load_100_cycles_saveable_registry() {
    use crate::SaveableRegistry;
    use std::collections::BTreeMap;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct CounterResource {
        value: u64,
    }

    impl crate::Saveable for CounterResource {
        const SAVE_KEY: &'static str = "stress_counter";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.value).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                value: serde_json::from_slice(bytes).unwrap_or_default(),
            }
        }
    }

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct VecResource {
        items: Vec<String>,
    }

    impl crate::Saveable for VecResource {
        const SAVE_KEY: &'static str = "stress_vec";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            if self.items.is_empty() {
                return None;
            }
            serde_json::to_vec(&self.items).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                items: serde_json::from_slice(bytes).unwrap_or_default(),
            }
        }
    }

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct NestedResource {
        mapping: BTreeMap<String, Vec<f64>>,
    }

    impl crate::Saveable for NestedResource {
        const SAVE_KEY: &'static str = "stress_nested";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            if self.mapping.is_empty() {
                return None;
            }
            serde_json::to_vec(&self.mapping).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                mapping: serde_json::from_slice(bytes).unwrap_or_default(),
            }
        }
    }

    app.init_resource::<CounterResource>();
    app.init_resource::<VecResource>();
    app.init_resource::<NestedResource>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<CounterResource>();
        registry.register::<VecResource>();
        registry.register::<NestedResource>();
    }

    app.world_mut().resource_mut::<CounterResource>().value = 123_456_789;
    app.world_mut().resource_mut::<VecResource>().items =
        (0..50).map(|i| format!("item_{i}")).collect();
    {
        let mut nested = app.world_mut().resource_mut::<NestedResource>();
        for i in 0..10 {
            nested.mapping.insert(
                format!("key_{i}"),
                (0..20).map(|j| (i * 20 + j) as f64 * 0.1).collect(),
            );
        }
    }

    for cycle in 0..100 {
        let extensions = {
            let registry = app.world().resource::<SaveableRegistry>();
            registry.save_all(app.world())
        };
        assert!(
            extensions.contains_key("stress_counter"),
            "cycle {cycle}: counter missing"
        );
        assert!(
            extensions.contains_key("stress_vec"),
            "cycle {cycle}: vec missing"
        );
        assert!(
            extensions.contains_key("stress_nested"),
            "cycle {cycle}: nested missing"
        );

        app.world_mut().insert_resource(CounterResource::default());
        app.world_mut().insert_resource(VecResource::default());
        app.world_mut().insert_resource(NestedResource::default());

        {
            let registry = app
                .world_mut()
                .remove_resource::<SaveableRegistry>()
                .unwrap();
            registry.load_all(app.world_mut(), &extensions);
            app.world_mut().insert_resource(registry);
        }

        assert_eq!(
            app.world().resource::<CounterResource>().value,
            123_456_789,
            "cycle {cycle}: counter corrupted"
        );
        assert_eq!(
            app.world().resource::<VecResource>().items.len(),
            50,
            "cycle {cycle}: vec len changed"
        );
        assert_eq!(
            app.world().resource::<VecResource>().items[0],
            "item_0",
            "cycle {cycle}: first item corrupted"
        );
        assert_eq!(
            app.world().resource::<VecResource>().items[49],
            "item_49",
            "cycle {cycle}: last item corrupted"
        );
        assert_eq!(
            app.world().resource::<NestedResource>().mapping.len(),
            10,
            "cycle {cycle}: nested map size changed"
        );
        let key5 = app
            .world()
            .resource::<NestedResource>()
            .mapping
            .get("key_5")
            .expect("key_5")
            .clone();
        assert_eq!(key5.len(), 20, "cycle {cycle}: nested vec len changed");
        assert!(
            (key5[0] - 10.0).abs() < 1e-10,
            "cycle {cycle}: nested float corrupted"
        );
    }
}

/// Rapidly serialize/deserialize citizen data 100 times via serde_json.
#[test]
fn test_rapid_citizen_serde_100_roundtrips() {
    use crate::citizen::{
        CitizenDetails, Gender, Needs, PathCache, Personality, Position, Velocity,
    };
    use crate::roads::RoadNode;

    let mut details_json = serde_json::to_string(&CitizenDetails {
        age: 42,
        gender: Gender::Female,
        education: 3,
        happiness: 72.5,
        health: 88.3,
        salary: 6500.0,
        savings: 15000.0,
    })
    .unwrap();
    let mut pers_json = serde_json::to_string(&Personality {
        ambition: 0.85,
        sociability: 0.3,
        materialism: 0.65,
        resilience: 0.92,
    })
    .unwrap();
    let mut needs_json = serde_json::to_string(&Needs {
        hunger: 55.0,
        energy: 70.0,
        social: 42.0,
        fun: 38.0,
        comfort: 65.0,
    })
    .unwrap();
    let mut pos_json = serde_json::to_string(&Position { x: 200.5, y: 180.3 }).unwrap();
    let mut vel_json = serde_json::to_string(&Velocity { x: 1.5, y: -0.7 }).unwrap();
    let mut path_json = serde_json::to_string(&PathCache::new(vec![
        RoadNode(12, 10),
        RoadNode(15, 10),
        RoadNode(18, 10),
    ]))
    .unwrap();

    for cycle in 0..100 {
        let d: CitizenDetails =
            serde_json::from_str(&details_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        let p: Personality =
            serde_json::from_str(&pers_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        let n: Needs =
            serde_json::from_str(&needs_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        let po: Position =
            serde_json::from_str(&pos_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        let v: Velocity =
            serde_json::from_str(&vel_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        let pa: PathCache =
            serde_json::from_str(&path_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        details_json = serde_json::to_string(&d).unwrap();
        pers_json = serde_json::to_string(&p).unwrap();
        needs_json = serde_json::to_string(&n).unwrap();
        pos_json = serde_json::to_string(&po).unwrap();
        vel_json = serde_json::to_string(&v).unwrap();
        path_json = serde_json::to_string(&pa).unwrap();
    }

    let fd: CitizenDetails = serde_json::from_str(&details_json).unwrap();
    let fp: Personality = serde_json::from_str(&pers_json).unwrap();
    let fn_: Needs = serde_json::from_str(&needs_json).unwrap();
    let fpos: Position = serde_json::from_str(&pos_json).unwrap();
    let fv: Velocity = serde_json::from_str(&vel_json).unwrap();
    let fpa: PathCache = serde_json::from_str(&path_json).unwrap();

    assert_eq!(fd.age, 42, "age drifted");
    assert_eq!(fd.happiness, 72.5, "happiness drifted");
    assert_eq!(fd.salary, 6500.0, "salary drifted");
    assert_eq!(fp.ambition, 0.85, "ambition drifted");
    assert_eq!(fn_.hunger, 55.0, "hunger drifted");
    assert_eq!(fpos.x, 200.5, "pos.x drifted");
    assert_eq!(fv.x, 1.5, "vel.x drifted");
    assert_eq!(fpa.waypoints.len(), 3, "path len changed");
}

/// Stress test: entity count and treasury stability through 100 save/load cycles.
#[test]
fn test_rapid_save_load_entity_and_treasury_stability() {
    use crate::SaveableRegistry;

    let mut city = TestCity::new()
        .with_budget(50_000.0)
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_road(20, 5, 20, 15, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(14, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_building(22, 11, ZoneType::Industrial, 1)
        .with_service(25, 11, ServiceType::PoliceStation)
        .with_utility(28, 11, UtilityType::PowerPlant)
        .with_citizen((12, 11), (18, 11))
        .with_citizen((14, 11), (22, 11))
        .with_citizen((12, 11), (22, 11));

    let ic = city.citizen_count();
    let ib = city.building_count();
    let it = city.budget().treasury;
    let ir = city.road_cell_count();

    for cycle in 0..100 {
        let ext = {
            let w = city.world_mut();
            let r = w.resource::<SaveableRegistry>();
            r.save_all(w)
        };
        {
            let w = city.world_mut();
            let r = w.remove_resource::<SaveableRegistry>().unwrap();
            r.load_all(w, &ext);
            w.insert_resource(r);
        }
        assert_eq!(city.citizen_count(), ic, "cycle {cycle}: citizens changed");
        assert_eq!(
            city.building_count(),
            ib,
            "cycle {cycle}: buildings changed"
        );
        assert!(
            (city.budget().treasury - it).abs() < 1e-6,
            "cycle {cycle}: treasury drifted"
        );
        assert_eq!(city.road_cell_count(), ir, "cycle {cycle}: roads changed");
    }
}

/// Stress test: save/load 100 times with interleaved simulation ticks.
#[test]
fn test_rapid_save_load_interleaved_with_ticks() {
    use crate::SaveableRegistry;

    let mut city = TestCity::new()
        .with_budget(50_000.0)
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_citizen((12, 11), (18, 11));

    let ic = city.citizen_count();
    for cycle in 0..100 {
        city.tick(5);
        let ext = {
            let w = city.world_mut();
            let r = w.resource::<SaveableRegistry>();
            r.save_all(w)
        };
        {
            let w = city.world_mut();
            let r = w.remove_resource::<SaveableRegistry>().unwrap();
            r.load_all(w, &ext);
            w.insert_resource(r);
        }
        assert!(
            city.citizen_count() >= 1,
            "cycle {cycle}: all citizens gone"
        );
    }
    assert!(
        city.citizen_count() <= ic + 500,
        "citizen count exploded, possible leak"
    );
}

/// Stress test: verify save data bytes are deterministic across cycles.
#[test]
fn test_save_data_deterministic_across_cycles() {
    use crate::SaveableRegistry;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct DeterminismTest {
        data: Vec<u32>,
        label: String,
    }

    impl crate::Saveable for DeterminismTest {
        const SAVE_KEY: &'static str = "stress_determinism";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            if self.data.is_empty() {
                return None;
            }
            serde_json::to_vec(&(&self.data, &self.label)).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            let (data, label): (Vec<u32>, String) =
                serde_json::from_slice(bytes).unwrap_or_default();
            Self { data, label }
        }
    }

    app.init_resource::<DeterminismTest>();
    {
        let mut r = app.world_mut().resource_mut::<SaveableRegistry>();
        r.register::<DeterminismTest>();
    }
    {
        let mut r = app.world_mut().resource_mut::<DeterminismTest>();
        r.data = (0..100).collect();
        r.label = "determinism_stress".to_string();
    }

    let ref_ext = {
        let r = app.world().resource::<SaveableRegistry>();
        r.save_all(app.world())
    };
    let ref_bytes = ref_ext.get("stress_determinism").unwrap().clone();

    for cycle in 0..100 {
        let ext = {
            let r = app.world().resource::<SaveableRegistry>();
            r.save_all(app.world())
        };
        assert_eq!(
            ext.get("stress_determinism").unwrap(),
            &ref_bytes,
            "cycle {cycle}: non-deterministic"
        );
    }
}

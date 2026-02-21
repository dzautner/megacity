use crate::buildings::Building;
use crate::grid::{RoadType, ZoneType};
use crate::test_harness::TestCity;

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

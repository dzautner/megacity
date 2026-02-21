use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;
use crate::utilities::{UtilitySource, UtilityType};

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

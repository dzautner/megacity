use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::test_harness::TestCity;

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

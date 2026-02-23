use crate::grid::{RoadType, ZoneType};
use crate::immigration::CityAttractiveness;use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

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
    // Prevent emigration during the tick run.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

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

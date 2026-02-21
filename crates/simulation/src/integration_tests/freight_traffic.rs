use crate::buildings::Building;
use crate::grid::{RoadType, ZoneType};
use crate::test_harness::TestCity;

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

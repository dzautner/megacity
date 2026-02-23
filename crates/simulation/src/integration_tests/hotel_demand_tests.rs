//! Integration tests for hotel demand and capacity system (SVC-019).

use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::hotel_demand::HotelDemandState;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::tourism::Tourism;

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_hotel_demand_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<HotelDemandState>();
}

#[test]
fn test_hotel_demand_default_state() {
    let city = TestCity::new();
    let state = city.resource::<HotelDemandState>();
    assert_eq!(state.total_capacity, 0);
    assert_eq!(state.hotel_count, 0);
    assert_eq!(state.rooms_demanded, 0);
    assert_eq!(state.occupancy_rate, 0.0);
    assert_eq!(state.monthly_tax_revenue, 0.0);
}

// ====================================================================
// No hotels scenario
// ====================================================================

#[test]
fn test_hotel_demand_no_commercial_buildings() {
    let mut city = TestCity::new();
    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert_eq!(state.total_capacity, 0);
    assert_eq!(state.hotel_count, 0);
    assert_eq!(state.monthly_tax_revenue, 0.0);
}

// ====================================================================
// Hotel capacity from commercial buildings
// ====================================================================

#[test]
fn test_hotel_demand_commercial_building_adds_capacity() {
    let mut city = TestCity::new().with_building(50, 50, ZoneType::CommercialHigh, 1);

    // Manually set occupants > 0 so the building counts as a hotel
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut b in query.iter_mut(world) {
            if b.zone_type == ZoneType::CommercialHigh {
                b.occupants = 5;
            }
        }
    }

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert_eq!(state.hotel_count, 1);
    assert_eq!(state.total_capacity, 50); // Level 1 = 50 rooms
}

#[test]
fn test_hotel_demand_higher_level_more_capacity() {
    let mut city = TestCity::new().with_building(50, 50, ZoneType::CommercialHigh, 3);

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut b in query.iter_mut(world) {
            if b.zone_type == ZoneType::CommercialHigh {
                b.occupants = 10;
            }
        }
    }

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert_eq!(state.total_capacity, 200); // Level 3 = 200 rooms
}

#[test]
fn test_hotel_demand_multiple_buildings_sum_capacity() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialHigh, 1)
        .with_building(52, 52, ZoneType::CommercialHigh, 2);

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut b in query.iter_mut(world) {
            if b.zone_type == ZoneType::CommercialHigh {
                b.occupants = 5;
            }
        }
    }

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert_eq!(state.hotel_count, 2);
    assert_eq!(state.total_capacity, 170); // 50 + 120
}

#[test]
fn test_hotel_demand_non_commercial_not_counted() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 3)
        .with_building(52, 52, ZoneType::Industrial, 2);

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert_eq!(state.hotel_count, 0);
    assert_eq!(state.total_capacity, 0);
}

#[test]
fn test_hotel_demand_empty_commercial_not_counted() {
    // Commercial building with 0 occupants should not count as a hotel
    let mut city = TestCity::new().with_building(50, 50, ZoneType::CommercialHigh, 1);

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert_eq!(state.hotel_count, 0);
    assert_eq!(state.total_capacity, 0);
}

// ====================================================================
// Attractiveness
// ====================================================================

#[test]
fn test_hotel_demand_attractiveness_from_landmarks() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::Museum)
        .with_service(55, 55, ServiceType::Stadium);

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert!(
        state.attractiveness_score > 0.0,
        "Landmarks should increase attractiveness, got {}",
        state.attractiveness_score,
    );
}

#[test]
fn test_hotel_demand_no_services_low_attractiveness() {
    let mut city = TestCity::new();
    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert_eq!(state.attractiveness_score, 0.0);
}

// ====================================================================
// Occupancy and revenue
// ====================================================================

#[test]
fn test_hotel_demand_occupancy_rate_bounded() {
    let mut city = TestCity::new().with_building(50, 50, ZoneType::CommercialHigh, 1);

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut b in query.iter_mut(world) {
            if b.zone_type == ZoneType::CommercialHigh {
                b.occupants = 5;
            }
        }
    }

    // Inject high tourism to get demand > capacity
    {
        let world = city.world_mut();
        let mut tourism = world.resource_mut::<Tourism>();
        tourism.monthly_visitors = 50_000;
    }

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert!(
        state.occupancy_rate <= 1.0,
        "Occupancy rate should be capped at 1.0, got {}",
        state.occupancy_rate,
    );
}

#[test]
fn test_hotel_demand_tax_revenue_when_occupied() {
    let mut city = TestCity::new().with_building(50, 50, ZoneType::CommercialHigh, 3);

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut b in query.iter_mut(world) {
            if b.zone_type == ZoneType::CommercialHigh {
                b.occupants = 10;
            }
        }
    }

    // Inject tourism visitors
    {
        let world = city.world_mut();
        let mut tourism = world.resource_mut::<Tourism>();
        tourism.monthly_visitors = 5_000;
    }

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert!(
        state.monthly_tax_revenue > 0.0,
        "Should generate tax revenue when rooms are occupied, got {}",
        state.monthly_tax_revenue,
    );
}

#[test]
fn test_hotel_demand_no_revenue_without_visitors() {
    let mut city = TestCity::new().with_building(50, 50, ZoneType::CommercialHigh, 1);

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut b in query.iter_mut(world) {
            if b.zone_type == ZoneType::CommercialHigh {
                b.occupants = 5;
            }
        }
    }

    // Ensure zero visitors
    {
        let world = city.world_mut();
        let mut tourism = world.resource_mut::<Tourism>();
        tourism.monthly_visitors = 0;
    }

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert_eq!(
        state.monthly_tax_revenue, 0.0,
        "No visitors should mean no tax revenue",
    );
}

// ====================================================================
// Over-capacity and under-capacity
// ====================================================================

#[test]
fn test_hotel_demand_lost_revenue_when_over_capacity() {
    let mut city = TestCity::new().with_building(50, 50, ZoneType::CommercialHigh, 1); // 50 rooms

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut b in query.iter_mut(world) {
            if b.zone_type == ZoneType::CommercialHigh {
                b.occupants = 5;
            }
        }
    }

    // Inject very high tourism to exceed 50-room capacity
    {
        let world = city.world_mut();
        let mut tourism = world.resource_mut::<Tourism>();
        tourism.monthly_visitors = 50_000;
    }

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert!(
        state.lost_revenue > 0.0,
        "Over-capacity should produce lost revenue, got {}",
        state.lost_revenue,
    );
    assert!(state.is_over_capacity());
}

#[test]
fn test_hotel_demand_wasted_investment_when_under_capacity() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialHigh, 5) // 500 rooms
        .with_building(52, 52, ZoneType::CommercialHigh, 5); // 500 rooms = 1000 total

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut b in query.iter_mut(world) {
            if b.zone_type == ZoneType::CommercialHigh {
                b.occupants = 10;
            }
        }
    }

    // Only a small number of visitors relative to 1000-room capacity
    {
        let world = city.world_mut();
        let mut tourism = world.resource_mut::<Tourism>();
        tourism.monthly_visitors = 100;
    }

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert!(
        state.wasted_investment > 0.0,
        "Under-capacity should produce wasted investment, got {}",
        state.wasted_investment,
    );
}

// ====================================================================
// Saveable round-trip
// ====================================================================

#[test]
fn test_hotel_demand_saveable_roundtrip() {
    use crate::Saveable;

    let mut state = HotelDemandState::default();
    state.total_capacity = 300;
    state.hotel_count = 5;
    state.rooms_demanded = 200;
    state.occupancy_rate = 0.67;
    state.monthly_tax_revenue = 1500.0;
    state.hotel_tax_rate = 0.15;

    let bytes = state
        .save_to_bytes()
        .expect("Should serialize non-default state");
    let restored = HotelDemandState::load_from_bytes(&bytes);

    assert_eq!(restored.total_capacity, 300);
    assert_eq!(restored.hotel_count, 5);
    assert_eq!(restored.rooms_demanded, 200);
    assert!((restored.occupancy_rate - 0.67).abs() < 0.01);
    assert!((restored.monthly_tax_revenue - 1500.0).abs() < 0.01);
    assert!((restored.hotel_tax_rate - 0.15).abs() < 0.01);
}

#[test]
fn test_hotel_demand_saveable_skip_default() {
    use crate::Saveable;

    let state = HotelDemandState::default();
    assert!(
        state.save_to_bytes().is_none(),
        "Default state should not save (returns None)",
    );
}

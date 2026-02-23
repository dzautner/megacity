//! Integration tests for hotel demand and capacity system (SVC-019).

use crate::grid::ZoneType;
use crate::hotel_demand::HotelDemandState;
use crate::services::ServiceType;
use crate::stats::CityStats;
use crate::test_harness::TestCity;

/// Boost average happiness above the building downgrade threshold (30.0)
/// so buildings don't randomly lose levels during long tick runs.
fn prevent_downgrade(city: &mut TestCity) {
    let world = city.world_mut();
    let mut stats = world.resource_mut::<CityStats>();
    stats.average_happiness = 60.0;
}

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
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialHigh, 1);

    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert_eq!(state.hotel_count, 1);
    assert_eq!(state.total_capacity, 50); // Level 1 = 50 rooms
}

#[test]
fn test_hotel_demand_higher_level_more_capacity() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialHigh, 3);

    // Set happiness above 30 to prevent downgrade_buildings from firing.
    // Re-set between slow cycles because update_stats resets happiness to 0
    // when there are no citizens.
    for _ in 0..2 {
        {
            let world = city.world_mut();
            world.resource_mut::<CityStats>().average_happiness = 50.0;
        }
        city.tick_slow_cycle();
    }
    let state = city.resource::<HotelDemandState>();
    assert_eq!(
        state.total_capacity, 200,
        "Level 3 hotel should have 200 rooms capacity, got {}",
        state.total_capacity,
    );
}

#[test]
fn test_hotel_demand_multiple_buildings_sum_capacity() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialHigh, 1)
        .with_building(52, 52, ZoneType::CommercialHigh, 2);

    // Set happiness above 30 to prevent downgrade_buildings from firing
    // on the level-2 building. Re-set between slow cycles because
    // update_stats resets happiness to 0 when there are no citizens.
    for _ in 0..2 {
        {
            let world = city.world_mut();
            world.resource_mut::<CityStats>().average_happiness = 50.0;
        }
        city.tick_slow_cycle();
    }
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
fn test_hotel_demand_commercial_low_not_counted() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialLow, 2);

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
// Revenue via real tourism from services
// ====================================================================

#[test]
fn test_hotel_demand_revenue_with_tourist_attractions() {
    // City with hotels and many tourist attractions to generate real visitors
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialHigh, 3) // 200 rooms
        .with_service(60, 60, ServiceType::Stadium)
        .with_service(65, 65, ServiceType::Museum)
        .with_service(70, 70, ServiceType::Cathedral)
        .with_service(75, 75, ServiceType::CityHall)
        .with_service(80, 80, ServiceType::LargePark);

    // Run enough for tourism to update (needs day > 30).
    // Set happiness above 30 before each slow cycle to prevent
    // downgrade_buildings from reducing the hotel level.
    for _ in 0..5 {
        {
            let world = city.world_mut();
            world.resource_mut::<CityStats>().average_happiness = 50.0;
        }
        city.tick_slow_cycle();
    }

    let state = city.resource::<HotelDemandState>();
    assert_eq!(
        state.total_capacity, 200,
        "Level 3 hotel should have 200 rooms capacity, got {}",
        state.total_capacity
    );
    assert_eq!(state.hotel_count, 1);
    assert!(state.attractiveness_score > 0.0);
}

#[test]
fn test_hotel_demand_no_revenue_without_visitors() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialHigh, 1);

    prevent_downgrade(&mut city);
    // No services = no tourism = no visitors
    city.tick_slow_cycles(2);
    let state = city.resource::<HotelDemandState>();
    assert_eq!(
        state.monthly_tax_revenue, 0.0,
        "No visitors should mean no tax revenue",
    );
}

// ====================================================================
// Over-capacity and under-capacity via direct resource mutation
// ====================================================================

#[test]
fn test_hotel_demand_over_capacity_detection() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<HotelDemandState>();
        state.total_capacity = 100;
        state.rooms_demanded = 200;
    }
    let state = city.resource::<HotelDemandState>();
    assert!(state.is_over_capacity());
}

#[test]
fn test_hotel_demand_not_over_capacity_when_balanced() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<HotelDemandState>();
        state.total_capacity = 200;
        state.rooms_demanded = 100;
    }
    let state = city.resource::<HotelDemandState>();
    assert!(!state.is_over_capacity());
}

#[test]
fn test_hotel_demand_under_capacity_detection() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<HotelDemandState>();
        state.total_capacity = 500;
        state.occupancy_rate = 0.2;
    }
    let state = city.resource::<HotelDemandState>();
    assert!(state.is_under_capacity());
}

#[test]
fn test_hotel_demand_not_under_capacity_when_busy() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<HotelDemandState>();
        state.total_capacity = 500;
        state.occupancy_rate = 0.8;
    }
    let state = city.resource::<HotelDemandState>();
    assert!(!state.is_under_capacity());
}

#[test]
fn test_hotel_demand_effective_room_rate_premium() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<HotelDemandState>();
        state.occupancy_rate = 0.95;
    }
    let state = city.resource::<HotelDemandState>();
    let rate = state.effective_room_rate();
    // High occupancy should yield premium rate (> base rate of 120)
    assert!(rate > 120.0, "Expected premium rate at 95% occupancy, got {}", rate);
}

#[test]
fn test_hotel_demand_effective_room_rate_discount() {
    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<HotelDemandState>();
        state.occupancy_rate = 0.3;
    }
    let state = city.resource::<HotelDemandState>();
    let rate = state.effective_room_rate();
    // Low occupancy should yield discount rate (< base rate of 120)
    assert!(rate < 120.0, "Expected discount rate at 30% occupancy, got {}", rate);
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

    let bytes = state.save_to_bytes().expect("Should serialize non-default state");
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

//! Integration tests for POLL-016: Airport Air Pollution Sources.

use crate::pollution::PollutionGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::wind::WindState;

// ====================================================================
// Small airstrip emits area pollution on footprint
// ====================================================================

#[test]
fn test_airport_pollution_small_airstrip_emits() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::SmallAirstrip);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    // Small airstrip has 3x3 footprint starting at (50, 50)
    let at_origin = pollution.get(50, 50);
    let at_corner = pollution.get(52, 52);
    assert!(
        at_origin > 0,
        "Airport origin cell should have pollution, got {at_origin}"
    );
    assert!(
        at_corner > 0,
        "Airport footprint corner (52,52) should have pollution, got {at_corner}"
    );
}

// ====================================================================
// Regional airport emits on 4x3 footprint
// ====================================================================

#[test]
fn test_airport_pollution_regional_airport_emits() {
    let mut city = TestCity::new().with_service(60, 60, ServiceType::RegionalAirport);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    // Regional airport has 4x3 footprint: (60,60) to (63,62)
    let at_origin = pollution.get(60, 60);
    let at_far_x = pollution.get(63, 60);
    let at_far_y = pollution.get(60, 62);
    assert!(
        at_origin > 0,
        "Regional airport origin should have pollution, got {at_origin}"
    );
    assert!(
        at_far_x > 0,
        "Regional airport far-x cell should have pollution, got {at_far_x}"
    );
    assert!(
        at_far_y > 0,
        "Regional airport far-y cell should have pollution, got {at_far_y}"
    );
}

// ====================================================================
// International airport emits on 4x4 footprint
// ====================================================================

#[test]
fn test_airport_pollution_international_airport_emits() {
    let mut city =
        TestCity::new().with_service(70, 70, ServiceType::InternationalAirport);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    // International airport has 4x4 footprint: (70,70) to (73,73)
    let at_origin = pollution.get(70, 70);
    let at_corner = pollution.get(73, 73);
    assert!(
        at_origin > 0,
        "International airport origin should have pollution, got {at_origin}"
    );
    assert!(
        at_corner > 0,
        "International airport footprint corner should have pollution, got {at_corner}"
    );
}

// ====================================================================
// Non-airport service buildings do not get airport pollution
// ====================================================================

#[test]
fn test_airport_pollution_non_airport_no_extra_pollution() {
    // Place a park at (50,50) -- should have zero pollution
    let mut city = TestCity::new().with_service(50, 50, ServiceType::SmallPark);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_park = pollution.get(50, 50);
    assert_eq!(
        at_park, 0,
        "Park should not produce airport pollution, got {at_park}"
    );
}

// ====================================================================
// Cells outside airport footprint receive less pollution
// ====================================================================

#[test]
fn test_airport_pollution_drops_outside_footprint() {
    let mut city = TestCity::new().with_service(100, 100, ServiceType::SmallAirstrip);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    // Inside footprint: (100,100) to (102,102) for 3x3
    let inside = pollution.get(101, 101);
    // Well outside footprint
    let outside = pollution.get(120, 120);
    assert!(
        inside > outside,
        "Pollution inside footprint ({inside}) should be greater than \
         far outside ({outside})"
    );
}

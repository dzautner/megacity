//! Integration tests for POLL-015: Road Traffic Vehicle Emission Pollution Source.

use crate::grid::RoadType;
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;
use crate::traffic::TrafficGrid;
use crate::traffic_emissions::{base_emission_q, traffic_emission_q, traffic_scaling_factor};
use crate::wind::WindState;

// ====================================================================
// Unit-level: scaling factor boundaries
// ====================================================================

#[test]
fn test_traffic_emissions_scaling_factor_empty_is_lowest() {
    let empty = traffic_scaling_factor(0, RoadType::Local);
    let moderate = traffic_scaling_factor(5, RoadType::Local);
    assert!(
        moderate > empty,
        "Moderate ({moderate}) should be greater than empty ({empty})"
    );
}

#[test]
fn test_traffic_emissions_scaling_factor_over_capacity_is_highest() {
    let congested = traffic_scaling_factor(19, RoadType::Local);
    let over = traffic_scaling_factor(25, RoadType::Local);
    assert!(
        over > congested,
        "Over-capacity ({over}) should exceed congested ({congested})"
    );
}

// ====================================================================
// Unit-level: highway emits more than local at same utilization
// ====================================================================

#[test]
fn test_traffic_emissions_highway_emits_more_than_local_at_same_ratio() {
    // Both at 50% utilization: highway capacity=80, local capacity=20
    let highway_q = traffic_emission_q(RoadType::Highway, 40);
    let local_q = traffic_emission_q(RoadType::Local, 10);
    assert!(
        highway_q > local_q,
        "Highway ({highway_q}) should emit more than local ({local_q}) at same utilization"
    );
}

// ====================================================================
// Integration: road cells gain pollution from traffic emissions
// ====================================================================

#[test]
fn test_traffic_emissions_road_gains_pollution() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Highway);
    {
        let world = city.world_mut();
        // Set wind to zero to isolate traffic emissions from plume dispersion
        world.resource_mut::<WindState>().speed = 0.0;
        // Add some traffic to the road cells
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=60 {
            traffic.set(x, 50, 40); // moderate-to-congested on highway
        }
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    // Highway cells at x=55,y=50 should have some pollution
    let at_road = pollution.get(55, 50);
    assert!(
        at_road > 0,
        "Highway road cell with traffic should have pollution, got {at_road}"
    );
}

#[test]
fn test_traffic_emissions_empty_road_has_minimal_pollution() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        // No traffic set — density stays at 0
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_road = pollution.get(55, 50);
    // Empty local road emits Q=0.1, which rounds to 1 as u8
    // Combined with wind_pollution's road_emission_q, total should be small
    assert!(
        at_road <= 5,
        "Empty local road should have minimal pollution, got {at_road}"
    );
}

// ====================================================================
// Integration: busy highway corridor shows elevated pollution
// ====================================================================

#[test]
fn test_traffic_emissions_busy_highway_corridor_elevated() {
    let mut city = TestCity::new()
        .with_road(30, 50, 70, 50, RoadType::Highway);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        let mut traffic = world.resource_mut::<TrafficGrid>();
        // Set over-capacity traffic along the corridor
        for x in 30..=70 {
            traffic.set(x, 50, 100); // over capacity for highway (cap=80)
        }
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_corridor = pollution.get(50, 50);
    let away_from_corridor = pollution.get(50, 10);
    assert!(
        at_corridor > away_from_corridor,
        "Highway corridor ({at_corridor}) should have more pollution than \
         away from it ({away_from_corridor})"
    );
}

// ====================================================================
// Path roads produce no vehicle emissions
// ====================================================================

#[test]
fn test_traffic_emissions_path_no_emissions() {
    assert_eq!(
        base_emission_q(RoadType::Path),
        0.0,
        "Pedestrian paths should have zero base emission"
    );

    let q = traffic_emission_q(RoadType::Path, 5);
    assert_eq!(q, 0.0, "Path emission should be zero regardless of traffic");
}

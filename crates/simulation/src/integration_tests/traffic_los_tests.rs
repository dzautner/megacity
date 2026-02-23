//! Integration tests for the Traffic LOS grading system.

use crate::grid::RoadType;
use crate::test_harness::TestCity;
use crate::traffic::TrafficGrid;
use crate::traffic_los::{LosDistribution, LosGrade, TrafficLosGrid, TrafficLosState};

#[test]
fn test_traffic_los_resources_exist() {
    let city = TestCity::new();
    city.assert_resource_exists::<TrafficLosGrid>();
    city.assert_resource_exists::<TrafficLosState>();
    city.assert_resource_exists::<LosDistribution>();
}

#[test]
fn test_traffic_los_grid_defaults_to_all_a() {
    let city = TestCity::new();
    let los_grid = city.resource::<TrafficLosGrid>();
    // Spot check a few cells
    assert_eq!(los_grid.get(0, 0), LosGrade::A);
    assert_eq!(los_grid.get(10, 10), LosGrade::A);
    assert_eq!(los_grid.get(128, 128), LosGrade::A);
}

#[test]
fn test_traffic_los_state_empty_initially() {
    let city = TestCity::new();
    let los_state = city.resource::<TrafficLosState>();
    assert_eq!(los_state.segment_count(), 0);
}

#[test]
fn test_traffic_los_grades_road_cells() {
    let mut city = TestCity::new().with_road(50, 50, 80, 50, RoadType::Local);

    // Inject traffic density on the road cells
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        // Local capacity = 20. Set density=15 -> V/C=0.75 -> LOS C
        for x in 50..=80 {
            traffic.set(x, 50, 15);
        }
    }

    // Tick enough for the LOS system to run (runs every 10 ticks)
    city.tick(10);

    let los_grid = city.resource::<TrafficLosGrid>();
    // Road cells should have LOS C (V/C = 15/20 = 0.75)
    assert_eq!(
        los_grid.get(65, 50),
        LosGrade::C,
        "Road cell with V/C=0.75 should be LOS C"
    );
}

#[test]
fn test_traffic_los_segment_tracking() {
    let mut city = TestCity::new().with_road(50, 50, 70, 50, RoadType::Local);

    // Inject traffic density
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        // Local capacity = 20. Set density=18 -> V/C=0.9 -> LOS D
        for x in 50..=70 {
            traffic.set(x, 50, 18);
        }
    }

    city.tick(10);

    let los_state = city.resource::<TrafficLosState>();
    assert!(
        los_state.segment_count() > 0,
        "Should have at least one segment graded"
    );
}

#[test]
fn test_traffic_los_distribution_computed() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_road(50, 60, 70, 60, RoadType::Highway);

    // Inject different traffic levels
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        // Local (cap=20): density=18 -> V/C=0.9 -> LOS D
        for x in 50..=70 {
            traffic.set(x, 50, 18);
        }
        // Highway (cap=80): density=5 -> V/C=0.0625 -> LOS A
        for x in 50..=70 {
            traffic.set(x, 60, 5);
        }
    }

    city.tick(10);

    let distribution = city.resource::<LosDistribution>();
    assert!(distribution.total > 0, "Distribution should track segments");
}

#[test]
fn test_traffic_los_highway_vs_local_same_volume() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_road(50, 60, 70, 60, RoadType::Highway);

    // Same traffic volume on both road types
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=70 {
            traffic.set(x, 50, 15); // Local: 15/20=0.75 -> LOS C
            traffic.set(x, 60, 15); // Highway: 15/80=0.1875 -> LOS A
        }
    }

    city.tick(10);

    let los_grid = city.resource::<TrafficLosGrid>();
    let local_grade = los_grid.get(60, 50);
    let highway_grade = los_grid.get(60, 60);

    assert!(
        (highway_grade as u8) < (local_grade as u8),
        "Highway should have better LOS than local road at same volume. \
         Highway={:?}, Local={:?}",
        highway_grade,
        local_grade
    );
}

#[test]
fn test_traffic_los_clears_when_traffic_clears() {
    let mut city = TestCity::new().with_road(50, 50, 70, 50, RoadType::Local);

    // Add heavy traffic
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=70 {
            traffic.set(x, 50, 25); // V/C = 25/20 = 1.25 -> LOS F
        }
    }
    city.tick(10);

    let los_grid = city.resource::<TrafficLosGrid>();
    assert_eq!(
        los_grid.get(60, 50),
        LosGrade::F,
        "Heavy traffic should produce LOS F"
    );

    // Clear traffic
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=70 {
            traffic.set(x, 50, 0);
        }
    }
    city.tick(10);

    let los_grid = city.resource::<TrafficLosGrid>();
    assert_eq!(
        los_grid.get(60, 50),
        LosGrade::A,
        "Cleared traffic should return to LOS A"
    );
}

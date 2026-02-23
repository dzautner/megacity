//! Integration tests for the Traffic LOS grading system.

use crate::grid::RoadType;
use crate::road_segments::SegmentId;
use crate::test_harness::TestCity;
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
fn test_traffic_los_distribution_empty_initially() {
    let city = TestCity::new();
    let dist = city.resource::<LosDistribution>();
    assert_eq!(dist.total, 0);
    assert_eq!(dist.congested_percentage(), 0.0);
}

#[test]
fn test_traffic_los_grid_manual_set_and_get() {
    let mut city = TestCity::new().with_road(50, 50, 80, 50, RoadType::Local);

    {
        let world = city.world_mut();
        let mut los_grid = world.resource_mut::<TrafficLosGrid>();
        los_grid.set(60, 50, LosGrade::C);
        los_grid.set(70, 50, LosGrade::F);
    }

    let los_grid = city.resource::<TrafficLosGrid>();
    assert_eq!(los_grid.get(60, 50), LosGrade::C);
    assert_eq!(los_grid.get(70, 50), LosGrade::F);
    assert_eq!(los_grid.get(50, 50), LosGrade::A);
}

#[test]
fn test_traffic_los_state_manual_set_and_get() {
    let mut city = TestCity::new().with_road(50, 50, 70, 50, RoadType::Local);

    let seg_id = {
        let segments = city.road_segments();
        segments.segments[0].id
    };

    {
        let world = city.world_mut();
        let mut los_state = world.resource_mut::<TrafficLosState>();
        los_state.set(seg_id, LosGrade::D);
    }

    let los_state = city.resource::<TrafficLosState>();
    assert_eq!(los_state.get(seg_id), LosGrade::D);
    assert_eq!(los_state.segment_count(), 1);
}

#[test]
fn test_traffic_los_distribution_recompute() {
    // Test LosDistribution recompute using standalone resources
    let mut state = TrafficLosState::default();
    state.set(SegmentId(0), LosGrade::A);
    state.set(SegmentId(1), LosGrade::D);
    state.set(SegmentId(2), LosGrade::F);

    let mut dist = LosDistribution::default();
    dist.recompute(&state);

    assert_eq!(dist.total, 3);
    assert!((dist.percentage(LosGrade::A) - 33.33).abs() < 1.0);
    assert!((dist.congested_percentage() - 66.66).abs() < 1.0);
}

#[test]
fn test_traffic_los_system_runs_with_road_segments() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_road(50, 60, 70, 60, RoadType::Highway);

    // Run enough ticks for the LOS systems to fire
    city.tick(10);

    // With no commuting citizens, traffic is 0 so all LOS should be A
    let los_grid = city.resource::<TrafficLosGrid>();
    assert_eq!(los_grid.get(60, 50), LosGrade::A);

    // Segments should have been graded
    let los_state = city.resource::<TrafficLosState>();
    assert!(
        los_state.segment_count() > 0,
        "Segments should have been graded after ticking"
    );
}

#[test]
fn test_traffic_los_segment_grades_all_a_with_no_traffic() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_road(50, 60, 70, 60, RoadType::Highway);

    city.tick(10);

    let los_state = city.resource::<TrafficLosState>();
    let segments = city.road_segments();

    for segment in &segments.segments {
        assert_eq!(
            los_state.get(segment.id),
            LosGrade::A,
            "Segment {:?} should be LOS A with no traffic",
            segment.id
        );
    }
}

#[test]
fn test_traffic_los_distribution_after_system_run() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_road(50, 60, 70, 60, RoadType::Highway);

    city.tick(10);

    let dist = city.resource::<LosDistribution>();
    assert!(
        dist.total > 0,
        "Distribution should track segments after system runs"
    );
    // With no traffic, all segments should be A
    assert!((dist.percentage(LosGrade::A) - 100.0).abs() < 0.01);
    assert_eq!(dist.congested_percentage(), 0.0);
}

#[test]
fn test_traffic_los_vc_ratio_produces_correct_grades() {
    // Test the pure LOS grading logic against issue #445 specified values
    assert_eq!(LosGrade::from_vc_ratio(0.3), LosGrade::A);
    assert_eq!(LosGrade::from_vc_ratio(0.9), LosGrade::D);
    assert_eq!(LosGrade::from_vc_ratio(1.2), LosGrade::F);
}

#[test]
fn test_traffic_los_highway_capacity_advantage() {
    let local_cap = RoadType::Local.capacity() as f32;
    let highway_cap = RoadType::Highway.capacity() as f32;
    let volume = 15.0;

    let local_grade = LosGrade::from_vc_ratio(volume / local_cap);
    let highway_grade = LosGrade::from_vc_ratio(volume / highway_cap);

    assert!(
        (highway_grade as u8) < (local_grade as u8),
        "Highway should have better LOS than local road at same volume. \
         Highway={:?} (V/C={:.2}), Local={:?} (V/C={:.2})",
        highway_grade,
        volume / highway_cap,
        local_grade,
        volume / local_cap
    );
}

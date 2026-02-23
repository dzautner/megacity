//! Unit tests for the Traffic LOS grading system.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::RoadType;
use crate::Saveable;

use super::grades::LosGrade;
use super::grid::TrafficLosGrid;
use super::segment_los::{LosDistribution, TrafficLosState};

// ====================================================================
// LosGrade classification tests
// ====================================================================

#[test]
fn test_los_grade_from_vc_ratio() {
    // LOS A: v/c < 0.35
    assert_eq!(LosGrade::from_vc_ratio(0.0), LosGrade::A);
    assert_eq!(LosGrade::from_vc_ratio(0.20), LosGrade::A);
    assert_eq!(LosGrade::from_vc_ratio(0.34), LosGrade::A);
    // LOS B: v/c < 0.55
    assert_eq!(LosGrade::from_vc_ratio(0.35), LosGrade::B);
    assert_eq!(LosGrade::from_vc_ratio(0.54), LosGrade::B);
    // LOS C: v/c < 0.77
    assert_eq!(LosGrade::from_vc_ratio(0.55), LosGrade::C);
    assert_eq!(LosGrade::from_vc_ratio(0.76), LosGrade::C);
    // LOS D: v/c < 0.93
    assert_eq!(LosGrade::from_vc_ratio(0.77), LosGrade::D);
    assert_eq!(LosGrade::from_vc_ratio(0.89), LosGrade::D);
    assert_eq!(LosGrade::from_vc_ratio(0.92), LosGrade::D);
    // LOS E: v/c < 1.00
    assert_eq!(LosGrade::from_vc_ratio(0.93), LosGrade::E);
    assert_eq!(LosGrade::from_vc_ratio(0.99), LosGrade::E);
    // LOS F: v/c >= 1.00
    assert_eq!(LosGrade::from_vc_ratio(1.00), LosGrade::F);
    assert_eq!(LosGrade::from_vc_ratio(2.50), LosGrade::F);
}

#[test]
fn test_issue_specified_values() {
    // From issue #445: V/C=0.3 -> LOS A, V/C=0.9 -> LOS D, V/C=1.2 -> LOS F
    assert_eq!(LosGrade::from_vc_ratio(0.3), LosGrade::A);
    assert_eq!(LosGrade::from_vc_ratio(0.9), LosGrade::D);
    assert_eq!(LosGrade::from_vc_ratio(1.2), LosGrade::F);
}

#[test]
fn test_boundary_thresholds_match_spec() {
    // Verify exact boundary values from the spec
    assert_eq!(LosGrade::from_vc_ratio(0.349), LosGrade::A);
    assert_eq!(LosGrade::from_vc_ratio(0.351), LosGrade::B);
    assert_eq!(LosGrade::from_vc_ratio(0.549), LosGrade::B);
    assert_eq!(LosGrade::from_vc_ratio(0.551), LosGrade::C);
    assert_eq!(LosGrade::from_vc_ratio(0.769), LosGrade::C);
    assert_eq!(LosGrade::from_vc_ratio(0.771), LosGrade::D);
    assert_eq!(LosGrade::from_vc_ratio(0.929), LosGrade::D);
    assert_eq!(LosGrade::from_vc_ratio(0.931), LosGrade::E);
    assert_eq!(LosGrade::from_vc_ratio(0.999), LosGrade::E);
    assert_eq!(LosGrade::from_vc_ratio(1.001), LosGrade::F);
}

// ====================================================================
// LosGrade utility tests
// ====================================================================

#[test]
fn test_los_grade_as_t() {
    assert!((LosGrade::A.as_t() - 0.0).abs() < f32::EPSILON);
    assert!((LosGrade::B.as_t() - 0.2).abs() < f32::EPSILON);
    assert!((LosGrade::C.as_t() - 0.4).abs() < f32::EPSILON);
    assert!((LosGrade::D.as_t() - 0.6).abs() < f32::EPSILON);
    assert!((LosGrade::E.as_t() - 0.8).abs() < f32::EPSILON);
    assert!((LosGrade::F.as_t() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_los_label_non_empty() {
    let grades = [
        LosGrade::A,
        LosGrade::B,
        LosGrade::C,
        LosGrade::D,
        LosGrade::E,
        LosGrade::F,
    ];
    for grade in &grades {
        assert!(!grade.label().is_empty());
    }
}

#[test]
fn test_los_color_has_alpha() {
    let grades = [
        LosGrade::A,
        LosGrade::B,
        LosGrade::C,
        LosGrade::D,
        LosGrade::E,
        LosGrade::F,
    ];
    for grade in &grades {
        let color = grade.color();
        assert!(color[3] > 0.0, "Alpha should be > 0 for {grade:?}");
    }
}

#[test]
fn test_los_letter() {
    assert_eq!(LosGrade::A.letter(), 'A');
    assert_eq!(LosGrade::B.letter(), 'B');
    assert_eq!(LosGrade::C.letter(), 'C');
    assert_eq!(LosGrade::D.letter(), 'D');
    assert_eq!(LosGrade::E.letter(), 'E');
    assert_eq!(LosGrade::F.letter(), 'F');
}

// ====================================================================
// TrafficLosGrid tests
// ====================================================================

#[test]
fn test_los_grid_default() {
    let grid = TrafficLosGrid::default();
    assert_eq!(grid.grades.len(), GRID_WIDTH * GRID_HEIGHT);
    for y in 0..3 {
        for x in 0..3 {
            assert_eq!(grid.get(x, y), LosGrade::A);
        }
    }
}

#[test]
fn test_los_grid_set_get() {
    let mut grid = TrafficLosGrid::default();
    grid.set(5, 5, LosGrade::C);
    assert_eq!(grid.get(5, 5), LosGrade::C);
    grid.set(5, 5, LosGrade::F);
    assert_eq!(grid.get(5, 5), LosGrade::F);
}

#[test]
fn test_los_grid_get_t() {
    let mut grid = TrafficLosGrid::default();
    grid.set(0, 0, LosGrade::A);
    assert!((grid.get_t(0, 0) - 0.0).abs() < f32::EPSILON);
    grid.set(0, 0, LosGrade::F);
    assert!((grid.get_t(0, 0) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_saveable_skip_default() {
    let grid = TrafficLosGrid::default();
    assert!(
        grid.save_to_bytes().is_none(),
        "Default state should skip saving"
    );
}

#[test]
fn test_saveable_roundtrip() {
    let mut grid = TrafficLosGrid::default();
    grid.set(10, 10, LosGrade::D);
    grid.set(20, 20, LosGrade::F);

    let bytes = grid.save_to_bytes().expect("Non-default should save");
    let restored = TrafficLosGrid::load_from_bytes(&bytes);

    assert_eq!(restored.get(10, 10), LosGrade::D);
    assert_eq!(restored.get(20, 20), LosGrade::F);
    assert_eq!(restored.get(0, 0), LosGrade::A);
}

// ====================================================================
// TrafficLosState tests
// ====================================================================

#[test]
fn test_los_state_default_grade() {
    let state = TrafficLosState::default();
    assert_eq!(
        state.get(crate::road_segments::SegmentId(42)),
        LosGrade::A,
        "Unknown segment should default to LOS A"
    );
}

#[test]
fn test_los_state_set_get() {
    let mut state = TrafficLosState::default();
    let id = crate::road_segments::SegmentId(7);
    state.set(id, LosGrade::D);
    assert_eq!(state.get(id), LosGrade::D);
    state.set(id, LosGrade::F);
    assert_eq!(state.get(id), LosGrade::F);
}

#[test]
fn test_los_state_remove() {
    let mut state = TrafficLosState::default();
    let id = crate::road_segments::SegmentId(3);
    state.set(id, LosGrade::C);
    assert_eq!(state.segment_count(), 1);
    state.remove(id);
    assert_eq!(state.segment_count(), 0);
    assert_eq!(state.get(id), LosGrade::A);
}

#[test]
fn test_los_state_saveable_roundtrip() {
    let mut state = TrafficLosState::default();
    state.set(crate::road_segments::SegmentId(1), LosGrade::B);
    state.set(crate::road_segments::SegmentId(5), LosGrade::E);

    let bytes = state.save_to_bytes().expect("Non-empty should save");
    let restored = TrafficLosState::load_from_bytes(&bytes);

    assert_eq!(
        restored.get(crate::road_segments::SegmentId(1)),
        LosGrade::B
    );
    assert_eq!(
        restored.get(crate::road_segments::SegmentId(5)),
        LosGrade::E
    );
}

// ====================================================================
// LosDistribution tests
// ====================================================================

#[test]
fn test_distribution_empty() {
    let dist = LosDistribution::default();
    assert_eq!(dist.total, 0);
    assert_eq!(dist.percentage(LosGrade::A), 0.0);
    assert_eq!(dist.weighted_average(), 0.0);
    assert_eq!(dist.congested_percentage(), 0.0);
}

#[test]
fn test_distribution_recompute() {
    let mut state = TrafficLosState::default();
    state.set(crate::road_segments::SegmentId(0), LosGrade::A);
    state.set(crate::road_segments::SegmentId(1), LosGrade::A);
    state.set(crate::road_segments::SegmentId(2), LosGrade::C);
    state.set(crate::road_segments::SegmentId(3), LosGrade::F);

    let mut dist = LosDistribution::default();
    dist.recompute(&state);

    assert_eq!(dist.total, 4);
    assert_eq!(dist.counts[0], 2); // A
    assert_eq!(dist.counts[2], 1); // C
    assert_eq!(dist.counts[5], 1); // F
    assert!((dist.percentage(LosGrade::A) - 50.0).abs() < 0.01);
    assert!((dist.congested_percentage() - 25.0).abs() < 0.01);
}

#[test]
fn test_distribution_weighted_average() {
    let mut state = TrafficLosState::default();
    // All grade F (value 5)
    for i in 0..10 {
        state.set(crate::road_segments::SegmentId(i), LosGrade::F);
    }
    let mut dist = LosDistribution::default();
    dist.recompute(&state);
    assert!((dist.weighted_average() - 5.0).abs() < 0.01);

    // All grade A (value 0)
    let mut state2 = TrafficLosState::default();
    for i in 0..10 {
        state2.set(crate::road_segments::SegmentId(i), LosGrade::A);
    }
    dist.recompute(&state2);
    assert!((dist.weighted_average() - 0.0).abs() < 0.01);
}

// ====================================================================
// Road capacity ordering test
// ====================================================================

#[test]
fn test_road_capacity_ordering() {
    assert!(RoadType::Path.capacity() < RoadType::Local.capacity());
    assert!(RoadType::Local.capacity() < RoadType::Avenue.capacity());
    assert!(RoadType::Avenue.capacity() < RoadType::Boulevard.capacity());
    assert!(RoadType::Boulevard.capacity() < RoadType::Highway.capacity());
}

#[test]
fn test_highway_needs_more_traffic_for_congestion() {
    // Highway: 15/80=0.1875 -> A, Local: 15/20=0.75 -> C
    let highway_vc = 15.0 / RoadType::Highway.capacity() as f32;
    let local_vc = 15.0 / RoadType::Local.capacity() as f32;

    let highway_grade = LosGrade::from_vc_ratio(highway_vc);
    let local_grade = LosGrade::from_vc_ratio(local_vc);

    assert!(
        (highway_grade as u8) < (local_grade as u8),
        "Highway should have better LOS than local road with same traffic"
    );
}

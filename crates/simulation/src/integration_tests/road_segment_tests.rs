//! TEST-038: Integration tests for road segment (Bezier) operations.
//!
//! Tests Bezier segment rasterization, intersection detection between
//! crossing segments, segment splitting, and arc-length calculation.

use bevy::math::Vec2;

use crate::grid::{CellType, RoadType};
use crate::road_segments::{RoadSegment, RoadSegmentStore, SegmentId, SegmentNodeId};
use crate::roads::RoadNetwork;
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_segment(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> RoadSegment {
    let mut seg = RoadSegment {
        id: SegmentId(0),
        start_node: SegmentNodeId(0),
        end_node: SegmentNodeId(1),
        p0,
        p1,
        p2,
        p3,
        road_type: RoadType::Local,
        arc_length: 0.0,
        rasterized_cells: Vec::new(),
    };
    seg.arc_length = seg.compute_arc_length();
    seg
}

fn make_linear_segment(from: Vec2, to: Vec2) -> RoadSegment {
    let p1 = from + (to - from) / 3.0;
    let p2 = from + (to - from) * 2.0 / 3.0;
    make_segment(from, p1, p2, to)
}

/// De Casteljau split at parameter t.
fn de_casteljau_split(
    seg: &RoadSegment,
    t: f32,
) -> ((Vec2, Vec2, Vec2, Vec2), (Vec2, Vec2, Vec2, Vec2)) {
    let p01 = seg.p0.lerp(seg.p1, t);
    let p12 = seg.p1.lerp(seg.p2, t);
    let p23 = seg.p2.lerp(seg.p3, t);
    let p012 = p01.lerp(p12, t);
    let p123 = p12.lerp(p23, t);
    let p0123 = p012.lerp(p123, t);
    ((seg.p0, p01, p012, p0123), (p0123, p123, p23, seg.p3))
}

// ===================================================================
// 1. Linear Bezier rasterization
// ===================================================================

/// Straight horizontal segment rasterizes along x-axis near y=128.
#[test]
fn test_road_segment_linear_horizontal_rasterizes_to_expected_cells() {
    let city = TestCity::new().with_road(100, 128, 110, 128, RoadType::Local);
    let seg = &city.road_segments().segments[0];

    assert!(!seg.rasterized_cells.is_empty());
    for &(_gx, gy) in &seg.rasterized_cells {
        assert!(gy >= 127 && gy <= 129, "cell y={} should be near 128", gy);
    }
    let min_x = seg.rasterized_cells.iter().map(|c| c.0).min().unwrap();
    let max_x = seg.rasterized_cells.iter().map(|c| c.0).max().unwrap();
    assert!(min_x <= 101, "min_x={} should start near 100", min_x);
    assert!(max_x >= 109, "max_x={} should reach near 110", max_x);
}

/// Straight vertical segment rasterizes along y-axis near x=128.
#[test]
fn test_road_segment_linear_vertical_rasterizes_correctly() {
    let city = TestCity::new().with_road(128, 100, 128, 110, RoadType::Avenue);
    let seg = &city.road_segments().segments[0];

    assert!(!seg.rasterized_cells.is_empty());
    for &(gx, _gy) in &seg.rasterized_cells {
        assert!(gx >= 127 && gx <= 129, "cell x={} should be near 128", gx);
    }
    let min_y = seg.rasterized_cells.iter().map(|c| c.1).min().unwrap();
    let max_y = seg.rasterized_cells.iter().map(|c| c.1).max().unwrap();
    assert!(min_y <= 101, "min_y={} should start near 100", min_y);
    assert!(max_y >= 109, "max_y={} should reach near 110", max_y);
}

/// All rasterized cells of a linear segment are marked as roads in the grid.
#[test]
fn test_road_segment_linear_cells_are_roads_in_grid() {
    let city = TestCity::new().with_road(120, 128, 130, 128, RoadType::Local);
    let grid = city.grid();
    let seg = &city.road_segments().segments[0];

    for &(gx, gy) in &seg.rasterized_cells {
        assert_eq!(
            grid.get(gx, gy).cell_type,
            CellType::Road,
            "cell ({}, {}) should be Road",
            gx,
            gy,
        );
    }
}

// ===================================================================
// 2. Curved Bezier rasterization bounds
// ===================================================================

/// Curved segment stays within bounding box of control points (with margin).
#[test]
fn test_road_segment_curved_rasterizes_within_bounds() {
    let city = TestCity::new().with_curved_road(120, 128, 125, 140, 130, 128, RoadType::Local);
    let seg = &city.road_segments().segments[0];

    assert!(!seg.rasterized_cells.is_empty());
    let margin: usize = 3;
    for &(gx, gy) in &seg.rasterized_cells {
        assert!(
            gx >= 120 - margin && gx <= 130 + margin && gy >= 128 - margin && gy <= 140 + margin,
            "cell ({}, {}) outside bounds",
            gx,
            gy,
        );
    }
}

/// Curved segment has cells that deviate from the straight line.
#[test]
fn test_road_segment_curved_cells_deviate_from_straight_line() {
    let city = TestCity::new().with_curved_road(120, 128, 125, 145, 130, 128, RoadType::Local);
    let seg = &city.road_segments().segments[0];
    let max_y = seg.rasterized_cells.iter().map(|c| c.1).max().unwrap();
    assert!(max_y > 130, "curved cells should reach above y=130, max_y={}", max_y);
}

// ===================================================================
// 3. Intersection detection between crossing segments
// ===================================================================

/// Two crossing segments (horizontal + vertical) share rasterized cells.
#[test]
fn test_road_segment_crossing_segments_share_rasterized_cells() {
    let city = TestCity::new()
        .with_road(120, 128, 136, 128, RoadType::Local)
        .with_road(128, 120, 128, 136, RoadType::Local);

    let segments = city.road_segments();
    assert_eq!(segments.segments.len(), 2);

    let cells_a: std::collections::HashSet<_> =
        segments.segments[0].rasterized_cells.iter().copied().collect();
    let cells_b: std::collections::HashSet<_> =
        segments.segments[1].rasterized_cells.iter().copied().collect();
    let shared: Vec<_> = cells_a.intersection(&cells_b).collect();

    assert!(!shared.is_empty(), "crossing segments should share cells");
    for &&(gx, gy) in &shared {
        assert!(
            gx >= 126 && gx <= 130 && gy >= 126 && gy <= 130,
            "intersection cell ({}, {}) should be near (128, 128)",
            gx,
            gy,
        );
    }
}

/// Two parallel segments do NOT share rasterized cells.
#[test]
fn test_road_segment_parallel_segments_no_shared_cells() {
    let city = TestCity::new()
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_road(100, 110, 120, 110, RoadType::Local);

    let segments = city.road_segments();
    let cells_a: std::collections::HashSet<_> =
        segments.segments[0].rasterized_cells.iter().copied().collect();
    let cells_b: std::collections::HashSet<_> =
        segments.segments[1].rasterized_cells.iter().copied().collect();
    let shared_count = cells_a.intersection(&cells_b).count();
    assert_eq!(shared_count, 0, "parallel segments should not share cells");
}

/// Two crossing curved segments share rasterized cells.
#[test]
fn test_road_segment_curved_crossing_detected() {
    let city = TestCity::new()
        .with_curved_road(115, 128, 125, 140, 135, 128, RoadType::Local)
        .with_curved_road(125, 118, 118, 128, 125, 138, RoadType::Local);

    let segments = city.road_segments();
    let cells_a: std::collections::HashSet<_> =
        segments.segments[0].rasterized_cells.iter().copied().collect();
    let cells_b: std::collections::HashSet<_> =
        segments.segments[1].rasterized_cells.iter().copied().collect();
    let shared_count = cells_a.intersection(&cells_b).count();
    assert!(shared_count > 0, "crossing curved segments should share cells");
}

// ===================================================================
// 4. Segment splitting produces valid sub-segments
// ===================================================================

/// Split at t=0.5 produces sub-segments whose combined length matches original.
#[test]
fn test_road_segment_split_preserves_arc_length() {
    let seg = make_segment(
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 200.0),
        Vec2::new(200.0, -100.0),
        Vec2::new(300.0, 0.0),
    );
    let original_length = seg.compute_arc_length();
    let (left, right) = de_casteljau_split(&seg, 0.5);
    let combined = make_segment(left.0, left.1, left.2, left.3).compute_arc_length()
        + make_segment(right.0, right.1, right.2, right.3).compute_arc_length();
    let error = (combined - original_length).abs();
    assert!(error < 2.0, "combined={}, original={}, error={}", combined, original_length, error);
}

/// Split sub-segments have endpoint continuity: left.p3 == right.p0 == evaluate(t).
#[test]
fn test_road_segment_split_endpoint_continuity() {
    let seg = make_segment(
        Vec2::new(0.0, 0.0),
        Vec2::new(80.0, 200.0),
        Vec2::new(220.0, -100.0),
        Vec2::new(300.0, 50.0),
    );
    for &t in &[0.25_f32, 0.5, 0.75] {
        let split_point = seg.evaluate(t);
        let (left, right) = de_casteljau_split(&seg, t);
        assert!((left.3 - split_point).length() < 0.1, "left.p3 != evaluate({})", t);
        assert!((right.0 - split_point).length() < 0.1, "right.p0 != evaluate({})", t);
        assert!((left.3 - right.0).length() < 0.01, "left.p3 != right.p0 at t={}", t);
    }
}

/// Sub-segments trace the original curve when re-parameterized.
#[test]
fn test_road_segment_split_subsegments_trace_original_curve() {
    let seg = make_segment(
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 300.0),
        Vec2::new(200.0, -200.0),
        Vec2::new(300.0, 100.0),
    );
    let t_split = 0.4_f32;
    let (left, right) = de_casteljau_split(&seg, t_split);
    let left_seg = make_segment(left.0, left.1, left.2, left.3);
    let right_seg = make_segment(right.0, right.1, right.2, right.3);

    for i in 0..=10 {
        let u = i as f32 / 10.0;
        let left_pt = left_seg.evaluate(u);
        let orig_pt = seg.evaluate(u * t_split);
        assert!(
            (left_pt - orig_pt).length() < 1.0,
            "left at u={}: {:?} vs original at t={}: {:?}",
            u, left_pt, u * t_split, orig_pt,
        );

        let right_pt = right_seg.evaluate(u);
        let orig_t = t_split + u * (1.0 - t_split);
        let orig_pt2 = seg.evaluate(orig_t);
        assert!(
            (right_pt - orig_pt2).length() < 1.0,
            "right at u={}: {:?} vs original at t={}: {:?}",
            u, right_pt, orig_t, orig_pt2,
        );
    }
}

/// Split at boundary (t=0) leaves entire curve in right half.
#[test]
fn test_road_segment_split_at_zero_preserves_curve() {
    let seg = make_segment(
        Vec2::new(0.0, 0.0),
        Vec2::new(50.0, 100.0),
        Vec2::new(150.0, 100.0),
        Vec2::new(200.0, 0.0),
    );
    let (_left, right) = de_casteljau_split(&seg, 0.0);
    let right_seg = make_segment(right.0, right.1, right.2, right.3);
    assert!((right_seg.p0 - seg.p0).length() < 0.01);
    assert!((right_seg.p3 - seg.p3).length() < 0.01);
    assert!((right_seg.compute_arc_length() - seg.compute_arc_length()).abs() < 1.0);
}

// ===================================================================
// 5. Segment length calculation
// ===================================================================

/// Straight-line Bezier arc length matches Euclidean distance (3-4-5 triangle).
#[test]
fn test_road_segment_length_straight_equals_euclidean() {
    let seg = make_linear_segment(Vec2::new(0.0, 0.0), Vec2::new(400.0, 300.0));
    let computed = seg.compute_arc_length();
    assert!((computed - 500.0).abs() < 1.0, "length={}, expected ~500", computed);
}

/// Curved Bezier arc length exceeds straight-line distance.
#[test]
fn test_road_segment_length_curved_exceeds_straight() {
    let seg = make_segment(
        Vec2::new(0.0, 0.0),
        Vec2::new(50.0, 200.0),
        Vec2::new(250.0, -100.0),
        Vec2::new(300.0, 0.0),
    );
    let straight = (seg.p3 - seg.p0).length();
    assert!(seg.compute_arc_length() > straight, "arc should exceed chord");
}

/// Degenerate (zero-length) segment has arc length near zero.
#[test]
fn test_road_segment_length_degenerate_zero() {
    let pt = Vec2::new(100.0, 100.0);
    let len = make_segment(pt, pt, pt, pt).compute_arc_length();
    assert!(len < 0.01, "degenerate length should be ~0, got {}", len);
}

/// Evaluate at t=0 returns p0, at t=1 returns p3.
#[test]
fn test_road_segment_evaluate_at_endpoints() {
    let seg = make_segment(
        Vec2::new(10.0, 20.0),
        Vec2::new(50.0, 80.0),
        Vec2::new(150.0, 90.0),
        Vec2::new(200.0, 30.0),
    );
    assert!((seg.evaluate(0.0) - seg.p0).length() < 0.01);
    assert!((seg.evaluate(1.0) - seg.p3).length() < 0.01);
}

/// Tangent at t=0 aligns with (p1 - p0).
#[test]
fn test_road_segment_tangent_direction_at_start() {
    let seg = make_segment(
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 50.0),
        Vec2::new(200.0, 50.0),
        Vec2::new(300.0, 0.0),
    );
    let dot = (seg.p1 - seg.p0).normalize().dot(seg.tangent(0.0).normalize());
    assert!(dot > 0.99, "tangent dot product should be ~1, got {}", dot);
}

/// sample_uniform returns requested count of points.
#[test]
fn test_road_segment_sample_uniform_count() {
    let seg = make_linear_segment(Vec2::new(0.0, 0.0), Vec2::new(300.0, 0.0));
    for n in [0, 1, 5, 20] {
        assert_eq!(seg.sample_uniform(n).len(), n);
    }
}

// ===================================================================
// Integration: store operations via TestCity
// ===================================================================

/// Adding then removing a segment clears all road cells.
#[test]
fn test_road_segment_add_then_remove_clears_grid() {
    use crate::grid::WorldGrid;

    let mut city = TestCity::new().with_road(120, 128, 130, 128, RoadType::Local);
    assert!(city.road_cell_count() > 0);

    let world = city.world_mut();
    world.resource_scope(|world, mut segments: bevy::prelude::Mut<RoadSegmentStore>| {
        let seg_id = segments.segments[0].id;
        world.resource_scope(|world, mut grid: bevy::prelude::Mut<WorldGrid>| {
            world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                segments.remove_segment(seg_id, &mut grid, &mut roads);
            });
        });
    });
    assert_eq!(city.road_cell_count(), 0, "road cells should be cleared");
}

/// Connected segments reuse the same node at their shared endpoint.
#[test]
fn test_road_segment_node_snapping_reuses_nodes() {
    let city = TestCity::new()
        .with_road(100, 128, 120, 128, RoadType::Local)
        .with_road(120, 128, 140, 128, RoadType::Local);

    let segments = city.road_segments();
    assert_eq!(segments.segments.len(), 2);

    let seg0_end = segments.segments[0].end_node;
    let seg1_start = segments.segments[1].start_node;
    assert_eq!(seg0_end, seg1_start, "should share node at junction");

    let node = segments.get_node(seg0_end).unwrap();
    assert!(node.connected_segments.len() >= 2, "shared node needs 2+ connections");
}

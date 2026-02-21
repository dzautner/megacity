use crate::grid::{CellType, RoadType, WorldGrid};
use crate::test_harness::TestCity;

// ====================================================================
// Curve Road Drawing (UX-019) tests
// ====================================================================

#[test]
fn test_curved_road_creates_segment_with_nonlinear_control_points() {
    let city = TestCity::new().with_budget(100_000.0).with_curved_road(
        120,
        128,
        125,
        135,
        130,
        128,
        RoadType::Local,
    );

    let segments = city.road_segments();
    assert_eq!(segments.segments.len(), 1, "should have exactly 1 segment");

    let seg = &segments.segments[0];
    let straight_p1 = seg.p0 + (seg.p3 - seg.p0) / 3.0;
    let straight_p2 = seg.p0 + (seg.p3 - seg.p0) * 2.0 / 3.0;

    let p1_diff = (seg.p1 - straight_p1).length();
    let p2_diff = (seg.p2 - straight_p2).length();
    assert!(
        p1_diff > 1.0 || p2_diff > 1.0,
        "curved segment should have non-trivial control points, p1_diff={}, p2_diff={}",
        p1_diff,
        p2_diff
    );
}

#[test]
fn test_curved_road_rasterizes_cells() {
    let city = TestCity::new().with_budget(100_000.0).with_curved_road(
        120,
        128,
        125,
        135,
        130,
        128,
        RoadType::Local,
    );

    let segments = city.road_segments();
    let seg = &segments.segments[0];
    assert!(
        !seg.rasterized_cells.is_empty(),
        "curved segment should rasterize to grid cells"
    );

    let grid = city.grid();
    let road_cells = seg
        .rasterized_cells
        .iter()
        .filter(|&&(gx, gy)| grid.get(gx, gy).cell_type == CellType::Road)
        .count();
    assert!(road_cells > 0, "rasterized cells should include road cells");
}

#[test]
fn test_curved_road_has_longer_arc_than_straight_distance() {
    let city = TestCity::new().with_budget(100_000.0).with_curved_road(
        120,
        128,
        125,
        140,
        130,
        128,
        RoadType::Avenue,
    );

    let segments = city.road_segments();
    let seg = &segments.segments[0];
    let straight_dist = (seg.p3 - seg.p0).length();

    assert!(
        seg.arc_length > straight_dist,
        "curved road arc length ({}) should exceed straight distance ({})",
        seg.arc_length,
        straight_dist
    );
}

#[test]
fn test_curved_road_endpoints_match_requested_positions() {
    use crate::config::CELL_SIZE;

    let city = TestCity::new().with_budget(100_000.0).with_curved_road(
        120,
        128,
        125,
        135,
        130,
        128,
        RoadType::Local,
    );

    let segments = city.road_segments();
    let seg = &segments.segments[0];

    let at_start = seg.evaluate(0.0);
    let at_end = seg.evaluate(1.0);
    assert!(
        (at_start - seg.p0).length() < 0.01,
        "curve start should match p0"
    );
    assert!(
        (at_end - seg.p3).length() < 0.01,
        "curve end should match p3"
    );

    let (wx0, wy0) = WorldGrid::grid_to_world(120, 128);
    let (wx1, wy1) = WorldGrid::grid_to_world(130, 128);
    assert!(
        (seg.p0 - bevy::math::Vec2::new(wx0, wy0)).length() < CELL_SIZE,
        "start should be near grid (120, 128)"
    );
    assert!(
        (seg.p3 - bevy::math::Vec2::new(wx1, wy1)).length() < CELL_SIZE,
        "end should be near grid (130, 128)"
    );
}

#[test]
fn test_curved_road_creates_nodes() {
    let city = TestCity::new().with_budget(100_000.0).with_curved_road(
        120,
        128,
        125,
        135,
        130,
        128,
        RoadType::Local,
    );

    let segments = city.road_segments();
    assert!(
        segments.nodes.len() >= 2,
        "curved road should create at least 2 nodes, got {}",
        segments.nodes.len()
    );
}

#[test]
fn test_curved_road_different_types() {
    for road_type in [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
    ] {
        let city = TestCity::new()
            .with_budget(100_000.0)
            .with_curved_road(120, 128, 125, 135, 130, 128, road_type);

        let segments = city.road_segments();
        assert_eq!(
            segments.segments[0].road_type, road_type,
            "segment road type should match requested type"
        );
    }
}

#[test]
fn test_quadratic_to_cubic_conversion_preserves_midpoint() {
    use crate::curve_road_drawing::quadratic_to_cubic;
    use bevy::math::Vec2;

    let p0 = Vec2::new(0.0, 0.0);
    let control = Vec2::new(150.0, 200.0);
    let p3 = Vec2::new(300.0, 0.0);

    let (p1, p2) = quadratic_to_cubic(p0, control, p3);

    let quad_mid: Vec2 = 0.25 * p0 + 0.5 * control + 0.25 * p3;
    let cubic_mid: Vec2 = 0.125 * p0 + 0.375 * p1 + 0.375 * p2 + 0.125 * p3;

    assert!(
        (quad_mid - cubic_mid).length() < 0.1,
        "cubic midpoint ({:?}) should match quadratic midpoint ({:?})",
        cubic_mid,
        quad_mid,
    );
}

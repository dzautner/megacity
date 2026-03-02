//! Integration tests for freehand Bezier curve fitting.

use bevy::math::Vec2;

use crate::freehand_road::{bezier_arc_length, bezier_point, fit_catmull_rom_beziers};

#[test]
fn test_fit_empty_points_returns_empty() {
    let result = fit_catmull_rom_beziers(&[]);
    assert!(result.is_empty());
}

#[test]
fn test_fit_single_point_returns_empty() {
    let result = fit_catmull_rom_beziers(&[Vec2::new(100.0, 200.0)]);
    assert!(result.is_empty());
}

#[test]
fn test_fit_two_points_returns_straight_segment() {
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(300.0, 0.0);
    let result = fit_catmull_rom_beziers(&[p0, p3]);

    assert_eq!(result.len(), 1, "Two points should produce exactly 1 segment");

    let (rp0, c1, c2, rp3) = result[0];
    assert!((rp0 - p0).length() < 0.01, "Start point should match");
    assert!((rp3 - p3).length() < 0.01, "End point should match");

    // Control points should be at 1/3 and 2/3 along the line
    let expected_c1 = p0 + (p3 - p0) / 3.0;
    let expected_c2 = p0 + (p3 - p0) * (2.0 / 3.0);
    assert!(
        (c1 - expected_c1).length() < 0.01,
        "Control1 should be at 1/3"
    );
    assert!(
        (c2 - expected_c2).length() < 0.01,
        "Control2 should be at 2/3"
    );
}

#[test]
fn test_fit_three_points_returns_two_smooth_curves() {
    let points = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(150.0, 100.0),
        Vec2::new(300.0, 0.0),
    ];
    let result = fit_catmull_rom_beziers(&points);

    assert_eq!(result.len(), 2, "Three points should produce 2 segments");

    // Verify endpoint continuity: end of segment 0 == start of segment 1
    let (_, _, _, end0) = result[0];
    let (start1, _, _, _) = result[1];
    assert!(
        (end0 - start1).length() < 0.01,
        "Segments should share endpoints (C0 continuity)"
    );

    // Verify C1 continuity at the shared point: tangent directions should be equal
    // The tangent at t=1 of segment 0 should be parallel to tangent at t=0 of segment 1
    let (_p0_0, _c1_0, c2_0, p3_0) = result[0];
    let (p0_1, c1_1, _c2_1, _p3_1) = result[1];

    // Tangent at t=1: 3*(p3 - c2), tangent at t=0: 3*(c1 - p0)
    let tangent_end_0 = p3_0 - c2_0;
    let tangent_start_1 = c1_1 - p0_1;

    // They should point in the same direction (cross product ~ 0)
    let cross = tangent_end_0.x * tangent_start_1.y - tangent_end_0.y * tangent_start_1.x;
    assert!(
        cross.abs() < 1.0,
        "Tangents at shared point should be aligned (C1 continuity), cross={}",
        cross
    );
}

#[test]
fn test_fit_five_points_all_segments_valid() {
    let points = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 50.0),
        Vec2::new(200.0, 0.0),
        Vec2::new(300.0, -50.0),
        Vec2::new(400.0, 0.0),
    ];
    let result = fit_catmull_rom_beziers(&points);

    assert_eq!(result.len(), 4, "Five points should produce 4 segments");

    for (i, (p0, c1, c2, p3)) in result.iter().enumerate() {
        // All control points should be finite
        assert!(p0.is_finite(), "Segment {} p0 should be finite", i);
        assert!(c1.is_finite(), "Segment {} c1 should be finite", i);
        assert!(c2.is_finite(), "Segment {} c2 should be finite", i);
        assert!(p3.is_finite(), "Segment {} p3 should be finite", i);

        // Control points should not be degenerate (not collapsed to endpoints)
        // unless the segment is very short
        let seg_len = (*p3 - *p0).length();
        if seg_len > 10.0 {
            assert!(
                (*c1 - *p0).length() > 0.1,
                "Segment {} c1 should not collapse to p0",
                i
            );
            assert!(
                (*c2 - *p3).length() > 0.1,
                "Segment {} c2 should not collapse to p3",
                i
            );
        }
    }

    // Verify endpoint continuity across all adjacent pairs
    for i in 0..(result.len() - 1) {
        let (_, _, _, end) = result[i];
        let (start, _, _, _) = result[i + 1];
        assert!(
            (end - start).length() < 0.01,
            "Segments {} and {} should share endpoints",
            i,
            i + 1
        );
    }
}

#[test]
fn test_bezier_point_endpoints() {
    let p0 = Vec2::new(10.0, 20.0);
    let p1 = Vec2::new(30.0, 50.0);
    let p2 = Vec2::new(70.0, 50.0);
    let p3 = Vec2::new(90.0, 20.0);

    let start = bezier_point(p0, p1, p2, p3, 0.0);
    let end = bezier_point(p0, p1, p2, p3, 1.0);

    assert!((start - p0).length() < 0.01);
    assert!((end - p3).length() < 0.01);
}

#[test]
fn test_bezier_arc_length_straight_line() {
    let p0 = Vec2::new(0.0, 0.0);
    let p1 = Vec2::new(100.0, 0.0);
    let p2 = Vec2::new(200.0, 0.0);
    let p3 = Vec2::new(300.0, 0.0);

    let length = bezier_arc_length(p0, p1, p2, p3, 64);
    assert!(
        (length - 300.0).abs() < 1.0,
        "Arc length of straight Bezier should equal endpoint distance, got {}",
        length
    );
}

#[test]
fn test_fit_collinear_points_produces_near_straight_curves() {
    // Points along a straight line should produce near-straight Bezier curves
    let points: Vec<Vec2> = (0..5).map(|i| Vec2::new(i as f32 * 100.0, 0.0)).collect();
    let result = fit_catmull_rom_beziers(&points);

    assert_eq!(result.len(), 4);

    for (p0, c1, c2, p3) in &result {
        // Sample the curve and verify all points are close to y=0
        for j in 0..=10 {
            let t = j as f32 / 10.0;
            let pt = bezier_point(*p0, *c1, *c2, *p3, t);
            assert!(
                pt.y.abs() < 1.0,
                "Collinear input should produce near-straight curves, got y={}",
                pt.y
            );
        }
    }
}

//! Curve road drawing mode for placing Bezier-curved road segments.
//!
//! When enabled, road drawing uses a 3-click workflow:
//! 1. First click — place start point
//! 2. Second click — place control point (defines curve shape)
//! 3. Third click — place end point and commit the curved segment
//!
//! The control point is converted from a quadratic-style user control point
//! into the cubic Bezier control points (p1, p2) used by `RoadSegment`.

use bevy::prelude::*;

/// Resource that tracks whether curve drawing mode is active.
/// When disabled, road drawing uses the normal 2-click straight-line mode.
#[derive(Resource, Default)]
pub struct CurveDrawMode {
    pub enabled: bool,
}

/// Convert a user-specified quadratic control point into cubic Bezier control
/// points. Given start (p0), user control point (c), and end (p3), the cubic
/// control points are computed using the standard quadratic-to-cubic promotion:
///   p1 = p0 + 2/3 * (c - p0)
///   p2 = p3 + 2/3 * (c - p3)
///
/// This produces a cubic Bezier that is exactly equivalent to the quadratic
/// Bezier through the three points, giving intuitive "pull toward control point"
/// behavior.
pub fn quadratic_to_cubic(p0: Vec2, control: Vec2, p3: Vec2) -> (Vec2, Vec2) {
    let p1 = p0 + (2.0 / 3.0) * (control - p0);
    let p2 = p3 + (2.0 / 3.0) * (control - p3);
    (p1, p2)
}

pub struct CurveRoadDrawingPlugin;

impl Plugin for CurveRoadDrawingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurveDrawMode>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_to_cubic_straight_line() {
        // When control point is midpoint, result should be a straight line
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(300.0, 0.0);
        let control = Vec2::new(150.0, 0.0); // midpoint

        let (p1, p2) = quadratic_to_cubic(p0, control, p3);

        // For a straight line, p1 should be at 1/3 and p2 at 2/3
        assert!((p1 - Vec2::new(100.0, 0.0)).length() < 0.01);
        assert!((p2 - Vec2::new(200.0, 0.0)).length() < 0.01);
    }

    #[test]
    fn test_quadratic_to_cubic_curved() {
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(300.0, 0.0);
        let control = Vec2::new(150.0, 200.0); // above midpoint

        let (p1, p2) = quadratic_to_cubic(p0, control, p3);

        // p1 should have positive y (pulled toward control)
        assert!(p1.y > 100.0);
        // p2 should also have positive y
        assert!(p2.y > 100.0);
        // Both should have roughly 2/3 * 200 = 133.3 for y
        assert!((p1.y - 200.0 * 2.0 / 3.0).abs() < 0.01);
        assert!((p2.y - 200.0 * 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_quadratic_to_cubic_endpoints_match() {
        // The cubic Bezier evaluated at t=0 should be p0, at t=1 should be p3
        let p0 = Vec2::new(10.0, 20.0);
        let p3 = Vec2::new(200.0, 50.0);
        let control = Vec2::new(100.0, 150.0);

        let (p1, p2) = quadratic_to_cubic(p0, control, p3);

        // Evaluate at t=0: should be p0
        let at_0 = eval_cubic(p0, p1, p2, p3, 0.0);
        assert!((at_0 - p0).length() < 0.01);

        // Evaluate at t=1: should be p3
        let at_1 = eval_cubic(p0, p1, p2, p3, 1.0);
        assert!((at_1 - p3).length() < 0.01);
    }

    fn eval_cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
        let u = 1.0 - t;
        let uu = u * u;
        let tt = t * t;
        u * uu * p0 + 3.0 * uu * t * p1 + 3.0 * u * tt * p2 + t * tt * p3
    }
}

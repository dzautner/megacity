//! Tests for road grade indicator helpers.

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use simulation::grid::{CellType, WorldGrid};

    use crate::input::ActiveTool;
    use crate::road_grade::constants::{GRADE_LOW_THRESHOLD, GRADE_MEDIUM_THRESHOLD};
    use crate::road_grade::helpers::{
        approximate_arc_length, evaluate_bezier, grade_to_color, is_road_tool, lerp_color,
        sample_cell_type_at, sample_elevation_at,
    };

    #[test]
    fn test_grade_to_color_low() {
        // 0% grade should be green
        let c = grade_to_color(0.0);
        let s = c.to_srgba();
        assert!(
            s.green > s.red,
            "0% grade should be green, got r={} g={}",
            s.red,
            s.green
        );
    }

    #[test]
    fn test_grade_to_color_medium() {
        // 4.5% grade should be between green and yellow (yellowish)
        let c = grade_to_color(0.045);
        let s = c.to_srgba();
        assert!(
            s.green > 0.5,
            "4.5% grade green should be > 0.5, got {}",
            s.green
        );
        assert!(s.red > 0.3, "4.5% grade red should be > 0.3, got {}", s.red);
    }

    #[test]
    fn test_grade_to_color_high() {
        // 10% grade should be red
        let c = grade_to_color(0.10);
        let s = c.to_srgba();
        assert!(
            s.red > s.green,
            "10% grade should be red, got r={} g={}",
            s.red,
            s.green
        );
    }

    #[test]
    fn test_grade_to_color_at_thresholds() {
        // At exactly 3% threshold, should be green
        let c_low = grade_to_color(GRADE_LOW_THRESHOLD);
        let s_low = c_low.to_srgba();
        assert!(
            s_low.green > s_low.red,
            "at 3% threshold should still be green"
        );

        // At exactly 6% threshold, should be yellow
        let c_med = grade_to_color(GRADE_MEDIUM_THRESHOLD);
        let s_med = c_med.to_srgba();
        assert!(s_med.red > 0.5, "at 6% threshold red should be > 0.5");
        assert!(s_med.green > 0.5, "at 6% threshold green should be > 0.5");
    }

    #[test]
    fn test_lerp_color_endpoints() {
        let a = Color::srgba(0.0, 0.0, 0.0, 1.0);
        let b = Color::srgba(1.0, 1.0, 1.0, 1.0);

        let start = lerp_color(a, b, 0.0).to_srgba();
        assert!((start.red - 0.0).abs() < 1e-5);

        let end = lerp_color(a, b, 1.0).to_srgba();
        assert!((end.red - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_lerp_color_midpoint() {
        let a = Color::srgba(0.0, 0.0, 0.0, 1.0);
        let b = Color::srgba(1.0, 1.0, 1.0, 1.0);
        let mid = lerp_color(a, b, 0.5).to_srgba();
        assert!((mid.red - 0.5).abs() < 1e-5);
        assert!((mid.green - 0.5).abs() < 1e-5);
        assert!((mid.blue - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_lerp_color_clamps() {
        let a = Color::srgba(0.2, 0.3, 0.4, 1.0);
        let b = Color::srgba(0.8, 0.7, 0.6, 1.0);

        let below = lerp_color(a, b, -1.0).to_srgba();
        let at_zero = lerp_color(a, b, 0.0).to_srgba();
        assert!((below.red - at_zero.red).abs() < 1e-5);

        let above = lerp_color(a, b, 2.0).to_srgba();
        let at_one = lerp_color(a, b, 1.0).to_srgba();
        assert!((above.red - at_one.red).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_bezier_endpoints() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(10.0, 0.0);
        let p2 = Vec2::new(20.0, 0.0);
        let p3 = Vec2::new(30.0, 0.0);

        let start = evaluate_bezier(p0, p1, p2, p3, 0.0);
        assert!((start - p0).length() < 1e-5);

        let end = evaluate_bezier(p0, p1, p2, p3, 1.0);
        assert!((end - p3).length() < 1e-5);
    }

    #[test]
    fn test_approximate_arc_length_straight_line() {
        let p0 = Vec2::new(0.0, 0.0);
        let p3 = Vec2::new(100.0, 0.0);
        let p1 = Vec2::new(33.33, 0.0);
        let p2 = Vec2::new(66.67, 0.0);

        let length = approximate_arc_length(p0, p1, p2, p3);
        assert!(
            (length - 100.0).abs() < 1.0,
            "straight line arc length should be ~100, got {length}"
        );
    }

    #[test]
    fn test_approximate_arc_length_zero() {
        let p = Vec2::new(50.0, 50.0);
        let length = approximate_arc_length(p, p, p, p);
        assert!(
            length < 0.01,
            "zero-length curve should have ~0 arc length, got {length}"
        );
    }

    #[test]
    fn test_sample_elevation_at_in_bounds() {
        let mut grid = WorldGrid::new(16, 16);
        grid.get_mut(5, 5).elevation = 0.75;
        let (wx, wy) = WorldGrid::grid_to_world(5, 5);
        let elev = sample_elevation_at(&grid, Vec2::new(wx, wy));
        assert!(
            (elev - 0.75).abs() < 1e-5,
            "should sample correct elevation, got {elev}"
        );
    }

    #[test]
    fn test_sample_elevation_at_out_of_bounds() {
        let grid = WorldGrid::new(16, 16);
        let elev = sample_elevation_at(&grid, Vec2::new(-100.0, -100.0));
        assert!(
            (elev - 0.0).abs() < 1e-5,
            "out-of-bounds should return 0.0, got {elev}"
        );
    }

    #[test]
    fn test_sample_cell_type_at_water() {
        let mut grid = WorldGrid::new(16, 16);
        grid.get_mut(3, 7).cell_type = CellType::Water;
        let (wx, wy) = WorldGrid::grid_to_world(3, 7);
        let ct = sample_cell_type_at(&grid, Vec2::new(wx, wy));
        assert_eq!(ct, CellType::Water);
    }

    #[test]
    fn test_sample_cell_type_at_out_of_bounds() {
        let grid = WorldGrid::new(16, 16);
        let ct = sample_cell_type_at(&grid, Vec2::new(-50.0, -50.0));
        assert_eq!(ct, CellType::Grass, "out-of-bounds should default to Grass");
    }

    #[test]
    fn test_is_road_tool() {
        assert!(is_road_tool(&ActiveTool::Road));
        assert!(is_road_tool(&ActiveTool::RoadAvenue));
        assert!(is_road_tool(&ActiveTool::RoadBoulevard));
        assert!(is_road_tool(&ActiveTool::RoadHighway));
        assert!(is_road_tool(&ActiveTool::RoadOneWay));
        assert!(is_road_tool(&ActiveTool::RoadPath));
        assert!(!is_road_tool(&ActiveTool::Bulldoze));
        assert!(!is_road_tool(&ActiveTool::Inspect));
    }

    #[test]
    fn test_grade_color_monotonic_red() {
        let low = grade_to_color(0.01).to_srgba();
        let high = grade_to_color(0.12).to_srgba();
        assert!(
            high.red > low.red,
            "higher grade should have more red: low.r={} high.r={}",
            low.red,
            high.red
        );
    }

    #[test]
    fn test_grade_color_low_is_green_dominant() {
        let c = grade_to_color(0.01).to_srgba();
        assert!(
            c.green > c.red && c.green > c.blue,
            "low grade should be green dominant"
        );
    }
}

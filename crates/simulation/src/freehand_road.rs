//! Freehand road drawing mode (UX-020).
//!
//! When freehand mode is active and a road tool is selected, the player can
//! hold the mouse button and draw roads by moving the cursor freely. The path
//! is simplified using Ramer-Douglas-Peucker to avoid generating too many
//! tiny segments, then road segments are created along the simplified path.

use bevy::prelude::*;

/// Minimum distance (world units) between consecutive raw sample points.
/// Roughly 3 grid cells (CELL_SIZE = 16.0).
pub const FREEHAND_MIN_SAMPLE_DIST: f32 = 48.0;

/// Ramer-Douglas-Peucker simplification tolerance (world units).
/// One grid cell width.
pub const FREEHAND_SIMPLIFY_TOLERANCE: f32 = 16.0;

/// Minimum segment length after simplification (world units).
/// Prevents degenerate micro-segments.
pub const FREEHAND_MIN_SEGMENT_LEN: f32 = 24.0;

/// Resource tracking freehand drawing state.
#[derive(Resource, Default)]
pub struct FreehandDrawState {
    /// Whether freehand drawing mode is enabled.
    pub enabled: bool,
    /// Whether the user is currently drawing (mouse held down).
    pub drawing: bool,
    /// Raw sample points collected during the current stroke (world coords).
    pub raw_points: Vec<Vec2>,
}

impl FreehandDrawState {
    /// Reset the current stroke without changing the enabled state.
    pub fn reset_stroke(&mut self) {
        self.drawing = false;
        self.raw_points.clear();
    }

    /// Add a sample point, enforcing minimum distance from the previous point.
    /// Returns `true` if the point was added.
    pub fn add_sample(&mut self, pos: Vec2) -> bool {
        if let Some(&last) = self.raw_points.last() {
            if (pos - last).length() < FREEHAND_MIN_SAMPLE_DIST {
                return false;
            }
        }
        self.raw_points.push(pos);
        true
    }
}

/// Ramer-Douglas-Peucker line simplification.
///
/// Given a polyline, returns a simplified version with fewer points while
/// preserving the overall shape within the given `tolerance`.
pub fn simplify_rdp(points: &[Vec2], tolerance: f32) -> Vec<Vec2> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let first = points[0];
    let Some(&last) = points.last() else { return points.to_vec(); };

    // Find point with maximum perpendicular distance from the line (first, last)
    let mut max_dist = 0.0_f32;
    let mut max_idx = 0;

    let line_dir = last - first;
    let line_len = line_dir.length();

    for (i, &pt) in points.iter().enumerate().skip(1).take(points.len() - 2) {
        let dist = if line_len < 1e-6 {
            (pt - first).length()
        } else {
            let t = (pt - first).dot(line_dir) / (line_len * line_len);
            let proj = first + line_dir * t.clamp(0.0, 1.0);
            (pt - proj).length()
        };
        if dist > max_dist {
            max_dist = dist;
            max_idx = i;
        }
    }

    if max_dist > tolerance {
        // Recurse on both halves
        let mut left = simplify_rdp(&points[..=max_idx], tolerance);
        let right = simplify_rdp(&points[max_idx..], tolerance);
        // Remove duplicate point at the split
        left.pop();
        left.extend(right);
        left
    } else {
        // All intermediate points are within tolerance; keep only endpoints
        vec![first, last]
    }
}

/// Filter out segments shorter than `min_len` by merging consecutive short hops.
pub fn filter_short_segments(points: &[Vec2], min_len: f32) -> Vec<Vec2> {
    if points.len() <= 1 {
        return points.to_vec();
    }

    let mut result = vec![points[0]];

    for &pt in &points[1..] {
        let Some(&last) = result.last() else { continue; };
        if (pt - last).length() >= min_len {
            result.push(pt);
        }
    }

    // Always keep the last point if there are at least 2 input points
    if result.len() == 1 && points.len() >= 2 {
        let Some(&last) = points.last() else { return result; };
        if (last - result[0]).length() >= min_len {
            result.push(last);
        }
    } else if let Some(&last_result) = result.last() {
        let Some(&last_input) = points.last() else { return result; };
        if (last_input - last_result).length() > 1e-3 {
            // The last input point was filtered out; ensure we include it
            // only if it's far enough from the previous kept point
            if (last_input - last_result).length() >= min_len {
                result.push(last_input);
            } else {
                // Replace the last kept point with the final input point
                // to ensure the stroke ends at the cursor position
                if result.len() > 1 {
                    if let Some(last) = result.last_mut() {
                        *last = last_input;
                    }
                }
            }
        }
    }

    result
}

pub struct FreehandRoadPlugin;

impl Plugin for FreehandRoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FreehandDrawState>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_rdp_straight_line() {
        // Points along a straight line should simplify to just endpoints
        let points: Vec<Vec2> = (0..10).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();
        let simplified = simplify_rdp(&points, 1.0);
        assert_eq!(simplified.len(), 2);
        assert!((simplified[0] - Vec2::new(0.0, 0.0)).length() < 0.01);
        assert!((simplified[1] - Vec2::new(90.0, 0.0)).length() < 0.01);
    }

    #[test]
    fn test_simplify_rdp_preserves_corners() {
        // L-shaped path should keep the corner
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 50.0),
            Vec2::new(100.0, 100.0),
        ];
        let simplified = simplify_rdp(&points, 5.0);
        // Should keep start, corner, and end
        assert!(simplified.len() >= 3);
        assert!((simplified[0] - Vec2::new(0.0, 0.0)).length() < 0.01);
        assert!((simplified.last().unwrap() - Vec2::new(100.0, 100.0)).length() < 0.01);
    }

    #[test]
    fn test_simplify_rdp_two_points() {
        let points = vec![Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)];
        let simplified = simplify_rdp(&points, 1.0);
        assert_eq!(simplified.len(), 2);
    }

    #[test]
    fn test_simplify_rdp_single_point() {
        let points = vec![Vec2::new(42.0, 42.0)];
        let simplified = simplify_rdp(&points, 1.0);
        assert_eq!(simplified.len(), 1);
    }

    #[test]
    fn test_simplify_rdp_empty() {
        let points: Vec<Vec2> = vec![];
        let simplified = simplify_rdp(&points, 1.0);
        assert!(simplified.is_empty());
    }

    #[test]
    fn test_filter_short_segments() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),   // too close to previous
            Vec2::new(100.0, 0.0), // far enough
            Vec2::new(105.0, 0.0), // too close
            Vec2::new(200.0, 0.0), // far enough
        ];
        let filtered = filter_short_segments(&points, 30.0);
        assert_eq!(filtered.len(), 3);
        assert!((filtered[0] - Vec2::new(0.0, 0.0)).length() < 0.01);
        assert!((filtered[1] - Vec2::new(100.0, 0.0)).length() < 0.01);
        assert!((filtered[2] - Vec2::new(200.0, 0.0)).length() < 0.01);
    }

    #[test]
    fn test_add_sample_enforces_min_distance() {
        let mut state = FreehandDrawState::default();
        assert!(state.add_sample(Vec2::new(0.0, 0.0)));
        // Too close
        assert!(!state.add_sample(Vec2::new(10.0, 0.0)));
        // Far enough
        assert!(state.add_sample(Vec2::new(100.0, 0.0)));
        assert_eq!(state.raw_points.len(), 2);
    }

    #[test]
    fn test_reset_stroke() {
        let mut state = FreehandDrawState::default();
        state.enabled = true;
        state.drawing = true;
        state.raw_points.push(Vec2::ZERO);
        state.reset_stroke();
        assert!(state.enabled); // enabled stays on
        assert!(!state.drawing);
        assert!(state.raw_points.is_empty());
    }

    #[test]
    fn test_simplify_rdp_curve() {
        // Approximate a quarter circle with many points
        let n = 20;
        let points: Vec<Vec2> = (0..=n)
            .map(|i| {
                let angle = std::f32::consts::FRAC_PI_2 * (i as f32 / n as f32);
                Vec2::new(angle.cos() * 200.0, angle.sin() * 200.0)
            })
            .collect();
        let simplified = simplify_rdp(&points, 5.0);
        // Should have fewer points than the original but more than 2
        assert!(simplified.len() < points.len());
        assert!(simplified.len() > 2);
    }
}

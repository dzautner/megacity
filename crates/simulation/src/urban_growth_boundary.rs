use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Urban Growth Boundary (UGB) resource.
///
/// Models Portland, Oregon-style urban growth boundaries: a polygon drawn on the map
/// inside which urban development is permitted. Cells outside the boundary cannot be
/// zoned (except agricultural/rural if those zone types exist). Land values inside
/// the boundary receive a scarcity premium while land values outside drop.
///
/// When `enabled` is false (default), the UGB has no effect and all cells are
/// considered inside the boundary (backward-compatible behavior).
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct UrbanGrowthBoundary {
    /// Whether the UGB is active.
    pub enabled: bool,
    /// Polygon vertices as grid coordinates (x, y). The polygon is implicitly closed
    /// (an edge connects the last vertex back to the first).
    pub vertices: Vec<(f32, f32)>,
}

/// Land value modifier applied inside the UGB (scarcity premium).
const UGB_INSIDE_PREMIUM: i32 = 8;

/// Land value modifier applied outside the UGB (development restriction penalty).
const UGB_OUTSIDE_PENALTY: i32 = 12;

impl UrbanGrowthBoundary {
    /// Returns true if the UGB is active and has a valid polygon (>= 3 vertices).
    pub fn is_active(&self) -> bool {
        self.enabled && self.vertices.len() >= 3
    }

    /// Check whether a grid cell (x, y) is inside the UGB polygon.
    /// When the UGB is not active, all cells are considered "inside".
    pub fn contains(&self, x: usize, y: usize) -> bool {
        if !self.is_active() {
            return true;
        }
        // Use cell center for the point-in-polygon test.
        let px = x as f32 + 0.5;
        let py = y as f32 + 0.5;
        point_in_polygon(px, py, &self.vertices)
    }

    /// Returns true if zoning is allowed at the given cell.
    /// When UGB is active, only cells inside the boundary may be zoned.
    pub fn allows_zoning(&self, x: usize, y: usize) -> bool {
        self.contains(x, y)
    }

    /// Returns true if building upgrades are allowed at the given cell.
    /// Existing buildings outside the boundary remain but cannot be upgraded.
    pub fn allows_upgrade(&self, x: usize, y: usize) -> bool {
        self.contains(x, y)
    }

    /// Returns the land value modifier for a cell.
    /// Positive inside the boundary (scarcity premium), negative outside.
    /// Returns 0 when UGB is not active.
    pub fn land_value_modifier(&self, x: usize, y: usize) -> i32 {
        if !self.is_active() {
            return 0;
        }
        if self.contains(x, y) {
            UGB_INSIDE_PREMIUM
        } else {
            -UGB_OUTSIDE_PENALTY
        }
    }

    /// Add a vertex to the boundary polygon.
    pub fn add_vertex(&mut self, x: f32, y: f32) {
        self.vertices.push((x, y));
    }

    /// Clear all vertices and disable the boundary.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.enabled = false;
    }

    /// Expand the boundary by adding a new vertex. The boundary remains enabled.
    pub fn expand_with_vertex(&mut self, x: f32, y: f32) {
        self.vertices.push((x, y));
    }
}

/// Ray-casting point-in-polygon test.
/// Returns true if point (px, py) is inside the polygon defined by `vertices`.
/// The polygon is implicitly closed (last vertex connects back to first).
fn point_in_polygon(px: f32, py: f32, vertices: &[(f32, f32)]) -> bool {
    let n = vertices.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = vertices[i];
        let (xj, yj) = vertices[j];
        // Check if the ray from (px, py) going right crosses edge (i, j).
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_ugb() -> UrbanGrowthBoundary {
        // A square from (10, 10) to (100, 100) in grid coords.
        UrbanGrowthBoundary {
            enabled: true,
            vertices: vec![(10.0, 10.0), (100.0, 10.0), (100.0, 100.0), (10.0, 100.0)],
        }
    }

    #[test]
    fn test_disabled_ugb_allows_everything() {
        let ugb = UrbanGrowthBoundary::default();
        assert!(!ugb.is_active());
        assert!(ugb.contains(0, 0));
        assert!(ugb.contains(128, 128));
        assert!(ugb.allows_zoning(200, 200));
        assert_eq!(ugb.land_value_modifier(50, 50), 0);
    }

    #[test]
    fn test_active_ugb_inside() {
        let ugb = square_ugb();
        assert!(ugb.is_active());
        // Cell (50, 50) center is (50.5, 50.5) -- inside the square.
        assert!(ugb.contains(50, 50));
        assert!(ugb.allows_zoning(50, 50));
        assert!(ugb.allows_upgrade(50, 50));
        assert_eq!(ugb.land_value_modifier(50, 50), UGB_INSIDE_PREMIUM);
    }

    #[test]
    fn test_active_ugb_outside() {
        let ugb = square_ugb();
        // Cell (5, 5) center is (5.5, 5.5) -- outside the square.
        assert!(!ugb.contains(5, 5));
        assert!(!ugb.allows_zoning(5, 5));
        assert!(!ugb.allows_upgrade(5, 5));
        assert_eq!(ugb.land_value_modifier(5, 5), -UGB_OUTSIDE_PENALTY);
    }

    #[test]
    fn test_active_ugb_outside_far() {
        let ugb = square_ugb();
        // Cell (200, 200) is well outside the boundary.
        assert!(!ugb.contains(200, 200));
        assert!(!ugb.allows_zoning(200, 200));
    }

    #[test]
    fn test_ugb_boundary_edge() {
        let ugb = square_ugb();
        // Cell (10, 10) center is (10.5, 10.5) -- just inside.
        assert!(ugb.contains(10, 10));
        // Cell (9, 9) center is (9.5, 9.5) -- just outside.
        assert!(!ugb.contains(9, 9));
    }

    #[test]
    fn test_point_in_polygon_triangle() {
        let tri = vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)];
        assert!(point_in_polygon(5.0, 5.0, &tri));
        assert!(!point_in_polygon(0.0, 10.0, &tri));
        assert!(!point_in_polygon(20.0, 5.0, &tri));
    }

    #[test]
    fn test_point_in_polygon_insufficient_vertices() {
        assert!(!point_in_polygon(5.0, 5.0, &[]));
        assert!(!point_in_polygon(5.0, 5.0, &[(0.0, 0.0)]));
        assert!(!point_in_polygon(5.0, 5.0, &[(0.0, 0.0), (10.0, 10.0)]));
    }

    #[test]
    fn test_ugb_not_active_with_fewer_than_3_vertices() {
        let ugb = UrbanGrowthBoundary {
            enabled: true,
            vertices: vec![(10.0, 10.0), (100.0, 10.0)],
        };
        assert!(!ugb.is_active());
        // Not active means everything is "inside".
        assert!(ugb.contains(200, 200));
    }

    #[test]
    fn test_add_vertex_and_expand() {
        let mut ugb = UrbanGrowthBoundary::default();
        ugb.enabled = true;
        ugb.add_vertex(0.0, 0.0);
        ugb.add_vertex(100.0, 0.0);
        ugb.add_vertex(100.0, 100.0);
        assert!(ugb.is_active());
        assert!(ugb.contains(50, 50));

        // Expand with another vertex.
        ugb.expand_with_vertex(0.0, 100.0);
        assert_eq!(ugb.vertices.len(), 4);
    }

    #[test]
    fn test_clear_ugb() {
        let mut ugb = square_ugb();
        assert!(ugb.is_active());
        ugb.clear();
        assert!(!ugb.is_active());
        assert!(ugb.vertices.is_empty());
    }
}

pub struct UrbanGrowthBoundaryPlugin;

impl Plugin for UrbanGrowthBoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UrbanGrowthBoundary>();
    }
}

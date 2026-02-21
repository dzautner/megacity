use bevy::prelude::*;

use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};

const BUCKET_SIZE: f32 = 128.0; // pixels per spatial bucket
const BUCKETS_X: usize = (GRID_WIDTH as f32 * CELL_SIZE / BUCKET_SIZE) as usize + 1;
const BUCKETS_Y: usize = (GRID_HEIGHT as f32 * CELL_SIZE / BUCKET_SIZE) as usize + 1;
const TOTAL_BUCKETS: usize = BUCKETS_X * BUCKETS_Y;

#[derive(Resource)]
pub struct SpatialGrid {
    buckets: Vec<Vec<Entity>>,
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self {
            buckets: (0..TOTAL_BUCKETS).map(|_| Vec::new()).collect(),
        }
    }
}

impl SpatialGrid {
    pub fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
    }

    pub fn insert(&mut self, entity: Entity, x: f32, y: f32) {
        let bx = (x / BUCKET_SIZE).floor() as i32;
        let by = (y / BUCKET_SIZE).floor() as i32;
        if let Some(idx) = Self::flat_index(bx, by) {
            self.buckets[idx].push(entity);
        }
    }

    pub fn query_rect(&self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Vec<Entity> {
        let min_bx = (min_x / BUCKET_SIZE).floor() as i32;
        let min_by = (min_y / BUCKET_SIZE).floor() as i32;
        let max_bx = (max_x / BUCKET_SIZE).floor() as i32;
        let max_by = (max_y / BUCKET_SIZE).floor() as i32;

        let mut result = Vec::new();
        for by in min_by..=max_by {
            for bx in min_bx..=max_bx {
                if let Some(idx) = Self::flat_index(bx, by) {
                    result.extend_from_slice(&self.buckets[idx]);
                }
            }
        }
        result
    }

    #[inline]
    fn flat_index(bx: i32, by: i32) -> Option<usize> {
        if bx >= 0 && by >= 0 && (bx as usize) < BUCKETS_X && (by as usize) < BUCKETS_Y {
            Some(by as usize * BUCKETS_X + bx as usize)
        } else {
            None
        }
    }

    pub fn entity_count(&self) -> usize {
        self.buckets.iter().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // World is 4096x4096 pixels (256 grid cells * 16.0 CELL_SIZE)
    // Buckets are 128x128 pixels, giving 33x33 buckets

    // ------------------------------------------------------------------
    // Basic insertion and querying
    // ------------------------------------------------------------------

    #[test]
    fn test_spatial_insert_query() {
        let mut grid = SpatialGrid::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        grid.insert(e1, 50.0, 50.0);
        grid.insert(e2, 200.0, 200.0);
        grid.insert(e3, 1000.0, 1000.0);

        let result = grid.query_rect(0.0, 0.0, 300.0, 300.0);
        assert!(result.contains(&e1));
        assert!(result.contains(&e2));
        assert!(!result.contains(&e3));
    }

    #[test]
    fn test_spatial_clear() {
        let mut grid = SpatialGrid::default();
        grid.insert(Entity::from_raw(1), 50.0, 50.0);
        assert_eq!(grid.entity_count(), 1);

        grid.clear();
        assert_eq!(grid.entity_count(), 0);
    }

    // ------------------------------------------------------------------
    // Empty grid returns no results
    // ------------------------------------------------------------------

    #[test]
    fn test_empty_grid_query_returns_empty() {
        let grid = SpatialGrid::default();
        let result = grid.query_rect(0.0, 0.0, 4096.0, 4096.0);
        assert!(
            result.is_empty(),
            "querying an empty grid should return no entities"
        );
    }

    #[test]
    fn test_empty_grid_entity_count_is_zero() {
        let grid = SpatialGrid::default();
        assert_eq!(grid.entity_count(), 0);
    }

    // ------------------------------------------------------------------
    // Nearest lookup: query_rect returns closest entity in bucket
    // ------------------------------------------------------------------

    #[test]
    fn test_nearest_lookup_returns_closest_entity() {
        // Insert several entities; query a small rect around a point
        // to find the closest one (simulating nearest-lookup via rect query).
        let mut grid = SpatialGrid::default();
        let close = Entity::from_raw(10);
        let medium = Entity::from_raw(20);
        let far = Entity::from_raw(30);

        // Place entities at increasing distances from (100, 100)
        grid.insert(close, 110.0, 110.0); // ~14 pixels away
        grid.insert(medium, 200.0, 200.0); // ~141 pixels away
        grid.insert(far, 500.0, 500.0); // ~566 pixels away

        // Small rect around (100, 100) should find only the close entity
        // close is at (110, 110), in bucket (0, 0) (since 110/128 = 0)
        // A rect from (80, 80) to (127, 127) covers only bucket (0, 0)
        let result = grid.query_rect(80.0, 80.0, 127.0, 127.0);
        assert!(
            result.contains(&close),
            "close entity should be in the small rect"
        );
        assert!(
            !result.contains(&medium),
            "medium entity should not be in the small rect"
        );
        assert!(
            !result.contains(&far),
            "far entity should not be in the small rect"
        );
    }

    // ------------------------------------------------------------------
    // Multiple destinations: correct closest for various query points
    // ------------------------------------------------------------------

    #[test]
    fn test_multiple_destinations_correct_closest() {
        let mut grid = SpatialGrid::default();

        // Place entities in distinct buckets across the map
        let nw = Entity::from_raw(1); // northwest corner
        let ne = Entity::from_raw(2); // northeast corner
        let sw = Entity::from_raw(3); // southwest corner
        let se = Entity::from_raw(4); // southeast corner
        let center = Entity::from_raw(5); // center

        grid.insert(nw, 64.0, 64.0); // bucket (0, 0)
        grid.insert(ne, 3900.0, 64.0); // bucket (30, 0)
        grid.insert(sw, 64.0, 3900.0); // bucket (0, 30)
        grid.insert(se, 3900.0, 3900.0); // bucket (30, 30)
        grid.insert(center, 2048.0, 2048.0); // bucket (16, 16)

        // Query near northwest corner - should only find nw
        let nw_result = grid.query_rect(0.0, 0.0, 127.0, 127.0);
        assert!(nw_result.contains(&nw));
        assert_eq!(nw_result.len(), 1);

        // Query near northeast corner - should only find ne
        let ne_result = grid.query_rect(3840.0, 0.0, 3967.0, 127.0);
        assert!(ne_result.contains(&ne));
        assert_eq!(ne_result.len(), 1);

        // Query near center - should only find center
        let center_result = grid.query_rect(2000.0, 2000.0, 2100.0, 2100.0);
        assert!(center_result.contains(&center));
        assert_eq!(center_result.len(), 1);

        // Query the entire world - should find all 5
        let all_result = grid.query_rect(0.0, 0.0, 4095.0, 4095.0);
        assert_eq!(all_result.len(), 5);
    }

    // ------------------------------------------------------------------
    // All destinations within radius (rect approximation)
    // ------------------------------------------------------------------

    #[test]
    fn test_all_destinations_within_radius_found() {
        let mut grid = SpatialGrid::default();

        // Place a cluster of entities around (500, 500)
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);
        let e4 = Entity::from_raw(4);
        let e_far = Entity::from_raw(5);

        grid.insert(e1, 480.0, 480.0);
        grid.insert(e2, 500.0, 520.0);
        grid.insert(e3, 520.0, 490.0);
        grid.insert(e4, 510.0, 510.0);
        grid.insert(e_far, 2000.0, 2000.0); // far away

        // Query a 200x200 rect centered on (500, 500)
        let result = grid.query_rect(400.0, 400.0, 600.0, 600.0);
        assert!(result.contains(&e1), "e1 should be in radius");
        assert!(result.contains(&e2), "e2 should be in radius");
        assert!(result.contains(&e3), "e3 should be in radius");
        assert!(result.contains(&e4), "e4 should be in radius");
        assert!(!result.contains(&e_far), "e_far should not be in radius");
        assert_eq!(result.len(), 4);
    }

    // ------------------------------------------------------------------
    // Boundary conditions
    // ------------------------------------------------------------------

    #[test]
    fn test_boundary_insert_at_origin() {
        let mut grid = SpatialGrid::default();
        let e = Entity::from_raw(1);
        grid.insert(e, 0.0, 0.0);
        assert_eq!(grid.entity_count(), 1);
        let result = grid.query_rect(0.0, 0.0, 1.0, 1.0);
        assert!(result.contains(&e));
    }

    #[test]
    fn test_boundary_insert_at_max_world_edge() {
        let mut grid = SpatialGrid::default();
        let e = Entity::from_raw(1);
        // Place entity near the maximum world coordinate
        let max_coord = (GRID_WIDTH as f32 * CELL_SIZE) - 1.0; // 4095.0
        grid.insert(e, max_coord, max_coord);
        assert_eq!(grid.entity_count(), 1);
        let result = grid.query_rect(max_coord - 10.0, max_coord - 10.0, max_coord, max_coord);
        assert!(result.contains(&e));
    }

    #[test]
    fn test_boundary_negative_coordinates_ignored() {
        let mut grid = SpatialGrid::default();
        let e = Entity::from_raw(1);
        // Negative coords should fall outside valid bucket range
        grid.insert(e, -10.0, -10.0);
        // Entity should not be inserted (flat_index returns None for negative)
        assert_eq!(grid.entity_count(), 0);
    }

    #[test]
    fn test_boundary_beyond_world_coordinates_ignored() {
        let mut grid = SpatialGrid::default();
        let e = Entity::from_raw(1);
        // Way beyond the world boundary
        grid.insert(e, 10000.0, 10000.0);
        // Entity should not be inserted (flat_index returns None)
        assert_eq!(grid.entity_count(), 0);
    }

    #[test]
    fn test_boundary_exact_bucket_edge() {
        let mut grid = SpatialGrid::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);

        // Place entities right at bucket boundary (128.0)
        grid.insert(e1, 127.9, 127.9); // bucket (0, 0)
        grid.insert(e2, 128.0, 128.0); // bucket (1, 1)

        // Query only bucket (0, 0)
        let result = grid.query_rect(0.0, 0.0, 127.0, 127.0);
        assert!(
            result.contains(&e1),
            "e1 at 127.9 should be in bucket (0,0)"
        );
        assert!(
            !result.contains(&e2),
            "e2 at 128.0 should be in bucket (1,1), not (0,0)"
        );
    }

    // ------------------------------------------------------------------
    // Overlapping positions
    // ------------------------------------------------------------------

    #[test]
    fn test_overlapping_positions_all_returned() {
        let mut grid = SpatialGrid::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        // Insert three entities at the exact same position
        grid.insert(e1, 100.0, 100.0);
        grid.insert(e2, 100.0, 100.0);
        grid.insert(e3, 100.0, 100.0);

        assert_eq!(grid.entity_count(), 3);

        let result = grid.query_rect(0.0, 0.0, 200.0, 200.0);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&e1));
        assert!(result.contains(&e2));
        assert!(result.contains(&e3));
    }

    // ------------------------------------------------------------------
    // Clear and reuse
    // ------------------------------------------------------------------

    #[test]
    fn test_clear_then_reinsert() {
        let mut grid = SpatialGrid::default();

        // First pass
        grid.insert(Entity::from_raw(1), 100.0, 100.0);
        grid.insert(Entity::from_raw(2), 200.0, 200.0);
        assert_eq!(grid.entity_count(), 2);

        // Clear
        grid.clear();
        assert_eq!(grid.entity_count(), 0);
        let empty_result = grid.query_rect(0.0, 0.0, 4096.0, 4096.0);
        assert!(empty_result.is_empty());

        // Reinsert different entities
        let e3 = Entity::from_raw(3);
        grid.insert(e3, 300.0, 300.0);
        assert_eq!(grid.entity_count(), 1);
        let result = grid.query_rect(200.0, 200.0, 400.0, 400.0);
        assert!(result.contains(&e3));
    }

    // ------------------------------------------------------------------
    // Query with no matches in valid range
    // ------------------------------------------------------------------

    #[test]
    fn test_query_rect_no_match_in_populated_grid() {
        let mut grid = SpatialGrid::default();
        grid.insert(Entity::from_raw(1), 100.0, 100.0);
        grid.insert(Entity::from_raw(2), 200.0, 200.0);

        // Query a region with no entities
        let result = grid.query_rect(3000.0, 3000.0, 3500.0, 3500.0);
        assert!(
            result.is_empty(),
            "query in empty region should return nothing"
        );
    }

    // ------------------------------------------------------------------
    // Large-scale insert and query
    // ------------------------------------------------------------------

    #[test]
    fn test_many_inserts_correct_count() {
        let mut grid = SpatialGrid::default();
        let count = 1000;
        for i in 0..count {
            let x = (i as f32 * 4.0) % 4000.0;
            let y = (i as f32 * 3.0) % 4000.0;
            grid.insert(Entity::from_raw(i), x, y);
        }
        assert_eq!(grid.entity_count(), count as usize);
    }

    // ------------------------------------------------------------------
    // flat_index correctness
    // ------------------------------------------------------------------

    #[test]
    fn test_flat_index_valid_range() {
        // Bucket (0,0) should map to index 0
        assert_eq!(SpatialGrid::flat_index(0, 0), Some(0));
        // Bucket (1,0) should map to index 1
        assert_eq!(SpatialGrid::flat_index(1, 0), Some(1));
        // Bucket (0,1) should map to index BUCKETS_X
        assert_eq!(SpatialGrid::flat_index(0, 1), Some(BUCKETS_X));
        // Last valid bucket
        assert_eq!(
            SpatialGrid::flat_index(BUCKETS_X as i32 - 1, BUCKETS_Y as i32 - 1),
            Some(TOTAL_BUCKETS - 1)
        );
    }

    #[test]
    fn test_flat_index_out_of_bounds() {
        assert_eq!(SpatialGrid::flat_index(-1, 0), None);
        assert_eq!(SpatialGrid::flat_index(0, -1), None);
        assert_eq!(SpatialGrid::flat_index(-1, -1), None);
        assert_eq!(SpatialGrid::flat_index(BUCKETS_X as i32, 0), None);
        assert_eq!(SpatialGrid::flat_index(0, BUCKETS_Y as i32), None);
        assert_eq!(
            SpatialGrid::flat_index(BUCKETS_X as i32, BUCKETS_Y as i32),
            None
        );
    }

    // ------------------------------------------------------------------
    // Query rect spanning negative-to-positive range
    // ------------------------------------------------------------------

    #[test]
    fn test_query_rect_with_negative_coords_clips_to_valid() {
        let mut grid = SpatialGrid::default();
        let e = Entity::from_raw(42);
        grid.insert(e, 10.0, 10.0); // bucket (0, 0)

        // Query rect starting from negative coords but overlapping bucket (0,0)
        let result = grid.query_rect(-100.0, -100.0, 50.0, 50.0);
        assert!(
            result.contains(&e),
            "entity at (10,10) should be found even with negative query bounds"
        );
    }

    // ------------------------------------------------------------------
    // Single-bucket query precision
    // ------------------------------------------------------------------

    #[test]
    fn test_single_bucket_multiple_entities() {
        let mut grid = SpatialGrid::default();
        // All in bucket (1, 1) -> x in [128, 256), y in [128, 256)
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        grid.insert(e1, 130.0, 130.0);
        grid.insert(e2, 200.0, 200.0);
        grid.insert(e3, 255.0, 255.0);

        // Query exactly bucket (1, 1)
        let result = grid.query_rect(128.0, 128.0, 255.0, 255.0);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&e1));
        assert!(result.contains(&e2));
        assert!(result.contains(&e3));
    }

    // ------------------------------------------------------------------
    // Entities along grid edges (first and last columns/rows)
    // ------------------------------------------------------------------

    #[test]
    fn test_entities_along_grid_edges() {
        let mut grid = SpatialGrid::default();

        // Top edge (y=0)
        let top = Entity::from_raw(1);
        grid.insert(top, 2048.0, 0.0);

        // Bottom edge (y near max)
        let bottom = Entity::from_raw(2);
        grid.insert(bottom, 2048.0, 4090.0);

        // Left edge (x=0)
        let left = Entity::from_raw(3);
        grid.insert(left, 0.0, 2048.0);

        // Right edge (x near max)
        let right = Entity::from_raw(4);
        grid.insert(right, 4090.0, 2048.0);

        assert_eq!(grid.entity_count(), 4);

        // Verify each can be found
        let top_result = grid.query_rect(2000.0, 0.0, 2100.0, 10.0);
        assert!(top_result.contains(&top));

        let bottom_result = grid.query_rect(2000.0, 4080.0, 2100.0, 4095.0);
        assert!(bottom_result.contains(&bottom));

        let left_result = grid.query_rect(0.0, 2000.0, 10.0, 2100.0);
        assert!(left_result.contains(&left));

        let right_result = grid.query_rect(4080.0, 2000.0, 4095.0, 2100.0);
        assert!(right_result.contains(&right));
    }

    // ------------------------------------------------------------------
    // Default grid has correct bucket count
    // ------------------------------------------------------------------

    #[test]
    fn test_default_grid_has_correct_bucket_count() {
        let grid = SpatialGrid::default();
        // 33 * 33 = 1089 buckets
        assert_eq!(BUCKETS_X, 33);
        assert_eq!(BUCKETS_Y, 33);
        assert_eq!(TOTAL_BUCKETS, 33 * 33);
        assert_eq!(grid.entity_count(), 0);
    }
}

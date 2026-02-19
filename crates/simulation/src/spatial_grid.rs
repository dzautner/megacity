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
}

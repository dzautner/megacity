//! Flood simulation resources: `FloodGrid` and `FloodState`.
//!
//! `FloodGrid` tracks per-cell flood depth (in feet) while `FloodState` provides
//! aggregate statistics (total flooded cells, cumulative damage, maximum depth).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

use super::damage_curves::FLOOD_DEPTH_THRESHOLD;

// =============================================================================
// FloodGrid resource
// =============================================================================

/// Per-cell flood depth in feet. Only actively maintained during flooding events.
#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct FloodGrid {
    /// Flood depth per cell in feet (GRID_WIDTH * GRID_HEIGHT).
    pub cells: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for FloodGrid {
    fn default() -> Self {
        Self {
            cells: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl FloodGrid {
    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.cells[self.index(x, y)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        let idx = self.index(x, y);
        self.cells[idx] = val;
    }

    /// Returns true if any cell has depth >= `FLOOD_DEPTH_THRESHOLD`.
    pub fn has_flooding(&self) -> bool {
        self.cells.iter().any(|&d| d >= FLOOD_DEPTH_THRESHOLD)
    }

    /// Clear all flood depths to zero.
    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|d| *d = 0.0);
    }
}

// =============================================================================
// FloodState resource
// =============================================================================

/// Aggregate flood statistics for the city.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct FloodState {
    /// Whether a flood event is currently active.
    pub is_flooding: bool,
    /// Number of cells with flood depth >= `FLOOD_DEPTH_THRESHOLD`.
    pub total_flooded_cells: u32,
    /// Cumulative monetary damage from the current flood event.
    pub total_damage: f64,
    /// Maximum flood depth across all cells (feet).
    pub max_depth: f32,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // FloodGrid resource tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_flood_grid_default() {
        let fg = FloodGrid::default();
        assert_eq!(fg.cells.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(fg.width, GRID_WIDTH);
        assert_eq!(fg.height, GRID_HEIGHT);
        assert!(fg.cells.iter().all(|&d| d == 0.0));
    }

    #[test]
    fn test_flood_grid_get_set() {
        let mut fg = FloodGrid::default();
        fg.set(10, 20, 3.5);
        assert!((fg.get(10, 20) - 3.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_flood_grid_index() {
        let fg = FloodGrid::default();
        assert_eq!(fg.index(0, 0), 0);
        assert_eq!(fg.index(1, 0), 1);
        assert_eq!(fg.index(0, 1), GRID_WIDTH);
        assert_eq!(fg.index(5, 3), 3 * GRID_WIDTH + 5);
    }

    #[test]
    fn test_flood_grid_has_flooding_false_when_empty() {
        let fg = FloodGrid::default();
        assert!(!fg.has_flooding());
    }

    #[test]
    fn test_flood_grid_has_flooding_true_when_above_threshold() {
        let mut fg = FloodGrid::default();
        fg.set(50, 50, FLOOD_DEPTH_THRESHOLD);
        assert!(fg.has_flooding());
    }

    #[test]
    fn test_flood_grid_has_flooding_false_when_below_threshold() {
        let mut fg = FloodGrid::default();
        fg.set(50, 50, FLOOD_DEPTH_THRESHOLD - 0.01);
        assert!(!fg.has_flooding());
    }

    #[test]
    fn test_flood_grid_clear() {
        let mut fg = FloodGrid::default();
        fg.set(10, 10, 5.0);
        fg.set(20, 20, 3.0);
        fg.clear();
        assert!(fg.cells.iter().all(|&d| d == 0.0));
    }

    // -------------------------------------------------------------------------
    // FloodState resource tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_flood_state_default() {
        let fs = FloodState::default();
        assert!(!fs.is_flooding);
        assert_eq!(fs.total_flooded_cells, 0);
        assert!((fs.total_damage - 0.0).abs() < f64::EPSILON);
        assert!((fs.max_depth - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_flood_state_clone() {
        let mut fs = FloodState::default();
        fs.is_flooding = true;
        fs.total_flooded_cells = 42;
        fs.total_damage = 123456.0;
        fs.max_depth = 8.5;
        let cloned = fs.clone();
        assert!(cloned.is_flooding);
        assert_eq!(cloned.total_flooded_cells, 42);
        assert!((cloned.total_damage - 123456.0).abs() < f64::EPSILON);
        assert!((cloned.max_depth - 8.5).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // FloodGrid serde round-trip test
    // -------------------------------------------------------------------------

    #[test]
    fn test_flood_grid_serde_roundtrip() {
        let mut fg = FloodGrid::default();
        fg.set(5, 5, 2.0);
        fg.set(100, 100, 7.5);

        let json = serde_json::to_string(&fg).expect("serialize");
        let deserialized: FloodGrid = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.width, GRID_WIDTH);
        assert_eq!(deserialized.height, GRID_HEIGHT);
        assert!((deserialized.get(5, 5) - 2.0).abs() < f32::EPSILON);
        assert!((deserialized.get(100, 100) - 7.5).abs() < f32::EPSILON);
        assert!((deserialized.get(0, 0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_flood_state_serde_roundtrip() {
        let fs = FloodState {
            is_flooding: true,
            total_flooded_cells: 150,
            total_damage: 999_999.99,
            max_depth: 12.3,
        };

        let json = serde_json::to_string(&fs).expect("serialize");
        let deserialized: FloodState = serde_json::from_str(&json).expect("deserialize");

        assert!(deserialized.is_flooding);
        assert_eq!(deserialized.total_flooded_cells, 150);
        assert!((deserialized.total_damage - 999_999.99).abs() < 0.01);
        assert!((deserialized.max_depth - 12.3).abs() < f32::EPSILON);
    }
}

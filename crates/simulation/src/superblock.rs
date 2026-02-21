//! Barcelona Superblock District Policy (TRAF-008).
//!
//! Implements a "superblock" district policy inspired by Barcelona's model.
//! A superblock is a rectangular area of city blocks where interior roads have
//! restricted through-traffic, creating pedestrian-friendly zones.
//!
//! ## Gameplay effects
//!
//! - **Traffic penalty**: Interior roads within a superblock incur a pathfinding
//!   cost multiplier, discouraging through-traffic. Perimeter roads are unaffected.
//! - **Happiness bonus**: Residential zones inside superblocks receive a happiness
//!   bonus from reduced traffic, noise, and improved walkability.
//! - **Land value bonus**: Cells inside superblocks gain a land value boost.
//!
//! ## Design
//!
//! A superblock is defined by its bounding rectangle in grid coordinates.
//! Interior cells are those that are not on the perimeter of the rectangle.
//! The perimeter roads continue to carry normal traffic, while interior roads
//! are penalized for through-traffic.
//!
//! The `SuperblockState` resource tracks all designated superblocks and
//! provides a per-cell lookup grid for O(1) queries.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Pathfinding cost multiplier for interior superblock roads.
/// Higher values make through-traffic less likely to route through superblocks.
pub const SUPERBLOCK_TRAFFIC_PENALTY: f32 = 5.0;

/// Happiness bonus for residential cells inside a superblock.
pub const SUPERBLOCK_HAPPINESS_BONUS: f32 = 6.0;

/// Land value bonus (additive) for cells inside a superblock.
pub const SUPERBLOCK_LAND_VALUE_BONUS: i32 = 10;

/// Minimum superblock dimension in grid cells (must be at least 3 to have an interior).
pub const MIN_SUPERBLOCK_SIZE: usize = 3;

/// Maximum number of superblocks a city can have.
pub const MAX_SUPERBLOCKS: usize = 64;

// =============================================================================
// Superblock definition
// =============================================================================

/// A single superblock defined by its bounding rectangle in grid coordinates.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Superblock {
    /// Top-left corner X (inclusive).
    pub x0: usize,
    /// Top-left corner Y (inclusive).
    pub y0: usize,
    /// Bottom-right corner X (inclusive).
    pub x1: usize,
    /// Bottom-right corner Y (inclusive).
    pub y1: usize,
    /// Optional user-assigned name.
    pub name: String,
}

impl Superblock {
    /// Create a new superblock. Coordinates are automatically sorted so that
    /// (x0,y0) <= (x1,y1).
    pub fn new(x0: usize, y0: usize, x1: usize, y1: usize, name: String) -> Self {
        let (sx0, sx1) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        let (sy0, sy1) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
        Self {
            x0: sx0,
            y0: sy0,
            x1: sx1,
            y1: sy1,
            name,
        }
    }

    /// Width of the superblock in grid cells.
    pub fn width(&self) -> usize {
        self.x1 - self.x0 + 1
    }

    /// Height of the superblock in grid cells.
    pub fn height(&self) -> usize {
        self.y1 - self.y0 + 1
    }

    /// Total area in grid cells.
    pub fn area(&self) -> usize {
        self.width() * self.height()
    }

    /// Whether the superblock meets minimum size requirements.
    pub fn is_valid(&self) -> bool {
        self.width() >= MIN_SUPERBLOCK_SIZE && self.height() >= MIN_SUPERBLOCK_SIZE
    }

    /// Whether a cell is on the perimeter of this superblock.
    pub fn is_perimeter(&self, x: usize, y: usize) -> bool {
        if !self.contains(x, y) {
            return false;
        }
        x == self.x0 || x == self.x1 || y == self.y0 || y == self.y1
    }

    /// Whether a cell is in the interior (not on the perimeter) of this superblock.
    pub fn is_interior(&self, x: usize, y: usize) -> bool {
        self.contains(x, y) && !self.is_perimeter(x, y)
    }

    /// Whether a cell is contained within this superblock's bounds.
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x0 && x <= self.x1 && y >= self.y0 && y <= self.y1
    }
}

// =============================================================================
// Per-cell superblock classification
// =============================================================================

/// Classification of a cell relative to superblocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SuperblockCell {
    /// Cell is not in any superblock.
    #[default]
    None,
    /// Cell is on the perimeter of a superblock (normal traffic).
    Perimeter,
    /// Cell is in the interior of a superblock (restricted traffic).
    Interior,
}

// =============================================================================
// SuperblockState resource
// =============================================================================

/// Resource tracking all designated superblocks and per-cell lookup.
#[derive(Resource, Clone, Encode, Decode)]
pub struct SuperblockState {
    /// All designated superblocks.
    pub superblocks: Vec<Superblock>,
    /// Per-cell classification grid (GRID_WIDTH * GRID_HEIGHT).
    /// Encoded as u8: 0 = None, 1 = Perimeter, 2 = Interior.
    pub cell_grid: Vec<u8>,
    /// Number of interior cells across all superblocks.
    pub total_interior_cells: u32,
    /// Number of superblock cells (interior + perimeter).
    pub total_coverage_cells: u32,
    /// City-wide coverage ratio (superblock cells / total cells).
    pub coverage_ratio: f32,
}

impl Default for SuperblockState {
    fn default() -> Self {
        Self {
            superblocks: Vec::new(),
            cell_grid: vec![0; GRID_WIDTH * GRID_HEIGHT],
            total_interior_cells: 0,
            total_coverage_cells: 0,
            coverage_ratio: 0.0,
        }
    }
}

impl SuperblockState {
    /// Get the classification of a cell.
    #[inline]
    pub fn get_cell(&self, x: usize, y: usize) -> SuperblockCell {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return SuperblockCell::None;
        }
        match self.cell_grid[y * GRID_WIDTH + x] {
            1 => SuperblockCell::Perimeter,
            2 => SuperblockCell::Interior,
            _ => SuperblockCell::None,
        }
    }

    /// Whether a cell is in the interior of a superblock (traffic-restricted).
    #[inline]
    pub fn is_interior(&self, x: usize, y: usize) -> bool {
        self.get_cell(x, y) == SuperblockCell::Interior
    }

    /// Whether a cell is in any superblock (interior or perimeter).
    #[inline]
    pub fn is_in_superblock(&self, x: usize, y: usize) -> bool {
        self.get_cell(x, y) != SuperblockCell::None
    }

    /// Add a superblock. Returns `true` if added successfully.
    pub fn add_superblock(&mut self, superblock: Superblock) -> bool {
        if !superblock.is_valid() {
            return false;
        }
        if self.superblocks.len() >= MAX_SUPERBLOCKS {
            return false;
        }
        self.superblocks.push(superblock);
        self.rebuild_grid();
        true
    }

    /// Remove a superblock by index. Returns `true` if removed.
    pub fn remove_superblock(&mut self, index: usize) -> bool {
        if index >= self.superblocks.len() {
            return false;
        }
        self.superblocks.remove(index);
        self.rebuild_grid();
        true
    }

    /// Rebuild the per-cell classification grid from all superblocks.
    /// Also updates coverage statistics.
    pub fn rebuild_grid(&mut self) {
        self.cell_grid.fill(0);
        self.total_interior_cells = 0;
        self.total_coverage_cells = 0;

        for sb in &self.superblocks {
            for y in sb.y0..=sb.y1.min(GRID_HEIGHT - 1) {
                for x in sb.x0..=sb.x1.min(GRID_WIDTH - 1) {
                    let idx = y * GRID_WIDTH + x;
                    let is_perimeter = x == sb.x0 || x == sb.x1 || y == sb.y0 || y == sb.y1;
                    // Interior takes priority if overlapping superblocks
                    // (a cell on the perimeter of one but interior of another
                    // is effectively interior)
                    if is_perimeter {
                        if self.cell_grid[idx] == 0 {
                            self.cell_grid[idx] = 1; // Perimeter
                        }
                    } else {
                        self.cell_grid[idx] = 2; // Interior
                    }
                }
            }
        }

        // Count cells
        for &v in &self.cell_grid {
            match v {
                1 => self.total_coverage_cells += 1,
                2 => {
                    self.total_interior_cells += 1;
                    self.total_coverage_cells += 1;
                }
                _ => {}
            }
        }

        let total_cells = (GRID_WIDTH * GRID_HEIGHT) as f32;
        self.coverage_ratio = self.total_coverage_cells as f32 / total_cells;
    }

    /// Get the traffic cost multiplier for a cell.
    /// Interior cells return `SUPERBLOCK_TRAFFIC_PENALTY`, others return 1.0.
    #[inline]
    pub fn traffic_multiplier(&self, x: usize, y: usize) -> f32 {
        if self.is_interior(x, y) {
            SUPERBLOCK_TRAFFIC_PENALTY
        } else {
            1.0
        }
    }
}

// =============================================================================
// System: update superblock statistics
// =============================================================================

/// System that periodically recomputes superblock coverage statistics.
/// The grid itself is rebuilt on add/remove, but statistics are recalculated
/// on the slow tick in case the grid dimensions or superblock definitions
/// change through save/load.
pub fn update_superblock_stats(timer: Res<SlowTickTimer>, mut state: ResMut<SuperblockState>) {
    if !timer.should_run() {
        return;
    }

    // Rebuild grid (idempotent — ensures consistency after save/load)
    state.rebuild_grid();
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for SuperblockState {
    const SAVE_KEY: &'static str = "superblock_state";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.superblocks.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let mut state: Self = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        state.rebuild_grid();
        state
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct SuperblockPlugin;

impl Plugin for SuperblockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SuperblockState>().add_systems(
            FixedUpdate,
            update_superblock_stats
                .after(crate::districts::district_stats)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<SuperblockState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Superblock geometry tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_superblock_new_sorts_coordinates() {
        let sb = Superblock::new(10, 15, 5, 8, "test".to_string());
        assert_eq!(sb.x0, 5);
        assert_eq!(sb.y0, 8);
        assert_eq!(sb.x1, 10);
        assert_eq!(sb.y1, 15);
    }

    #[test]
    fn test_superblock_dimensions() {
        let sb = Superblock::new(10, 10, 14, 14, "test".to_string());
        assert_eq!(sb.width(), 5);
        assert_eq!(sb.height(), 5);
        assert_eq!(sb.area(), 25);
    }

    #[test]
    fn test_superblock_valid() {
        // 3x3 is minimum valid size
        let sb = Superblock::new(10, 10, 12, 12, "test".to_string());
        assert!(sb.is_valid());
    }

    #[test]
    fn test_superblock_too_small() {
        // 2x3 is too narrow
        let sb = Superblock::new(10, 10, 11, 12, "test".to_string());
        assert!(!sb.is_valid());
    }

    #[test]
    fn test_superblock_contains() {
        let sb = Superblock::new(10, 10, 14, 14, "test".to_string());
        assert!(sb.contains(10, 10));
        assert!(sb.contains(12, 12));
        assert!(sb.contains(14, 14));
        assert!(!sb.contains(9, 10));
        assert!(!sb.contains(10, 15));
    }

    #[test]
    fn test_superblock_perimeter() {
        let sb = Superblock::new(10, 10, 14, 14, "test".to_string());
        // Corners are perimeter
        assert!(sb.is_perimeter(10, 10));
        assert!(sb.is_perimeter(14, 14));
        // Edges are perimeter
        assert!(sb.is_perimeter(12, 10));
        assert!(sb.is_perimeter(10, 12));
        // Center is not perimeter
        assert!(!sb.is_perimeter(12, 12));
        // Outside is not perimeter
        assert!(!sb.is_perimeter(9, 10));
    }

    #[test]
    fn test_superblock_interior() {
        let sb = Superblock::new(10, 10, 14, 14, "test".to_string());
        // Center cells are interior
        assert!(sb.is_interior(11, 11));
        assert!(sb.is_interior(12, 12));
        assert!(sb.is_interior(13, 13));
        // Edges are not interior
        assert!(!sb.is_interior(10, 12));
        assert!(!sb.is_interior(14, 12));
        assert!(!sb.is_interior(12, 10));
        assert!(!sb.is_interior(12, 14));
    }

    #[test]
    fn test_superblock_3x3_has_one_interior() {
        let sb = Superblock::new(10, 10, 12, 12, "test".to_string());
        // Only center cell (11,11) is interior
        assert!(sb.is_interior(11, 11));
        // All edges/corners are perimeter
        for x in 10..=12 {
            for y in 10..=12 {
                if x == 11 && y == 11 {
                    continue;
                }
                assert!(sb.is_perimeter(x, y), "({x},{y}) should be perimeter");
            }
        }
    }

    // -------------------------------------------------------------------------
    // SuperblockState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_default_empty() {
        let state = SuperblockState::default();
        assert!(state.superblocks.is_empty());
        assert_eq!(state.total_interior_cells, 0);
        assert_eq!(state.total_coverage_cells, 0);
        assert!(state.coverage_ratio.abs() < f32::EPSILON);
    }

    #[test]
    fn test_state_add_superblock() {
        let mut state = SuperblockState::default();
        let sb = Superblock::new(10, 10, 14, 14, "test".to_string());
        assert!(state.add_superblock(sb));
        assert_eq!(state.superblocks.len(), 1);
        assert!(state.total_interior_cells > 0);
        assert!(state.total_coverage_cells > 0);
    }

    #[test]
    fn test_state_reject_too_small() {
        let mut state = SuperblockState::default();
        let sb = Superblock::new(10, 10, 11, 11, "tiny".to_string());
        assert!(!state.add_superblock(sb));
        assert!(state.superblocks.is_empty());
    }

    #[test]
    fn test_state_remove_superblock() {
        let mut state = SuperblockState::default();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "test".to_string()));
        assert!(state.remove_superblock(0));
        assert!(state.superblocks.is_empty());
        assert_eq!(state.total_interior_cells, 0);
    }

    #[test]
    fn test_state_remove_invalid_index() {
        let mut state = SuperblockState::default();
        assert!(!state.remove_superblock(0));
    }

    #[test]
    fn test_state_cell_classification() {
        let mut state = SuperblockState::default();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "test".to_string()));

        // Interior
        assert_eq!(state.get_cell(12, 12), SuperblockCell::Interior);
        assert!(state.is_interior(12, 12));
        assert!(state.is_in_superblock(12, 12));

        // Perimeter
        assert_eq!(state.get_cell(10, 10), SuperblockCell::Perimeter);
        assert!(!state.is_interior(10, 10));
        assert!(state.is_in_superblock(10, 10));

        // Outside
        assert_eq!(state.get_cell(5, 5), SuperblockCell::None);
        assert!(!state.is_interior(5, 5));
        assert!(!state.is_in_superblock(5, 5));
    }

    #[test]
    fn test_state_traffic_multiplier() {
        let mut state = SuperblockState::default();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "test".to_string()));

        // Interior gets penalty
        assert!(
            (state.traffic_multiplier(12, 12) - SUPERBLOCK_TRAFFIC_PENALTY).abs() < f32::EPSILON
        );

        // Perimeter is normal
        assert!((state.traffic_multiplier(10, 10) - 1.0).abs() < f32::EPSILON);

        // Outside is normal
        assert!((state.traffic_multiplier(5, 5) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_state_coverage_stats() {
        let mut state = SuperblockState::default();
        // 5x5 superblock = 25 total cells, 16 perimeter, 9 interior
        state.add_superblock(Superblock::new(10, 10, 14, 14, "test".to_string()));

        assert_eq!(state.total_interior_cells, 9);
        assert_eq!(state.total_coverage_cells, 25);

        let expected_ratio = 25.0 / (GRID_WIDTH * GRID_HEIGHT) as f32;
        assert!((state.coverage_ratio - expected_ratio).abs() < 1e-6);
    }

    #[test]
    fn test_state_overlapping_superblocks() {
        let mut state = SuperblockState::default();
        // Two overlapping 5x5 superblocks
        state.add_superblock(Superblock::new(10, 10, 14, 14, "A".to_string()));
        state.add_superblock(Superblock::new(12, 12, 16, 16, "B".to_string()));

        // Cell (13,13) is interior of both
        assert!(state.is_interior(13, 13));

        // Cell (12,12) is perimeter of A but interior of B => should be interior
        assert!(state.is_interior(12, 12));
        // Actually (12,12) is interior of A (not on edge 10/14) and perimeter of B (on edge 12)
        // Let me trace: A: x0=10,y0=10,x1=14,y1=14. For (12,12): not on edges of A → interior of A.
        // B: x0=12,y0=12,x1=16,y1=16. For (12,12): on edges of B → perimeter of B.
        // Since A is processed first, it sets interior (2). Then B processes it as perimeter,
        // but only writes perimeter if cell_grid[idx] == 0. So (12,12) stays interior. Correct!
    }

    #[test]
    fn test_state_max_superblocks() {
        let mut state = SuperblockState::default();
        for i in 0..MAX_SUPERBLOCKS {
            let x = (i * 10) % 200;
            let y = (i / 20) * 10;
            state.add_superblock(Superblock::new(x, y, x + 4, y + 4, format!("sb{i}")));
        }
        assert_eq!(state.superblocks.len(), MAX_SUPERBLOCKS);

        // Adding one more should fail
        assert!(!state.add_superblock(Superblock::new(200, 200, 204, 204, "overflow".to_string())));
    }

    #[test]
    fn test_state_out_of_bounds_cell() {
        let state = SuperblockState::default();
        assert_eq!(state.get_cell(999, 999), SuperblockCell::None);
        assert!(!state.is_interior(999, 999));
        assert!((state.traffic_multiplier(999, 999) - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = SuperblockState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_nonempty() {
        use crate::Saveable;
        let mut state = SuperblockState::default();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "test".to_string()));
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = SuperblockState::default();
        state.add_superblock(Superblock::new(10, 10, 14, 14, "A".to_string()));
        state.add_superblock(Superblock::new(50, 50, 55, 55, "B".to_string()));

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = SuperblockState::load_from_bytes(&bytes);

        assert_eq!(restored.superblocks.len(), 2);
        assert_eq!(restored.superblocks[0].name, "A");
        assert_eq!(restored.superblocks[1].name, "B");
        assert!(restored.is_interior(12, 12));
        assert!(restored.total_interior_cells > 0);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(SuperblockState::SAVE_KEY, "superblock_state");
    }

    // -------------------------------------------------------------------------
    // Constants verification
    // -------------------------------------------------------------------------

    #[test]
    fn test_traffic_penalty_positive() {
        assert!(SUPERBLOCK_TRAFFIC_PENALTY > 1.0);
    }

    #[test]
    fn test_happiness_bonus_positive() {
        assert!(SUPERBLOCK_HAPPINESS_BONUS > 0.0);
    }

    #[test]
    fn test_land_value_bonus_positive() {
        assert!(SUPERBLOCK_LAND_VALUE_BONUS > 0);
    }

    #[test]
    fn test_min_size_at_least_3() {
        assert!(MIN_SUPERBLOCK_SIZE >= 3);
    }
}

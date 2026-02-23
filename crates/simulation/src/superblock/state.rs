//! `SuperblockState` resource: tracks all designated superblocks and per-cell lookup.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

use super::constants::{MAX_SUPERBLOCKS, SUPERBLOCK_TRAFFIC_PENALTY};
use super::types::{Superblock, SuperblockCell};

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

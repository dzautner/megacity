//! Type definitions for superblocks: geometry and cell classification.

use bitcode::{Decode, Encode};

use super::constants::MIN_SUPERBLOCK_SIZE;

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

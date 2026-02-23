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

pub mod constants;
pub mod state;
pub mod systems;
mod tests;
pub mod types;

pub use constants::{
    MAX_SUPERBLOCKS, MIN_SUPERBLOCK_SIZE, SUPERBLOCK_HAPPINESS_BONUS, SUPERBLOCK_LAND_VALUE_BONUS,
    SUPERBLOCK_TRAFFIC_PENALTY,
};
pub use state::SuperblockState;
pub use systems::{update_superblock_stats, SuperblockPlugin};
pub use types::{Superblock, SuperblockCell};

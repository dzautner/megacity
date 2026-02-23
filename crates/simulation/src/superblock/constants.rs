//! Constants for the Barcelona Superblock District Policy (TRAF-008).

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

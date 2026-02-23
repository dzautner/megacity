// =============================================================================
// Constants
// =============================================================================

/// How often (in ticks) the forest fire system runs.
pub(crate) const FIRE_UPDATE_INTERVAL: u64 = 10;

/// Lightning strike chance per cell during storms (per update cycle).
/// Applied only to cells with trees. Very low: checked against hash % 100_000.
pub(crate) const LIGHTNING_CHANCE_PER_CELL: u64 = 2; // 2 in 100_000

/// Chance for fire to spread from a burning building (FireGrid) to adjacent forest cells.
/// Checked against hash % 1000.
pub(crate) const BUILDING_FIRE_SPREAD_THRESHOLD: u64 = 50; // 5% per neighbor

/// Chance for spontaneous ignition near industrial zones in hot weather.
/// Checked against hash % 100_000.
pub(crate) const INDUSTRIAL_IGNITION_THRESHOLD: u64 = 5; // 5 in 100_000

/// Base fire intensity when a cell first ignites.
pub(crate) const INITIAL_INTENSITY: u8 = 30;

/// How much intensity decreases per update tick (natural burnout).
pub(crate) const BURNOUT_RATE: u8 = 2;

/// How much rain reduces fire intensity per update tick.
pub(crate) const RAIN_REDUCTION: u8 = 8;

/// How much storm reduces fire intensity per update tick.
pub(crate) const STORM_REDUCTION: u8 = 15;

/// Threshold above which a forest fire can ignite a building.
pub(crate) const BUILDING_IGNITION_THRESHOLD: u8 = 100;

/// Land value penalty applied per burning cell in radius 3.
pub(crate) const LAND_VALUE_PENALTY: u8 = 5;

// =============================================================================
// Deterministic pseudo-random helper
// =============================================================================

/// Deterministic hash for a given tick and cell index.
/// Returns a value in [0, modulus).
#[inline]
pub(crate) fn fire_hash(tick: u64, cell_index: usize, salt: u64) -> u64 {
    tick.wrapping_mul(7919)
        .wrapping_add(cell_index as u64)
        .wrapping_add(salt)
        .wrapping_mul(2654435761)
}

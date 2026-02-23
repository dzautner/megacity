// ---------------------------------------------------------------------------
// Tick-based pseudo-random number generator
// ---------------------------------------------------------------------------

/// Simple hash-based pseudo-random from a tick value.
/// Returns a u32 suitable for modulo operations.
pub(crate) fn tick_pseudo_random(tick: u64) -> u32 {
    // Mix bits using a simple multiplicative hash (splitmix-inspired)
    let mut x = tick.wrapping_mul(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x = x ^ (x >> 31);
    x as u32
}

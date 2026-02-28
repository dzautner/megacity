//! Lightweight procedural terrain generation from a seed.
//!
//! This module provides a fast, dependency-free terrain generator suitable for
//! agent mode's `NewGame` command. It uses a simple hash-based pseudo-noise
//! approach to create coastline water bodies along one or two edges of the
//! 256×256 grid.
//!
//! For the full-featured terrain pipeline (fBm + hydraulic erosion + rivers +
//! biomes), see [`crate::terrain_generation`].

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};

// ---------------------------------------------------------------------------
// Hash-based pseudo-noise (no external crate dependencies)
// ---------------------------------------------------------------------------

/// Splitmix64-style hash for deterministic per-cell noise.
///
/// Produces a well-distributed u64 from `(seed, x, y)`. Same inputs always
/// yield the same output, so terrain is fully reproducible from a seed.
#[inline]
fn hash_cell(seed: u64, x: usize, y: usize) -> u64 {
    let mut h = seed;
    h = h.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(x as u64);
    h = h.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(y as u64);
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^= h >> 31;
    h
}

/// Map a hash value to a float in [0, 1].
#[inline]
fn hash_to_f32(h: u64) -> f32 {
    (h & 0x00FF_FFFF) as f32 / 0x00FF_FFFF as f32
}

// ---------------------------------------------------------------------------
// Coastline generation
// ---------------------------------------------------------------------------

/// Which edge(s) of the grid get a coastline.
#[derive(Debug, Clone, Copy)]
enum CoastEdge {
    North,
    South,
    East,
    West,
}

/// Select the primary coastline edge from the seed.
fn primary_edge(seed: u64) -> CoastEdge {
    match seed % 4 {
        0 => CoastEdge::North,
        1 => CoastEdge::South,
        2 => CoastEdge::East,
        _ => CoastEdge::West,
    }
}

/// Whether a second coastline edge should be added (corner peninsula effect).
/// Roughly 40% of seeds produce a second edge.
fn secondary_edge(seed: u64) -> Option<CoastEdge> {
    let h = hash_cell(seed, 9999, 9999);
    if h % 5 < 2 {
        // Pick the clockwise neighbor of the primary edge
        Some(match primary_edge(seed) {
            CoastEdge::North => CoastEdge::East,
            CoastEdge::East => CoastEdge::South,
            CoastEdge::South => CoastEdge::West,
            CoastEdge::West => CoastEdge::North,
        })
    } else {
        None
    }
}

/// Compute the base depth for a coastline (20–40 cells) from the seed.
fn base_depth(seed: u64) -> f32 {
    20.0 + hash_to_f32(hash_cell(seed, 12345, 67890)) * 20.0
}

/// For a given cell, compute whether it should be water based on its distance
/// to the coast edge and per-cell noise.
fn is_water_for_edge(
    edge: CoastEdge,
    x: usize,
    y: usize,
    depth: f32,
    noise_amplitude: f32,
    seed: u64,
) -> bool {
    // Distance from the edge (0 = at edge, increases inward)
    let dist = match edge {
        CoastEdge::North => y as f32,
        CoastEdge::South => (GRID_HEIGHT - 1 - y) as f32,
        CoastEdge::West => x as f32,
        CoastEdge::East => (GRID_WIDTH - 1 - x) as f32,
    };

    // Per-cell noise offset to make the boundary irregular
    let noise = (hash_to_f32(hash_cell(seed, x, y)) - 0.5) * noise_amplitude;

    // Smooth multi-scale noise for more natural coastlines: add a second
    // octave at half frequency / half amplitude by hashing at half resolution.
    let coarse_noise =
        (hash_to_f32(hash_cell(seed.wrapping_add(7), x / 4, y / 4)) - 0.5) * noise_amplitude;

    let threshold = depth + noise + coarse_noise;

    dist < threshold
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate procedural terrain features on the grid from the given seed.
///
/// Creates coastline water bodies along one or two edges of the grid. The
/// boundary is irregular (20–40 cells deep with noise) for a natural look.
///
/// This function is deterministic: same seed produces identical terrain.
///
/// # Arguments
/// * `grid` – mutable reference to the `WorldGrid` to populate
/// * `seed` – u64 seed controlling edge selection, depth, and noise
pub fn generate_terrain(grid: &mut WorldGrid, seed: u64) {
    let depth = base_depth(seed);
    let noise_amplitude = depth * 0.6; // noise is ±30% of depth

    let edge1 = primary_edge(seed);
    let edge2 = secondary_edge(seed);

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let water1 = is_water_for_edge(edge1, x, y, depth, noise_amplitude, seed);
            let water2 = edge2
                .map(|e| is_water_for_edge(e, x, y, depth, noise_amplitude, seed.wrapping_add(1)))
                .unwrap_or(false);

            if water1 || water2 {
                let cell = grid.get_mut(x, y);
                cell.cell_type = CellType::Water;
                // Set a low elevation for water cells so rendering treats them
                // as below the water threshold.
                cell.elevation = 0.15;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_cell_deterministic() {
        assert_eq!(hash_cell(42, 10, 20), hash_cell(42, 10, 20));
    }

    #[test]
    fn test_hash_cell_varies_with_input() {
        assert_ne!(hash_cell(42, 10, 20), hash_cell(42, 10, 21));
        assert_ne!(hash_cell(42, 10, 20), hash_cell(43, 10, 20));
    }

    #[test]
    fn test_hash_to_f32_range() {
        for seed in 0..100 {
            let v = hash_to_f32(hash_cell(seed, 0, 0));
            assert!((0.0..=1.0).contains(&v), "hash_to_f32 out of range: {v}");
        }
    }

    #[test]
    fn test_base_depth_range() {
        for seed in 0..100 {
            let d = base_depth(seed);
            assert!(d >= 20.0, "depth too small: {d}");
            assert!(d <= 40.0, "depth too large: {d}");
        }
    }

    #[test]
    fn test_primary_edge_coverage() {
        // All four edges should appear across seeds 0..4
        let edges: Vec<_> = (0..4).map(primary_edge).collect();
        assert!(matches!(edges[0], CoastEdge::North));
        assert!(matches!(edges[1], CoastEdge::South));
        assert!(matches!(edges[2], CoastEdge::East));
        assert!(matches!(edges[3], CoastEdge::West));
    }
}

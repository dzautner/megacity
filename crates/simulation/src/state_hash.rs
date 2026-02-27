//! Deterministic state hashing for replay verification.
//!
//! Computes a 64-bit hash of key simulation state every tick, stored in the
//! `StateHash` resource. The hash is computed in a fixed order over:
//!
//! 1. Tick counter
//! 2. Treasury (f64 → bits)
//! 3. Population count
//! 4. Average happiness (f32 → bits)
//! 5. SimRng internal state (word_pos, stream)
//! 6. Grid cells in row-major order (road_type, zone_type)
//!
//! All data is iterated in deterministic order (no HashMap, no entity IDs).
//! Float values are converted to their bit representation before hashing.

use std::hash::{Hash, Hasher};

use bevy::prelude::*;

use crate::economy::CityBudget;
use crate::grid::WorldGrid;
use crate::sim_rng::SimRng;
use crate::stats::CityStats;
use crate::SimulationSet;
use crate::TickCounter;

/// Stores the deterministic hash computed at the end of each simulation tick.
#[derive(Resource, Default, Clone, Debug)]
pub struct StateHash {
    /// The tick at which this hash was computed.
    pub tick: u64,
    /// The 64-bit FNV-1a hash of simulation state.
    pub hash: u64,
}

// ---------------------------------------------------------------------------
// FNV-1a hasher (deterministic, no random seed)
// ---------------------------------------------------------------------------

/// A simple FNV-1a hasher that produces deterministic output regardless of
/// platform or Rust version. Unlike `DefaultHasher`, this is not randomized.
struct Fnv1aHasher {
    state: u64,
}

impl Fnv1aHasher {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001B3;

    fn new() -> Self {
        Self {
            state: Self::FNV_OFFSET_BASIS,
        }
    }
}

impl Hasher for Fnv1aHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state ^= byte as u64;
            self.state = self.state.wrapping_mul(Self::FNV_PRIME);
        }
    }
}

// ---------------------------------------------------------------------------
// Public convenience function
// ---------------------------------------------------------------------------

/// Compute a deterministic hash of the current simulation state.
///
/// This function can be called from tests or replay verification without
/// needing the ECS system to have run.
pub fn compute_state_hash(
    tick: u64,
    treasury: f64,
    population: u32,
    average_happiness: f32,
    rng_word_pos: u128,
    rng_stream: u64,
    grid: &WorldGrid,
) -> u64 {
    let mut hasher = Fnv1aHasher::new();

    // 1. Tick counter
    tick.hash(&mut hasher);

    // 2. Treasury (f64 → deterministic bits)
    treasury.to_bits().hash(&mut hasher);

    // 3. Population count
    population.hash(&mut hasher);

    // 4. Average happiness (f32 → deterministic bits)
    average_happiness.to_bits().hash(&mut hasher);

    // 5. SimRng state
    rng_word_pos.hash(&mut hasher);
    rng_stream.hash(&mut hasher);

    // 6. Grid cells in row-major order
    for cell in &grid.cells {
        cell.road_type.hash(&mut hasher);
        cell.zone.hash(&mut hasher);
    }

    hasher.finish()
}

// ---------------------------------------------------------------------------
// ECS system
// ---------------------------------------------------------------------------

fn update_state_hash(
    tick: Res<TickCounter>,
    budget: Res<CityBudget>,
    stats: Res<CityStats>,
    sim_rng: Res<SimRng>,
    grid: Res<WorldGrid>,
    mut state_hash: ResMut<StateHash>,
) {
    let hash = compute_state_hash(
        tick.0,
        budget.treasury,
        stats.population,
        stats.average_happiness,
        sim_rng.0.get_word_pos(),
        sim_rng.0.get_stream(),
        &grid,
    );

    state_hash.tick = tick.0;
    state_hash.hash = hash;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct StateHashPlugin;

impl Plugin for StateHashPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StateHash>()
            .add_systems(
                FixedUpdate,
                update_state_hash.in_set(SimulationSet::PostSim),
            );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::WorldGrid;

    #[test]
    fn test_fnv1a_deterministic() {
        let mut h1 = Fnv1aHasher::new();
        let mut h2 = Fnv1aHasher::new();
        42u64.hash(&mut h1);
        42u64.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_fnv1a_different_inputs_differ() {
        let mut h1 = Fnv1aHasher::new();
        let mut h2 = Fnv1aHasher::new();
        1u64.hash(&mut h1);
        2u64.hash(&mut h2);
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_compute_state_hash_deterministic() {
        let grid = WorldGrid::new(4, 4);
        let h1 = compute_state_hash(10, 50000.0, 100, 75.0, 0, 0, &grid);
        let h2 = compute_state_hash(10, 50000.0, 100, 75.0, 0, 0, &grid);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_compute_state_hash_differs_on_treasury() {
        let grid = WorldGrid::new(4, 4);
        let h1 = compute_state_hash(10, 50000.0, 100, 75.0, 0, 0, &grid);
        let h2 = compute_state_hash(10, 50001.0, 100, 75.0, 0, 0, &grid);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_compute_state_hash_differs_on_tick() {
        let grid = WorldGrid::new(4, 4);
        let h1 = compute_state_hash(10, 50000.0, 100, 75.0, 0, 0, &grid);
        let h2 = compute_state_hash(11, 50000.0, 100, 75.0, 0, 0, &grid);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_compute_state_hash_differs_on_population() {
        let grid = WorldGrid::new(4, 4);
        let h1 = compute_state_hash(10, 50000.0, 100, 75.0, 0, 0, &grid);
        let h2 = compute_state_hash(10, 50000.0, 101, 75.0, 0, 0, &grid);
        assert_ne!(h1, h2);
    }
}

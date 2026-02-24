//! Deterministic simulation RNG resource.
//!
//! Wraps `ChaCha8Rng` for cross-platform deterministic randomness.
//! All simulation systems should use `ResMut<SimRng>` instead of
//! `rand::thread_rng()` so that identical seeds produce identical
//! simulation output.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Default seed used when no explicit seed is provided.
const DEFAULT_SEED: u64 = 42;

// ---------------------------------------------------------------------------
// Serializable snapshot of ChaCha8Rng state
// ---------------------------------------------------------------------------

/// Captures the full internal state of a `ChaCha8Rng` so it can be
/// round-tripped through bitcode.
#[derive(Encode, Decode)]
struct RngSnapshot {
    seed: [u8; 32],
    word_pos: u128,
    stream: u64,
}

impl RngSnapshot {
    fn from_rng(rng: &ChaCha8Rng) -> Self {
        Self {
            seed: rng.get_seed(),
            word_pos: rng.get_word_pos(),
            stream: rng.get_stream(),
        }
    }

    fn to_rng(&self) -> ChaCha8Rng {
        let mut rng = ChaCha8Rng::from_seed(self.seed);
        rng.set_stream(self.stream);
        rng.set_word_pos(self.word_pos);
        rng
    }
}

// ---------------------------------------------------------------------------
// SimRng resource
// ---------------------------------------------------------------------------

/// Deterministic RNG resource for all simulation randomness.
///
/// Systems that need randomness take `ResMut<SimRng>` and use `rng.0`
/// (which is a `ChaCha8Rng` implementing `rand::Rng`).
#[derive(Resource)]
pub struct SimRng(pub ChaCha8Rng);

impl Default for SimRng {
    fn default() -> Self {
        Self(ChaCha8Rng::seed_from_u64(DEFAULT_SEED))
    }
}

impl SimRng {
    /// Create a new `SimRng` seeded from the given `u64` value.
    pub fn from_seed_u64(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl crate::Saveable for SimRng {
    const SAVE_KEY: &'static str = "sim_rng";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        let snapshot = RngSnapshot::from_rng(&self.0);
        Some(bitcode::encode(&snapshot))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        match bitcode::decode::<RngSnapshot>(bytes) {
            Ok(snapshot) => Self(snapshot.to_rng()),
            Err(e) => {
                bevy::log::warn!(
                    "SimRng: failed to decode save data, falling back to default: {}",
                    e
                );
                Self::default()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SimRngPlugin;

impl Plugin for SimRngPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimRng>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<SimRng>();
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_default_is_deterministic() {
        let mut a = SimRng::default();
        let mut b = SimRng::default();
        let vals_a: Vec<f32> = (0..10).map(|_| a.0.gen::<f32>()).collect();
        let vals_b: Vec<f32> = (0..10).map(|_| b.0.gen::<f32>()).collect();
        assert_eq!(vals_a, vals_b);
    }

    #[test]
    fn test_from_seed_u64_deterministic() {
        let mut a = SimRng::from_seed_u64(12345);
        let mut b = SimRng::from_seed_u64(12345);
        let vals_a: Vec<u32> = (0..20).map(|_| a.0.gen_range(0..1000)).collect();
        let vals_b: Vec<u32> = (0..20).map(|_| b.0.gen_range(0..1000)).collect();
        assert_eq!(vals_a, vals_b);
    }

    #[test]
    fn test_different_seeds_differ() {
        let mut a = SimRng::from_seed_u64(1);
        let mut b = SimRng::from_seed_u64(2);
        let vals_a: Vec<f32> = (0..10).map(|_| a.0.gen::<f32>()).collect();
        let vals_b: Vec<f32> = (0..10).map(|_| b.0.gen::<f32>()).collect();
        assert_ne!(vals_a, vals_b);
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut rng = SimRng::from_seed_u64(999);
        // Advance the RNG a bit
        for _ in 0..100 {
            rng.0.gen::<f64>();
        }

        // Save
        let bytes = rng.save_to_bytes().expect("save should produce bytes");

        // Load
        let mut restored = SimRng::load_from_bytes(&bytes);

        // Both should produce identical output from this point
        let vals_orig: Vec<f32> = (0..50).map(|_| rng.0.gen::<f32>()).collect();
        let vals_rest: Vec<f32> = (0..50).map(|_| restored.0.gen::<f32>()).collect();
        assert_eq!(vals_orig, vals_rest);
    }
}

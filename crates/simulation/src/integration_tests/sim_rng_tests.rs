//! Integration tests for SimRng deterministic randomness.

use crate::sim_rng::SimRng;
use crate::Saveable;
use rand::Rng;

#[test]
fn test_sim_rng_same_seed_produces_same_output() {
    let mut rng_a = SimRng::from_seed_u64(42);
    let mut rng_b = SimRng::from_seed_u64(42);

    let results_a: Vec<f32> = (0..100).map(|_| rng_a.0.gen::<f32>()).collect();
    let results_b: Vec<f32> = (0..100).map(|_| rng_b.0.gen::<f32>()).collect();

    assert_eq!(
        results_a, results_b,
        "Same seed must produce identical output"
    );
}

#[test]
fn test_sim_rng_save_load_preserves_state() {
    let mut rng = SimRng::from_seed_u64(7777);

    // Advance the RNG so it's not at the initial position
    for _ in 0..200 {
        rng.0.gen::<u64>();
    }

    // Save
    let bytes = rng.save_to_bytes().expect("save must produce bytes");

    // Load into a new instance
    let mut restored = SimRng::load_from_bytes(&bytes);

    // Both should produce identical output from this point
    let vals_orig: Vec<u32> = (0..100).map(|_| rng.0.gen_range(0..10000)).collect();
    let vals_rest: Vec<u32> = (0..100).map(|_| restored.0.gen_range(0..10000)).collect();

    assert_eq!(
        vals_orig, vals_rest,
        "Save/load must preserve exact RNG state"
    );
}

#[test]
fn test_sim_rng_default_is_consistent() {
    let mut a = SimRng::default();
    let mut b = SimRng::default();

    let val_a = a.0.gen::<f64>();
    let val_b = b.0.gen::<f64>();

    assert_eq!(
        val_a, val_b,
        "Default SimRng must produce consistent results"
    );
}

#[test]
fn test_sim_rng_gen_range_deterministic() {
    let mut rng_a = SimRng::from_seed_u64(123);
    let mut rng_b = SimRng::from_seed_u64(123);

    let ranges_a: Vec<i32> = (0..50).map(|_| rng_a.0.gen_range(-100..100)).collect();
    let ranges_b: Vec<i32> = (0..50).map(|_| rng_b.0.gen_range(-100..100)).collect();

    assert_eq!(
        ranges_a, ranges_b,
        "gen_range must be deterministic with same seed"
    );
}

#[test]
fn test_sim_rng_different_seeds_diverge() {
    let mut rng_a = SimRng::from_seed_u64(1);
    let mut rng_b = SimRng::from_seed_u64(2);

    let vals_a: Vec<f32> = (0..20).map(|_| rng_a.0.gen::<f32>()).collect();
    let vals_b: Vec<f32> = (0..20).map(|_| rng_b.0.gen::<f32>()).collect();

    assert_ne!(
        vals_a, vals_b,
        "Different seeds must produce different output"
    );
}

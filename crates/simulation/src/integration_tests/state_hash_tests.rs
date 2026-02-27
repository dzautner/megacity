//! Integration tests for deterministic state hashing (#1883).

use crate::economy::CityBudget;
use crate::state_hash::StateHash;
use crate::test_harness::TestCity;

#[test]
fn test_identical_cities_produce_same_hash() {
    let mut city_a = TestCity::new();
    let mut city_b = TestCity::new();

    // Advance both by the same number of ticks
    city_a.tick(100);
    city_b.tick(100);

    let hash_a = city_a.resource::<StateHash>().clone();
    let hash_b = city_b.resource::<StateHash>().clone();

    assert_eq!(hash_a.tick, hash_b.tick, "Tick counters should match");
    assert_eq!(
        hash_a.hash, hash_b.hash,
        "Identical cities after same ticks must produce identical hashes"
    );
}

#[test]
fn test_state_hash_changes_after_treasury_modification() {
    let mut city = TestCity::new();

    // Advance to get a baseline hash
    city.tick(50);
    let hash_before = city.resource::<StateHash>().hash;

    // Modify treasury
    {
        let world = city.world_mut();
        world.resource_mut::<CityBudget>().treasury += 999_999.0;
    }

    // Advance one more tick so the hash recomputes
    city.tick(1);
    let hash_after = city.resource::<StateHash>().hash;

    assert_ne!(
        hash_before, hash_after,
        "Hash must change when treasury is modified"
    );
}

#[test]
fn test_state_hash_is_nonzero_after_ticks() {
    let mut city = TestCity::new();
    city.tick(10);

    let hash = city.resource::<StateHash>();
    assert_eq!(hash.tick, 10);
    // The hash should not be zero (extremely unlikely with FNV-1a)
    assert_ne!(hash.hash, 0, "Hash should be non-zero after simulation");
}

#[test]
fn test_state_hash_differs_between_ticks() {
    let mut city = TestCity::new();

    city.tick(10);
    let hash_at_10 = city.resource::<StateHash>().hash;

    city.tick(1);
    let hash_at_11 = city.resource::<StateHash>().hash;

    // Since the tick counter itself is hashed, consecutive ticks should differ
    assert_ne!(
        hash_at_10, hash_at_11,
        "Hash should differ between consecutive ticks"
    );
}

# TEST-020: Replace thread_rng with Seeded SimRng

## Priority: T2 (Depth)
## Effort: Medium (2-3 days)
## Source: testing_strategy.md -- Section 4.2: Sources of Non-Determinism

## Description
Replace all `rand::thread_rng()` usage in simulation crate with a deterministic `SimRng` resource seeded from game state. Uses `ChaCha8Rng` for cross-platform determinism.

## Acceptance Criteria
- [ ] `SimRng(ChaCha8Rng)` resource added
- [ ] All systems use `ResMut<SimRng>` instead of `thread_rng()`
- [ ] Seed configurable (from save file or new game settings)
- [ ] Same seed produces same simulation output

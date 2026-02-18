# INFRA-011: Seed-Based Map Generation with Playability Guarantees
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-001, INFRA-006, INFRA-008
**Source:** procedural_terrain.md, Section 6.1-6.3

## Description
Implement seed-based terrain generation where the same seed always produces the same map. Add playability validation: every generated map must have a minimum flat buildable area (>30% of cells), at least one water source, and a connected land mass large enough for a starter city. If validation fails, increment seed and retry. Add map preview UI showing terrain minimap before starting.

## Definition of Done
- [ ] `u64` seed parameter deterministically generates identical maps
- [ ] Playability validation rejects maps with <30% buildable area
- [ ] At least one contiguous flat area of 50x50 cells exists
- [ ] Seed input UI in new game screen
- [ ] Map preview thumbnail generated before game start
- [ ] Tests pass

## Test Plan
- Unit: Same seed produces identical heightmaps across runs
- Unit: Playability validator rejects a 100% water map
- Integration: 100 random seeds all produce playable maps (or retry succeeds within 10 attempts)

## Pitfalls
- Determinism requires identical float operations; use integer seeds only, avoid platform-dependent rounding
- `fastnoise-lite` seed is `i32`, not `u64`; may need truncation
- Retry loop needs a maximum attempt count to prevent infinite loops

## Relevant Code
- `crates/simulation/src/terrain.rs` -- seed parameter
- `crates/simulation/src/config.rs` -- seed storage

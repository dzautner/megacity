# POLL-025: Dirty Chunk Tracking for Pollution Grid Updates

## Priority: T2 (Depth)

## Description
Implement dirty chunk tracking to skip pollution calculations for chunks with no nearby sources and low pollution. Currently all 65,536 cells are updated every cycle. With 8x8 chunks (1,024 chunks), only chunks near sources need full update. Expected savings: 60-80% of cells skipped.

## Current State
- `PollutionGrid` clears to zero and recomputes every tick.
- No chunk-based spatial partitioning for pollution.
- Full grid iteration even for sparse cities.

## Definition of Done
- [ ] `BitVec` dirty flag per 8x8 chunk (1,024 bits = 128 bytes).
- [ ] Mark chunks within `max_radius` of any pollution source as dirty.
- [ ] Only update dirty chunks during dispersion.
- [ ] Chunks with pollution < 0.1 and no nearby sources: skip (fast decay to 0).
- [ ] Dirty flags recomputed when buildings are added/removed.
- [ ] Performance metric: measure actual cells skipped per update.

## Test Plan
- [ ] Unit test: chunk near factory is dirty; chunk in empty area is clean.
- [ ] Unit test: removing factory un-dirties chunks after pollution decays.
- [ ] Performance test: sparse city (10% developed) skips 60%+ of grid.
- [ ] Integration test: pollution values identical to non-optimized version.

## Pitfalls
- Dirty flag computation must not be more expensive than just updating everything.
- Edge effects: pollution near chunk boundaries must mark adjacent chunks.
- Adding wind drift complicates dirty tracking (plume moves across chunks).

## Code References
- `crates/simulation/src/pollution.rs`: `update_pollution`
- Research: `environment_climate.md` section 9.2.1

# POLL-011: Noise Barrier Attenuation (Buildings, Walls, Terrain)

## Priority: T3 (Differentiation)

## Description
Implement barrier attenuation for noise propagation. When a building, noise wall, tree row, or terrain hill is between a noise source and receiver, the noise level should be reduced. The research doc specifies Bresenham ray-casting with per-obstacle attenuation values capped at 40 dB total.

## Current State
- No barrier attenuation exists. Noise propagates through buildings unchanged.
- Trees reduce noise by a flat -2 in a 1-cell radius (not ray-based).
- No noise barrier wall building type.
- No terrain-based noise blocking.

## Definition of Done
- [ ] `estimate_barrier(sx, sy, rx, ry) -> f32` function using simplified ray check (3 sample points along path).
- [ ] Building barrier: solid = -15 dB, lightweight = -8 dB.
- [ ] Noise barrier wall (new building type): -12 dB.
- [ ] Dense trees: -3 dB per tree row crossed.
- [ ] Terrain elevation difference > 2: -10 dB (hill/berm).
- [ ] Total barrier attenuation capped at 40 dB.
- [ ] Performance: simplified 3-point sampling instead of full Bresenham per ray.

## Test Plan
- [ ] Unit test: noise behind a solid building is 15 dB lower than without.
- [ ] Unit test: barrier attenuation caps at 40 dB.
- [ ] Integration test: residential area behind a noise wall experiences less highway noise.
- [ ] Integration test: hill between highway and residential reduces noise.

## Pitfalls
- Full Bresenham ray-casting for every source-receiver pair is O(sources * cells * ray_length); must use simplified sampling.
- Building construction type (solid vs lightweight) is not currently tracked.
- Noise barrier wall needs to be added as a placeable building/service type.

## Code References
- `crates/simulation/src/noise.rs`: `update_noise_pollution`
- Research: `environment_climate.md` section 1.3.4

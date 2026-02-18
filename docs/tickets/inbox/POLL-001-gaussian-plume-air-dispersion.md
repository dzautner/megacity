# POLL-001: Implement Gaussian Plume Air Pollution Dispersion

## Priority: T2 (Depth)

## Description
Replace the current simple radial falloff + wind drift model in `pollution.rs` with a proper simplified Gaussian plume dispersion kernel. The current model clears the grid each tick, adds fixed values per source, and applies a wind drift step. The research doc specifies a wind-aware kernel with radial decay, wind directional bias (downwind elongation / upwind suppression), and stack height effects.

## Current State
- `PollutionGrid` is `u8` per cell (0-255), too coarse for the 0-1000 range in the research doc.
- Industrial buildings emit fixed `5 + level*3` scaled by policy multiplier.
- Wind drift is a single-cell shift per tick (not a plume shape).
- No per-source-type emission rates (all industrial buildings treated identically).
- No terrain blocking or vegetation filtering in dispersion.

## Definition of Done
- [ ] `PollutionGrid` uses `f32` or `u16` per cell to support 0-1000 range.
- [ ] `dispersion_kernel(dx, dy, wind_dir, wind_speed, stack_height) -> f32` implemented per research doc.
- [ ] Each source has a `max_radius` capped at 32 cells based on emission rate Q.
- [ ] Plume decays with `DECAY_RATE = 0.85` per update cycle instead of clearing to zero.
- [ ] Terrain blocking: cells behind hills reduce contribution by 0.3.
- [ ] Vegetation filtering: park/forest cells reduce contribution by 0.6.
- [ ] Existing overlay rendering updated to handle new value range.

## Test Plan
- [ ] Unit test: `dispersion_kernel` returns 1.0 at source, decreases with distance.
- [ ] Unit test: downwind cells receive more pollution than upwind cells when wind_speed > 0.
- [ ] Unit test: tall stack (stack_height=1.0) reduces near-field but extends far-field.
- [ ] Integration test: single factory, verify plume shape matches expected elongated ellipse.
- [ ] Performance test: 200 sources at radius 32 completes within 2ms.

## Pitfalls
- Changing from `u8` to wider type affects serialization (save/load migration needed).
- O(sources * radius^2) can be expensive; consider temporal amortization (1/4 grid per tick).
- Must update the overlay color ramp to map the new 0-1000 range.

## Code References
- `crates/simulation/src/pollution.rs`: `PollutionGrid`, `update_pollution`, `apply_wind_drift`
- `crates/simulation/src/wind.rs`: `WindState` (direction, speed, `direction_vector()`)
- `crates/rendering/src/overlay.rs`: pollution overlay rendering
- Research: `environment_climate.md` sections 1.1.3, 1.1.4

# POLL-017: Wind-Driven Pollution Drift System

## Priority: T1 (Core)

## Description
Enhance the current wind drift system for air pollution to properly shift the entire pollution grid based on wind direction and speed. Currently `apply_wind_drift` shifts by 1 cell in a cardinal direction. The new system should use fractional drift with interpolation.

## Current State
- `apply_wind_drift` runs every 3 ticks.
- Shifts pollution grid by 1 cell in the wind direction (N, S, E, W only).
- No diagonal drift.
- No fractional cell drift.

## Definition of Done
- [ ] Wind drift supports 8 directions (including diagonals).
- [ ] Drift magnitude = wind_speed * drift_rate per update.
- [ ] Fractional drift using bilinear interpolation between cells.
- [ ] Grid-wide drift operation is O(n) with no allocation.
- [ ] Strong wind pushes pollution off the map edge (boundary drain).
- [ ] Calm wind (speed < 0.1) does not drift.

## Test Plan
- [ ] Unit test: wind from west shifts pollution eastward.
- [ ] Unit test: diagonal wind (NE) shifts pollution in both axes.
- [ ] Unit test: calm wind produces no drift.
- [ ] Integration test: pollution plume from factory drifts downwind.
- [ ] Performance test: grid-wide drift completes within 0.5ms.

## Pitfalls
- Bilinear interpolation for fractional shifts may blur the grid over time.
- Must preserve total pollution mass during drift (no creation or destruction).
- Boundary conditions: pollution at map edges should dissipate, not wrap.

## Code References
- `crates/simulation/src/pollution.rs`: `apply_wind_drift`
- `crates/simulation/src/wind.rs`: `WindState.direction_vector()`
- Research: `environment_climate.md` section 1.1.4

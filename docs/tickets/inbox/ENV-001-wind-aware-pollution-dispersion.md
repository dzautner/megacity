# ENV-001: Wind-Aware Air Pollution Dispersion
**Priority:** T2
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M3; cities_skylines_analysis.md, section 10

## Description
Replace simple radius-based pollution with Gaussian plume model that disperses pollution downwind. Industrial pollution should blow in wind direction, creating asymmetric pollution patterns.

- Wind direction from weather system (wind.rs)
- Gaussian plume: concentration = Q/(2*pi*sigma_y*sigma_z*u) * exp(-y^2/(2*sigma_y^2)) * exp(-z^2/(2*sigma_z^2))
- Simplified for 2D grid: pollution spreads further downwind, less crosswind
- Wind speed affects dispersion distance (stronger wind = more spread, lower peak)
- Overlay shows wind-aware pollution contours
- Land value and health effects based on actual pollution at cell

## Definition of Done
- [ ] Pollution disperses in wind direction
- [ ] Gaussian plume model implemented (simplified)
- [ ] Pollution overlay shows wind-aware patterns
- [ ] Health and land value use dispersed pollution values

## Test Plan
- Unit: Pollution source with east wind shows higher pollution to the east
- Integration: Change wind direction, verify pollution pattern shifts

## Pitfalls
- wind.rs exists but may not provide continuous direction data
- Full Gaussian plume is expensive -- simplify to discrete grid with directional weighting
- Pollution sources include traffic, industrial, power plants

## Relevant Code
- `crates/simulation/src/pollution.rs` -- replace dispersion model
- `crates/simulation/src/wind.rs` -- wind direction input
- `crates/rendering/src/overlay.rs` -- pollution overlay

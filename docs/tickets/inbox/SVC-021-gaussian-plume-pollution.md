# SVC-021: Wind-Aware Gaussian Plume Pollution Dispersion

**Priority:** T2 (Depth)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.11

## Description

Replace simple neighbor diffusion for air pollution with wind-aware Gaussian plume model. Pollution from source spreads downwind in a cone pattern. Concentration at point (x,y) from source = Q / (2*pi*sigma_y*sigma_z*u) * exp(-y^2/(2*sigma_y^2)) * exp(-z^2/(2*sigma_z^2)). Wind direction from Weather resource determines dispersion direction. Industrial buildings, traffic, and power plants are sources with different emission rates. Technology upgrades (scrubbers) reduce source strength.

## Definition of Done

- [ ] Wind-aware dispersion replacing isotropic diffusion
- [ ] Gaussian plume approximation (simplified for grid)
- [ ] Pollution spreads primarily downwind
- [ ] Source emission rates per building type
- [ ] Technology upgrade policy reduces emissions (scrubbers: -50%)
- [ ] Pollution clears faster upwind, accumulates downwind
- [ ] Visual: pollution overlay shows wind-influenced patterns

## Test Plan

- Unit test: pollution concentration highest downwind of source
- Unit test: wind direction change shifts pollution pattern
- Unit test: scrubber technology reduces source strength
- Integration test: factory pollution visibly follows wind direction

## Pitfalls

- Full Gaussian plume is expensive; use discretized approximation on grid
- Wind changes should gradually shift pattern, not instant

## Relevant Code

- `crates/simulation/src/pollution.rs` (PollutionGrid, update_pollution)
- `crates/simulation/src/wind.rs` (WindState)
- `crates/simulation/src/weather.rs` (Weather)

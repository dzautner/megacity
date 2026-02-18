# INFRA-072: Wind-Aware Air Pollution Dispersion
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M3; infrastructure_engineering.md

## Description
Implement Gaussian plume model for air pollution dispersion. Pollution from sources (factories, power plants, traffic) spreads downwind based on wind direction and speed from `wind.rs`. Concentration at distance: `C = Q / (2*pi*u*sigma_y*sigma_z) * exp(-0.5*(y/sigma_y)^2) * exp(-0.5*(z/sigma_z)^2)`. Simplified for 2D grid: pollution concentration decreases with distance from source, concentrated in wind direction.

## Definition of Done
- [ ] Pollution sources emit at rate proportional to activity
- [ ] Wind direction and speed affect dispersion direction
- [ ] Pollution concentration computed per cell using Gaussian plume (simplified)
- [ ] Pollution overlay shows wind-influenced dispersion pattern
- [ ] Tests pass

## Test Plan
- Unit: Factory with north wind creates pollution plume to the south
- Unit: Higher wind speed disperses pollution faster (lower peak concentration)
- Integration: Placing industry upwind of residential creates visible health impact

## Pitfalls
- Full Gaussian plume is expensive for 256x256 grid; simplify to cone-shaped area
- Wind direction changes should gradually shift pollution pattern
- Multiple sources need superposition (sum of individual plumes)
- Current `pollution.rs` is basic; extend carefully

## Relevant Code
- `crates/simulation/src/pollution.rs` -- pollution system
- `crates/simulation/src/wind.rs` -- wind direction and speed

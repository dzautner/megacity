# POLL-018: Tree and Green Space Pollution Absorption Enhancement

## Priority: T2 (Depth)

## Description
Enhance the existing tree effects system with the research doc's vegetation filtering model. Currently trees reduce pollution by a flat 1-3 units in radius 2. The research doc specifies that vegetation cells reduce dispersion kernel contribution by 0.6x (air) and -1.5F UHI / -3dB noise per tree row.

## Current State
- Trees reduce air pollution by `3 - manhattan_distance` (max 3 at center) every 50 ticks.
- Trees reduce noise by the same formula.
- No vegetation fraction per cell.
- No cumulative tree row effect.

## Definition of Done
- [ ] Vegetation filtering in air dispersion: park/forest cells multiply incoming pollution by 0.6.
- [ ] Trees absorb CO2 (for future climate tracking): 48 lbs CO2/tree/year.
- [ ] Green space bonus: 10+ adjacent tree cells provide extra pollution absorption.
- [ ] Tree growth: planted trees take 5 game-days to reach full effectiveness.
- [ ] Tree canopy percentage per district for UHI calculation.
- [ ] Existing flat reduction replaced with percentage-based filtering.

## Test Plan
- [ ] Unit test: park cell receives 40% less pollution than bare cell.
- [ ] Unit test: newly planted tree has 0% effectiveness, full at 5 game-days.
- [ ] Integration test: green belt between factory and residential reduces pollution.

## Pitfalls
- Changing from absolute reduction to percentage filtering changes game balance.
- Tree growth timer needs tracking per tree (or simplified to city-wide maturity).
- Must not break existing tree planting/removal mechanics.

## Code References
- `crates/simulation/src/trees.rs`: `tree_effects`, `TreeGrid`
- `crates/simulation/src/pollution.rs`: pollution integration
- Research: `environment_climate.md` sections 1.1.4, 4.5.4

# POWER-006: Wind Turbine Farm Power Plant

## Priority: T1 (Core)

## Description
Implement wind turbine farm that generates power based on current wind speed. The existing `WindState` resource provides wind direction and speed (0-1). Wind power follows a cubic relationship with wind speed.

## Current State
- `WindState` exists in `wind.rs` with direction, speed, and compass/label functions.
- No wind power generation building.
- Wind speed is 0-1 normalized.

## Definition of Done
- [ ] Wind farm: 100 MW nameplate capacity.
- [ ] Power output follows cubic wind curve: `output = nameplate * wind_speed^3` (simplified).
- [ ] Cut-in speed: below 0.1, no output. Cut-out speed: above 0.95, shutdown for safety.
- [ ] Average capacity factor: 0.25-0.35 depending on location and wind patterns.
- [ ] Fuel cost: $0/MWh.
- [ ] Construction cost: $150M, build time: 4 game-days.
- [ ] Air pollution: Q=0.0.
- [ ] Noise: 55 dB source level.
- [ ] 3x3 building footprint.
- [ ] Placement bonus: higher elevation cells get +20% output.

## Test Plan
- [ ] Unit test: zero output below cut-in wind speed (0.1).
- [ ] Unit test: cubic scaling: 0.5 wind = 0.125 capacity factor.
- [ ] Unit test: shutdown above cut-out speed.
- [ ] Integration test: wind farm output varies with `WindState.speed`.
- [ ] Integration test: wind farm produces noise in surrounding area.

## Pitfalls
- Wind speed in `WindState` is 0-1; the cubic relationship means output is very low at moderate speeds.
- Must balance so wind is useful but not dominant at high wind speeds.
- Noise from wind turbines should affect nearby residential land value.

## Code References
- `crates/simulation/src/wind.rs`: `WindState`, `update_wind`
- Research: `environment_climate.md` section 3.2

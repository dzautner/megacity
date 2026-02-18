# DISASTER-019: Fire Smoke as Air Pollution Source

## Priority: T2 (Depth)

## Description
Implement smoke from burning cells (both building fires and wildfires) as a significant air pollution source. Each burning cell generates air pollution proportional to fire intensity, and the smoke plume follows wind direction using the air pollution dispersion model.

## Current State
- Forest fires affect land value and destroy trees but don't generate air pollution.
- Building fires don't generate air pollution.
- No smoke concept.

## Definition of Done
- [ ] Each burning cell (fire_intensity > 0) generates `air_pollution_source = fire_intensity * 50.0`.
- [ ] Forest fire cells generate `pollution = forest_fire_intensity * 30.0`.
- [ ] Smoke follows wind direction (uses existing air pollution wind drift).
- [ ] Smoke from large fires visible as persistent AQI spike downwind.
- [ ] Health advisory triggered when fire smoke pushes AQI > 200.
- [ ] Smoke clears within hours of fire extinguishment (fast decay).

## Test Plan
- [ ] Unit test: burning cell at intensity 100 generates 5000 units of air pollution.
- [ ] Unit test: smoke travels downwind.
- [ ] Integration test: large wildfire creates hazardous air quality downwind.
- [ ] Integration test: extinguishing fire clears smoke within a few ticks.

## Pitfalls
- Large fires generating massive pollution may overwhelm the AQI system.
- Must scale smoke contribution to be impactful but not break the pollution grid.
- Smoke is temporary (should decay faster than industrial pollution).

## Code References
- `crates/simulation/src/fire.rs`: `FireGrid`, building fires
- `crates/simulation/src/forest_fire.rs`: wildfire intensity
- `crates/simulation/src/pollution.rs`: air pollution integration
- Research: `environment_climate.md` section 5.3.6 (Smoke)

# WEATHER-011: Storm Wind Damage Thresholds (Beaufort Scale)

## Priority: T2 (Depth)

## Description
Implement wind damage scaling using Beaufort-inspired thresholds. The research doc specifies 7 tiers from "Breezy" (0-30 mph, no damage) to "Extreme" (111+ mph, catastrophic). Damage scales with the cube of wind speed above threshold. Combined with rain for compound storm damage.

## Current State
- `WeatherEvent::Storm` exists but has no infrastructure damage.
- `WindState.speed` is 0-1 normalized.
- Travel speed modifier exists (0.5 during storms) but no structural damage.

## Definition of Done
- [ ] Wind damage tiers: Breezy(0-0.3), Strong(0.3-0.45), Gale(0.45-0.6), Storm(0.6-0.75), Severe(0.75-0.9), Hurricane-force(0.9-0.95), Extreme(>0.95).
- [ ] Damage formula: `wind_damage = (wind_speed - 0.4)^3 * 1000` for speeds > 0.4.
- [ ] Hail damage: 15% to glass facades, 5% general at storm+hail events.
- [ ] Lightning: 0.2% fire ignition chance per cell during electrical storms.
- [ ] Power line damage: probability increases with wind speed > 0.6.
- [ ] Tree damage: trees knocked down at high wind speeds.

## Test Plan
- [ ] Unit test: no damage below wind speed 0.4.
- [ ] Unit test: cubic scaling at wind speed 0.7 = (0.3)^3 * 1000 = 27 damage units.
- [ ] Integration test: storm damages buildings and knocks down trees.
- [ ] Integration test: lightning starts fires during electrical storms.

## Pitfalls
- Current wind speed rarely exceeds 0.7; may need to increase range during storm events.
- Wind speed must be temporarily boosted during `WeatherEvent::Storm`.
- Tree knockdown removes trees from `TreeGrid`, which is permanent.

## Code References
- `crates/simulation/src/wind.rs`: `WindState`
- `crates/simulation/src/weather.rs`: `WeatherEvent::Storm`
- `crates/simulation/src/trees.rs`: `TreeGrid`
- Research: `environment_climate.md` section 4.4.3

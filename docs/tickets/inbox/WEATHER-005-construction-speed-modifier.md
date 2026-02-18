# WEATHER-005: Weather-Dependent Construction Speed Modifier

## Priority: T3 (Differentiation)

## Description
Implement construction speed modifiers based on season and weather conditions. Summer construction is 10% faster, winter is 40% slower, and storms halt construction entirely. Cost modifier also varies: winter is 25% more expensive.

## Current State
- No construction time system exists (buildings appear instantly).
- No weather impact on construction.

## Definition of Done
- [ ] Season speed factor: Spring=1.0, Summer=1.1, Autumn=0.9, Winter=0.6.
- [ ] Weather speed factor: Clear=1.0, Rain=0.5, Snow=0.3, Extreme cold=0.2, Storm=0.0.
- [ ] Cost modifier: Spring/Summer=1.0, Autumn=1.05, Winter=1.25.
- [ ] `construction_progress_per_tick = base_rate * season_factor * weather_factor`.
- [ ] Applied to building construction if/when build times are implemented.
- [ ] UI tooltip shows "Construction delayed due to weather" during storms/winter.

## Test Plan
- [ ] Unit test: storm weather factor = 0.0 (construction halted).
- [ ] Unit test: winter cost modifier = 1.25.
- [ ] Integration test: building takes longer to construct in winter.

## Pitfalls
- Depends on a construction time system existing (buildings are currently instant).
- If buildings are instant, this ticket must be deferred or paired with a build-time ticket.
- Very slow winter construction may frustrate players in Continental/Subarctic zones.

## Code References
- `crates/simulation/src/buildings.rs`: building spawning
- `crates/simulation/src/weather.rs`: `Weather`, `Season`
- Research: `environment_climate.md` section 4.3.3

# POWER-005: Solar Farm Power Plant

## Priority: T1 (Core)

## Description
Implement solar farm as a renewable power generator. Solar output varies by time of day, season, and cloud cover. Zero emissions but intermittent -- requires battery storage or backup generation for reliable supply.

## Current State
- No power generation buildings exist.
- `Weather` tracks season and weather conditions that would affect solar output.

## Definition of Done
- [ ] Solar farm: 50 MW nameplate capacity.
- [ ] Variable capacity factor by season: Spring=0.22, Summer=0.28, Autumn=0.18, Winter=0.12.
- [ ] Time-of-day output curve: zero at night, peak at noon.
- [ ] Cloud cover reduction: Overcast=-50%, Rain=-70%, Storm=-90%.
- [ ] Fuel cost: $0/MWh (zero marginal cost).
- [ ] Construction cost: $60M, build time: 3 game-days.
- [ ] Air pollution: Q=0.0.
- [ ] 4x4 building footprint (large land use).
- [ ] Actual output = nameplate * capacity_factor * time_curve * weather_modifier.

## Test Plan
- [ ] Unit test: solar output is zero at night (hour 0-6, 18-24).
- [ ] Unit test: summer capacity factor is higher than winter.
- [ ] Unit test: storm reduces output by 90%.
- [ ] Integration test: solar farm provides significant daytime power but nothing at night.
- [ ] Integration test: winter solar output is much lower than summer.

## Pitfalls
- Solar intermittency means the grid needs backup; without storage, solar alone causes blackouts at night.
- Must integrate with `GameClock.hour` for time-of-day output.
- Cloud cover mapping from `WeatherEvent` to solar modifier needs clear rules.

## Code References
- `crates/simulation/src/weather.rs`: `Weather`, `Season`, `WeatherEvent`
- `crates/simulation/src/time_of_day.rs`: `GameClock`
- Research: `environment_climate.md` sections 3.2, 4.3.1

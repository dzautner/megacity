# WEATHER-006: Heat Wave Health, Energy, and Infrastructure Effects

## Priority: T2 (Depth)

## Description
Implement detailed heat wave effects beyond the current happiness penalty. The research doc specifies exponential mortality curves, +40-80% energy demand, +60% water demand, road buckling, wildfire risk multiplier, and blackout risk from extreme AC load.

## Current State
- `WeatherEvent::HeatWave` exists, adds +10C to temperature.
- Happiness penalty of -5 during heat waves.
- No mortality effects, no energy demand spike, no infrastructure damage.

## Definition of Done
- [ ] Heat wave detection: 3+ consecutive days above 38C (100F) or 8C above seasonal average.
- [ ] Excess mortality curve: `excess_deaths_per_100k = 0.5 * exp(0.15 * (T - threshold))`.
- [ ] Vulnerability factors: age>65 (3x risk), age<5 (2x), no AC (5x), homeless (8x), outdoor workers (2.5x).
- [ ] Energy demand surge: +40-80% from AC load.
- [ ] Water demand surge: +60% from cooling and irrigation.
- [ ] Road damage: pavement buckling at sustained temperatures above 43C (110F).
- [ ] Wildfire risk multiplier: +300% when combined with drought.
- [ ] Blackout risk when AC demand exceeds grid capacity.

## Test Plan
- [ ] Unit test: mortality curve returns correct excess deaths at 100F, 105F, 110F.
- [ ] Unit test: elderly citizen has 3x risk multiplier.
- [ ] Integration test: heat wave causes excess deaths among elderly without AC.
- [ ] Integration test: heat wave triggers energy demand spike.

## Pitfalls
- Mortality mechanics may feel harsh to players; need balanced death rates.
- AC availability per citizen needs to be tracked (related to power service).
- Cascading failure: heat wave -> AC demand -> blackout -> more deaths.

## Code References
- `crates/simulation/src/weather.rs`: `WeatherEvent::HeatWave`
- `crates/simulation/src/happiness.rs`: happiness effects
- Research: `environment_climate.md` section 4.4.1

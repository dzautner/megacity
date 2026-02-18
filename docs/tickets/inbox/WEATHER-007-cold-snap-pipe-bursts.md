# WEATHER-007: Cold Snap Effects and Pipe Burst Probability

## Priority: T2 (Depth)

## Description
Implement detailed cold snap effects including pipe burst probability curves, increased heating demand, homeless mortality, vehicle failures, and school closures. The research doc provides a pipe burst probability per mile per day based on temperature tiers.

## Current State
- `WeatherEvent::ColdSnap` exists, subtracts 15C from temperature.
- Happiness penalty of -8 during cold snaps.
- No pipe burst mechanics.
- No infrastructure damage from cold.

## Definition of Done
- [ ] Cold snap detection: 3+ days below -12C (10F) or 11C below seasonal average.
- [ ] Pipe burst probability: 0.0001 baseline, 0.001 at freezing, 0.01 below -7C, 0.05 below -18C, 0.10 below -23C.
- [ ] Pipe bursts reduce water service to affected areas.
- [ ] Heating demand surge: +80-150% above normal.
- [ ] Homeless mortality: exponential below -18C without shelter.
- [ ] Vehicle failures: -20% traffic capacity.
- [ ] School closures below -29C (schools become shelters).
- [ ] Construction halted below -9C.
- [ ] Natural gas demand may exceed pipeline capacity (cascading failure).

## Test Plan
- [ ] Unit test: pipe burst rate at -20C matches expected 0.05/mile/day.
- [ ] Unit test: heating demand at -10C is 150%+ above normal.
- [ ] Integration test: extended cold snap causes water service disruptions.
- [ ] Integration test: homeless citizens die without shelter during extreme cold.

## Pitfalls
- Pipe burst mechanics require tracking water infrastructure mileage.
- "Miles of water mains" is not currently tracked; may approximate from road network length.
- Cold snap + heating demand spike + gas shortage = cascading failure.

## Code References
- `crates/simulation/src/weather.rs`: `WeatherEvent::ColdSnap`
- `crates/simulation/src/heating.rs`: `heating_demand()`
- Research: `environment_climate.md` section 4.4.2

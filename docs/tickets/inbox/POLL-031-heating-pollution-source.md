# POLL-031: Residential and Commercial Heating as Air Pollution Source

## Priority: T2 (Depth)

## Description
Implement residential and commercial heating (furnaces, boilers) as area pollution sources during cold weather. Building density and heating fuel type determine emissions. Gas heating is cleaner than oil or wood; electric heating produces no local emissions.

## Current State
- `HeatingPlant` types exist but don't generate air pollution.
- Residential buildings don't emit heating pollution.
- No heating fuel type distinction.

## Definition of Done
- [ ] High-density residential: Q=5.0 per cell when heating is active.
- [ ] Low-density residential: Q=2.0 per cell when heating is active.
- [ ] Commercial buildings: Q=3.0 per cell when heating is active.
- [ ] Heating fuel modifier: gas=1.0x, oil=1.5x, wood=2.0x, electric=0.0x.
- [ ] Heating is active when `heating_demand() > 0` (temperature < 10C).
- [ ] Seasonal scaling: emissions proportional to heating demand.
- [ ] Policy: "Clean Heat" mandate forces switch from oil/wood to gas/electric, reduces Q by 50%.

## Test Plan
- [ ] Unit test: heating emissions only active in cold weather.
- [ ] Unit test: electric heating produces zero local emissions.
- [ ] Unit test: dense residential emits more than sparse.
- [ ] Integration test: winter air quality is worse than summer in heating-dependent cities.

## Pitfalls
- Heating fuel type is not currently tracked per building.
- Must coordinate with existing heating system (heating.rs) for demand calculations.
- Clean Heat mandate policy needs policy panel integration.

## Code References
- `crates/simulation/src/heating.rs`: `heating_demand()`
- `crates/simulation/src/pollution.rs`: pollution sources
- Research: `environment_climate.md` section 1.1.2

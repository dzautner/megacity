# POWER-015: Oil-Fired Power Plant

## Priority: T3 (Differentiation)

## Description
Implement oil-fired power plant as a dispatchable but expensive and dirty power source. Available early in the game, becomes less competitive as cleaner options unlock.

## Current State
- No oil power generation.

## Definition of Done
- [ ] Oil plant: 100 MW capacity, dispatchable.
- [ ] Fuel cost: $70/MWh (expensive).
- [ ] Construction cost: $80M, build time: 5 game-days.
- [ ] Air pollution: Q=75.0 (high).
- [ ] CO2 emissions: 0.8 tons/MWh.
- [ ] 2x2 building footprint.
- [ ] Can use oil deposits from ResourceGrid for reduced fuel cost.

## Test Plan
- [ ] Unit test: fuel cost is $70/MWh.
- [ ] Integration test: oil plant dispatched only when cheaper options exhausted.

## Pitfalls
- Oil power is rarely cost-competitive; mainly for islands or areas without gas/coal.
- Oil supply chain from natural_resources.rs needs integration.

## Code References
- `crates/simulation/src/natural_resources.rs`: `ResourceType::Oil`
- Research: `environment_climate.md` section 3.2

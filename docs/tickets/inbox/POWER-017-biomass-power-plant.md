# POWER-017: Biomass Power Plant

## Priority: T3 (Differentiation)

## Description
Implement biomass power plant that burns organic waste or dedicated biomass crops for electricity. Considered renewable if feedstock is sustainably sourced. Moderate emissions and capacity.

## Current State
- No biomass generation.

## Definition of Done
- [ ] Biomass plant: 50 MW capacity, 0.75 capacity factor.
- [ ] Fuel cost: $25/MWh (lower if using city compost/organic waste).
- [ ] Construction cost: $100M, build time: 6 game-days.
- [ ] Air pollution: Q=30.0 (moderate; organic combustion).
- [ ] CO2 emissions: 0.0 net (carbon-neutral if sustainable feedstock).
- [ ] 3x3 building footprint.
- [ ] Synergy: if composting facility exists, fuel cost reduced by 30%.

## Test Plan
- [ ] Unit test: biomass output = 50 MW at full capacity.
- [ ] Unit test: fuel cost reduced with composting facility nearby.
- [ ] Integration test: biomass plant contributes to power grid.

## Pitfalls
- Carbon-neutral claim is simplified; real biomass has complex accounting.
- Biomass feedstock supply chain is not modeled; treat as always available.

## Code References
- Research: `environment_climate.md` section 3.2

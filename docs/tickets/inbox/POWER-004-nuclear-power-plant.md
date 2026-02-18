# POWER-004: Nuclear Power Plant

## Priority: T2 (Depth)

## Description
Implement nuclear power plant as a high-capacity, zero-emission (air) baseload generator. Nuclear provides massive reliable power but has very high construction cost, long build time, and produces radioactive waste requiring a hazardous waste facility.

## Current State
- No power generation buildings exist.

## Definition of Done
- [ ] Nuclear plant: 1000 MW capacity, 0.90 capacity factor (highest), baseload-only.
- [ ] Fuel cost: $10/MWh (very cheap).
- [ ] Construction cost: $5B (most expensive), build time: 20 game-days.
- [ ] Air pollution: Q=0.0 (zero emissions).
- [ ] CO2 emissions: 0.0 tons/MWh.
- [ ] Requires hazardous waste facility for spent fuel.
- [ ] Meltdown risk: 0.001% per game-year if maintenance neglected (disaster event).
- [ ] 5x5 building footprint.
- [ ] Exclusion zone: no residential within 3 cells.
- [ ] Cooling water demand: requires placement near water body.

## Test Plan
- [ ] Unit test: nuclear plant capacity factor is 0.90.
- [ ] Unit test: fuel cost is lowest of all thermal plants.
- [ ] Integration test: nuclear plant provides massive base load with zero air pollution.
- [ ] Integration test: no hazardous waste facility triggers penalty/alert.

## Pitfalls
- Meltdown disaster is a special case that needs its own disaster type.
- Exclusion zone enforcement requires placement validation logic.
- Cooling water requirement means not all maps can use nuclear.

## Code References
- Research: `environment_climate.md` section 3.2

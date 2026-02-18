# POWER-002: Coal Power Plant Generator

## Priority: T1 (Core)

## Description
Implement coal power plant as a placeable power generator building. Coal plants provide reliable baseload power but produce heavy air pollution and require fuel supply.

## Current State
- No power generation buildings exist.
- Power is implicitly available everywhere.
- No generation capacity tracking.

## Definition of Done
- [ ] `PowerPlant` component with `plant_type: PowerPlantType`, `capacity_mw: f32`, `current_output_mw: f32`, `fuel_cost: f32`.
- [ ] Coal plant: 200 MW capacity, 0.33 capacity factor, dispatchable (always available).
- [ ] Fuel cost: $30/MWh.
- [ ] Construction cost: $200M, build time: 8 game-days.
- [ ] Air pollution source: Q=100.0 with stack_height=1.0.
- [ ] CO2 emissions: 1.0 tons/MWh.
- [ ] 3x3 building footprint.
- [ ] Coal plant contributes to `EnergyGrid.total_supply_mwh`.

## Test Plan
- [ ] Unit test: coal plant generates 200 MW at full capacity.
- [ ] Unit test: fuel cost = output_mwh * $30.
- [ ] Integration test: placing coal plant increases total energy supply.
- [ ] Integration test: coal plant produces air pollution in surrounding area.

## Pitfalls
- Coal plants should be available from the start as the cheapest power option.
- Must integrate with air pollution system (POLL-002) for Q values.
- Build time system may not exist yet.

## Code References
- `crates/simulation/src/services.rs`: new building type
- `crates/simulation/src/pollution.rs`: pollution source integration
- Research: `environment_climate.md` section 3.2

# POWER-013: Geothermal Power Plant

## Priority: T3 (Differentiation)

## Description
Implement geothermal power plant as a renewable, dispatchable baseload generator. Map-dependent (only available on maps with geothermal resources). Zero air emissions, constant output regardless of weather.

## Current State
- `HeatingPlantType::Geothermal` exists for heating but not for electricity generation.
- No geothermal power generation.

## Definition of Done
- [ ] Geothermal plant: 30 MW capacity, 0.90 capacity factor, baseload.
- [ ] Fuel cost: $0/MWh (resource is free heat).
- [ ] Construction cost: $100M, build time: 8 game-days.
- [ ] Air pollution: Q=5.0 (trace H2S emissions).
- [ ] Map-dependent: only placeable on geothermal resource cells.
- [ ] 2x2 building footprint.
- [ ] Can also provide district heating as co-benefit.

## Test Plan
- [ ] Unit test: capacity factor is 0.90 regardless of weather.
- [ ] Unit test: can only be placed on geothermal resource cells.
- [ ] Integration test: geothermal provides steady baseload.

## Pitfalls
- Geothermal resource cells need to be added to terrain generation.
- Not all maps should have geothermal resources.
- Interaction with existing `HeatingPlantType::Geothermal` needs clarification.

## Code References
- `crates/simulation/src/heating.rs`: `HeatingPlantType::Geothermal`
- `crates/simulation/src/natural_resources.rs`: `ResourceGrid`
- Research: `environment_climate.md` section 3.2

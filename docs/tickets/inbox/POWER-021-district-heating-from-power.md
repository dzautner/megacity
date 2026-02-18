# POWER-021: Combined Heat and Power (CHP) from Power Plants

## Priority: T3 (Differentiation)

## Description
Implement combined heat and power where thermal power plants and WTE plants can provide district heating as a co-product of electricity generation. This improves overall energy efficiency and connects the power system to the heating system.

## Current State
- `HeatingPlantType` has its own building types (SmallBoiler, DistrictHeating, Geothermal).
- No connection between power plants and heating system.
- Power plants waste heat.

## Definition of Done
- [ ] CHP upgrade for thermal power plants (coal, gas, biomass): +15% overall efficiency.
- [ ] WTE plant CHP: provides district heating within radius 20 from waste heat.
- [ ] CHP heating coverage: replaces need for separate heating plant in served area.
- [ ] CHP construction: $20M upgrade cost for existing plants.
- [ ] Heat output: proportional to electricity output (0.5x kWh in heat per kWh electricity).
- [ ] Integration with `HeatingGrid` for coverage calculation.

## Test Plan
- [ ] Unit test: CHP plant provides both electricity and heat.
- [ ] Unit test: CHP upgrade increases overall efficiency.
- [ ] Integration test: CHP plant covers heating needs in its radius.
- [ ] Integration test: CHP reduces need for separate heating infrastructure.

## Pitfalls
- CHP during summer generates unwanted heat; may reduce electricity output.
- Integration between power and heating systems adds complexity.
- Must work with existing `HeatingGrid` BFS propagation.

## Code References
- `crates/simulation/src/heating.rs`: `HeatingGrid`, BFS propagation
- Research: `environment_climate.md` section 3.2 (CHP plants)

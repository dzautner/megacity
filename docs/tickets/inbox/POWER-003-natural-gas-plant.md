# POWER-003: Natural Gas Power Plant (Combined Cycle)

## Priority: T1 (Core)

## Description
Implement natural gas combined-cycle power plant. Gas plants are cleaner than coal, faster to ramp, and serve as mid-merit/peaking generation. They produce moderate air pollution and are fuel-dependent.

## Current State
- No power generation buildings exist.

## Definition of Done
- [ ] Gas combined-cycle: 500 MW capacity, 0.45 capacity factor, dispatchable.
- [ ] Fuel cost: $40/MWh.
- [ ] Construction cost: $500M, build time: 6 game-days.
- [ ] Air pollution: Q=35.0 (65% less than coal).
- [ ] CO2 emissions: 0.4 tons/MWh (60% less than coal).
- [ ] Ramp rate: fast (can respond to demand changes within 1 tick).
- [ ] 2x3 building footprint.
- [ ] Gas peaker variant: 100 MW, fuel cost $80/MWh, only runs during peak demand.

## Test Plan
- [ ] Unit test: gas plant output = 500 MW max.
- [ ] Unit test: fuel cost is higher than coal per MWh.
- [ ] Unit test: pollution Q is 35% of coal plant.
- [ ] Integration test: gas plant ramps up when demand exceeds baseload.

## Pitfalls
- Gas peaker vs combined-cycle distinction adds complexity; may simplify to one type initially.
- Fuel cost assumes natural gas availability; may need a fuel supply chain later.
- Must integrate with dispatch system (POWER-009).

## Code References
- Research: `environment_climate.md` section 3.2

# INFR-001: Power Demand/Supply Balance with Brownout/Blackout
**Priority:** T1
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M2; cities_skylines_analysis.md, section 10

## Description
Implement power demand/supply balance. Currently power propagates via BFS with no capacity limits. Buildings should consume power based on zone type and level. When supply < demand, brownouts and blackouts occur.

- Power demand per building: Residential 5kW/L1, 15/L2, 30/L3, 50/L4, 100/L5; Commercial 10/L1... ; Industrial 20/L1...
- Power supply: from power plants (Coal 100MW, Gas 50MW, Solar 10MW, Wind 15MW, Nuclear 200MW)
- When total_demand > total_supply: blackout probability = (demand - supply) / demand
- Blackout effects: buildings lose power flag, no building upgrade, happiness -15, some businesses close
- Brownout: when supply within 10% of demand, random buildings lose power temporarily
- Power grid visualization: overlay showing supply/demand per area

## Definition of Done
- [ ] Power demand calculated per building
- [ ] Power supply from power plants totaled
- [ ] Brownout/blackout when supply < demand
- [ ] Blackout effects on buildings and happiness
- [ ] Power supply/demand displayed in utility panel

## Test Plan
- Unit: Building power demand > 0 for all zone/level combos
- Integration: Remove power plant, verify blackout occurs
- Integration: Build excess power, verify all buildings powered

## Pitfalls
- Current BFS propagation doesn't track capacity -- needs redesign or capacity overlay
- Power demand changes when buildings upgrade (must recalculate)
- Solar/wind output varies with weather (day/night, wind speed)

## Relevant Code
- `crates/simulation/src/utilities.rs` -- power propagation
- `crates/simulation/src/buildings.rs:Building` -- power demand per level
- `crates/simulation/src/happiness.rs` -- blackout penalty

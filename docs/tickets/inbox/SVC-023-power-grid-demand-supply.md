# SVC-023: Power Grid Demand/Supply Balance

**Priority:** T2 (Depth)
**Complexity:** High (4-5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.13

## Description

Replace binary has_power with demand/supply power grid. Each building has power demand (residential: 1.2kW avg, commercial: 5kW, industrial: 20kW). Power plants have generation capacity (coal: 500MW, gas: 200MW, nuclear: 1000MW, solar: 50MW variable, wind: 100MW variable). Grid must balance: supply >= demand or brownouts. Time-of-day demand curves (residential peaks evening, commercial peaks daytime). Blackout cascade: underserved areas lose power, traffic lights fail, hospitals on backup.

## Definition of Done

- [ ] Per-building power demand calculation
- [ ] Power plant generation capacity per type
- [ ] Grid balance: total_supply vs total_demand
- [ ] Brownout when supply < demand (random buildings lose power)
- [ ] Blackout cascade when supply < 80% demand
- [ ] Time-of-day demand curves
- [ ] Solar/wind variability (weather-dependent output)
- [ ] Power grid stats in utility panel
- [ ] Power shortage notification

## Test Plan

- Unit test: residential demand peaks at evening
- Unit test: solar output zero at night
- Unit test: brownout occurs when supply < demand
- Integration test: building power plant restores grid balance

## Pitfalls

- Current BFS propagation marks has_power; must coexist during transition
- Blackout cascade must not be too punishing (gradual, not instant)

## Relevant Code

- `crates/simulation/src/utilities.rs` (UtilitySource, propagate_utilities)
- `crates/simulation/src/grid.rs` (Cell.has_power)

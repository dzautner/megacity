# ECON-003: Economic Cycles (Boom/Bust)
**Priority:** T2
**Complexity:** L
**Dependencies:** ECON-001
**Source:** economic_simulation.md, section 4; master_architecture.md, section 2

## Description
Implement periodic economic cycles that create boom and bust phases. Currently the economy is stable monotonic growth. Cycles create dynamic fiscal challenges and prevent the "stable plateau" problem.

- Economic cycle: 4-7 game-year period
- Phases: Expansion -> Peak -> Contraction -> Trough
- Effects during expansion: +20-40% commercial income, +immigration, +construction, higher demand
- Effects during contraction: -20-40% commercial income, -immigration, +abandonment, lower demand
- Cycle amplitude increases with city size (larger cities = more volatile)
- Random external shocks can trigger recession (10% chance per year)
- Player can mitigate via fiscal reserves, diversified economy, policy tools

## Definition of Done
- [ ] Economic cycle system with 4 phases
- [ ] Cycle affects commercial income, immigration, construction rate, zone demand
- [ ] Random external shocks can trigger off-cycle recession
- [ ] Cycle phase visible in economy panel
- [ ] Player actions can partially mitigate cycle effects

## Test Plan
- Unit: Expansion phase multiplies commercial income by > 1.0
- Unit: Contraction phase multiplies commercial income by < 1.0
- Integration: City survives full cycle without bankruptcy (at default tax rates)
- Integration: Economic indicator shows cycle phase

## Pitfalls
- Bust phase too harsh = player bankruptcy = frustration
- Bust phase too mild = no impact = boring
- Must tune so prepared players survive, unprepared players face difficult choices
- Cycle period needs to be long enough for player to react

## Relevant Code
- `crates/simulation/src/economy.rs` -- add EconomicCycle resource
- `crates/simulation/src/zones.rs:update_zone_demand` -- cycle affects demand
- `crates/simulation/src/immigration.rs` -- cycle affects immigration
- `crates/simulation/src/buildings.rs:building_spawner` -- cycle affects spawn rate

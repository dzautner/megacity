# INFRA-069: Market-Driven Zone Demand
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M2

## Description
Replace simple zone demand with market-driven model. Residential demand from immigration pressure and housing shortage. Commercial demand from consumer spending (population * income). Industrial demand from commercial supply needs. Office demand from employment gap. Demand should feed back into building spawn rates: high demand = faster spawning, low demand = slower/no spawning. Display RCI (Residential/Commercial/Industrial) demand bars in UI like SimCity.

## Definition of Done
- [ ] Per-zone-type demand calculation from economic factors
- [ ] Demand affects building spawn rate
- [ ] RCI demand bar UI showing relative demand per zone type
- [ ] Zero demand = no new buildings spawn
- [ ] Excess supply over demand leads to vacancy
- [ ] Tests pass

## Test Plan
- Unit: High immigration + low housing = high residential demand
- Unit: High population + low shops = high commercial demand
- Integration: RCI bars visually guide player on what to zone

## Pitfalls
- Current `zones.rs` has basic demand; extend rather than replace
- Demand oscillations (build lots, demand crashes, abandonment, demand rises) need damping

## Relevant Code
- `crates/simulation/src/zones.rs` -- zone demand
- `crates/simulation/src/buildings.rs` -- building spawn rate
- `crates/simulation/src/market.rs` -- market conditions

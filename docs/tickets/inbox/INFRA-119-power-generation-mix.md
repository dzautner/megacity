# INFRA-119: Power Generation Mix and Merit Order Dispatch
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-021
**Source:** infrastructure_engineering.md, Section 7

## Description
Implement merit order dispatch for power generation. Plants dispatched in order of marginal cost: Nuclear ($0/MWh, baseload) -> Hydro ($0, flexible) -> Wind/Solar ($0, intermittent) -> Coal ($20-30) -> Natural Gas CC ($30-50) -> Natural Gas CT ($50-100, peaking) -> Oil ($100-200, emergency). Wholesale price = marginal cost of most expensive running plant. Players build a generation portfolio balancing baseload, intermediate, and peaking capacity.

## Definition of Done
- [ ] Power plant types with marginal cost and ramp speed
- [ ] Merit order dispatch algorithm
- [ ] Wholesale electricity price from dispatch
- [ ] Time-of-day demand curve
- [ ] Peaker plants only run during peaks
- [ ] Generation mix display in power info panel
- [ ] Tests pass

## Test Plan
- Unit: Low demand dispatches only nuclear/hydro; high demand adds gas peakers
- Unit: Wholesale price increases when peakers are needed
- Integration: Adding solar reduces midday prices but creates evening ramp

## Pitfalls
- Duck curve problem with high solar penetration needs battery storage
- Nuclear ramp speed is days; can't respond to demand changes
- Wind/solar intermittency creates reliability challenges

## Relevant Code
- `crates/simulation/src/utilities.rs` -- power system
- `crates/simulation/src/economy.rs` -- electricity costs

# INFRA-086: Business Profitability and Closure Simulation
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-053
**Source:** master_architecture.md, M4

## Description
Model individual business (commercial/industrial building) profitability. Revenue from customer traffic and production output. Costs from rent, wages, utilities, taxes. Unprofitable businesses close (building becomes vacant). New businesses open in vacant buildings if conditions improve. Business closure cascades: job loss -> unemployment -> reduced spending -> more closures.

## Definition of Done
- [ ] Per-building revenue and cost tracking
- [ ] Profitability = revenue - costs
- [ ] Unprofitable buildings become vacant after grace period
- [ ] Vacant buildings available for new businesses
- [ ] Business closure affects employment stats
- [ ] Tests pass

## Test Plan
- Unit: Business with high rent and low foot traffic becomes unprofitable
- Unit: Vacant building gets new tenant if conditions improve
- Integration: Economic downturn causes visible business closures

## Pitfalls
- Too-granular business simulation is expensive; aggregate by building
- Closure cascades can create death spirals; need dampening
- Existing `abandonment.rs` handles building abandonment; may overlap

## Relevant Code
- `crates/simulation/src/buildings.rs` -- building properties
- `crates/simulation/src/abandonment.rs` -- vacancy/abandonment system
- `crates/simulation/src/market.rs` -- market conditions

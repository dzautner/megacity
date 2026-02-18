# INFRA-062: Economic Boom/Bust Cycles
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-053
**Source:** master_architecture.md, M3

## Description
Implement periodic economic cycles that affect immigration, commercial demand, tax revenue, and construction. Cycle length 3-7 game-years. Boom: +20-40% immigration, +15% commercial demand, +10% tax revenue. Bust: -30% immigration, -20% commercial demand, -15% revenue, increased unemployment, building abandonment. Cycle driven by random walk + mean-reversion. Prevents the "stable plateau" where nothing changes.

## Definition of Done
- [ ] `EconomicCycle` resource with phase, strength, and duration
- [ ] Boom/bust modifiers applied to immigration, demand, revenue
- [ ] Bust phase increases unemployment and abandonment
- [ ] Economic indicator displayed in UI (confidence index)
- [ ] Advisor warnings before and during bust
- [ ] Tests pass

## Test Plan
- Unit: Boom phase increases immigration rate by 20-40%
- Unit: Bust phase decreases tax revenue by 15%
- Integration: City experiences visible ups and downs over game time

## Pitfalls
- Too-severe busts can bankrupt player with no recovery path
- Cycles must be readable by player; use UI indicators
- Existing `market.rs` may already have some cycle logic; unify

## Relevant Code
- `crates/simulation/src/economy.rs` -- economic modifiers
- `crates/simulation/src/market.rs` -- market conditions
- `crates/simulation/src/immigration.rs` -- immigration rate modifiers

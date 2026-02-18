# CIT-055: Business Profitability Simulation

**Priority:** T3 (Differentiation)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** CIT-052 (property tax), CIT-054 (labor market)
**Source:** master_architecture.md Section 1.8

## Description

Commercial and industrial buildings simulate business profitability. Revenue from: customer spending (commercial), production output (industrial), contracts (office). Costs: rent (land value based), wages (from labor market), utilities, taxes. Profit = revenue - costs. Negative profit for 6+ months = business closure (building abandonment). Profitable businesses upgrade (building level up). Business profitability drives zone demand more accurately than population ratios.

## Definition of Done

- [ ] Per-building revenue estimation (customer traffic * spending)
- [ ] Per-building cost calculation (rent + wages + utilities + taxes)
- [ ] Profit = revenue - costs per building per month
- [ ] Negative profit counter (months in red)
- [ ] Business closure at 6+ months negative profit
- [ ] Profitable businesses trigger level-up
- [ ] Zone demand derived from average profitability (not just population ratio)
- [ ] Business profitability visible in building inspection

## Test Plan

- Unit test: profitable business stays open
- Unit test: unprofitable business closes after 6 months
- Unit test: high rent reduces profitability
- Integration test: zone demand shifts based on profitability signals

## Pitfalls

- Revenue estimation for commercial depends on nearby population and traffic
- Must balance so businesses aren't constantly closing

## Relevant Code

- `crates/simulation/src/buildings.rs` (Building)
- `crates/simulation/src/zones.rs` (ZoneDemand)
- `crates/simulation/src/economy.rs`

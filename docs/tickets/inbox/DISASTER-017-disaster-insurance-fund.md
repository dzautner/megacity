# DISASTER-017: City Disaster Fund and Insurance System

## Priority: T2 (Depth)

## Description
Implement a city disaster fund and insurance system. Players can pre-fund an emergency reserve, purchase insurance (2% property value/year covering 80% of damage), and receive federal aid for major disasters (>10% of city budget). Without coverage, full repair cost falls on the city budget.

## Current State
- No disaster fund.
- No insurance concept.
- No federal aid system.
- Disaster repair costs are not tracked.

## Definition of Done
- [ ] `DisasterFund` resource: player-managed reserve fund with manual contributions.
- [ ] Insurance policy: toggleable, costs 2% of total property value/year.
- [ ] Insurance payout: covers 80% of repair costs when disaster occurs.
- [ ] Federal aid trigger: damage > 10% of annual city budget, covers 75%.
- [ ] Uninsured cost: full repair cost on city budget.
- [ ] Fund depletion tracking: warning when fund falls below 50% of expected loss.
- [ ] Budget integration: insurance premiums and fund contributions as line items.

## Test Plan
- [ ] Unit test: insurance premium = 2% of property value.
- [ ] Unit test: insurance payout = 80% of damage cost.
- [ ] Unit test: federal aid triggers at correct threshold.
- [ ] Integration test: insured city recovers faster financially from earthquake.

## Pitfalls
- Total property value calculation requires summing all building values.
- Insurance + federal aid together may make disasters too cheap; cap combined coverage at 90%.
- Player may never contribute to disaster fund if no disasters occur.

## Code References
- `crates/simulation/src/economy.rs`: budget system
- Research: `environment_climate.md` section 5.7

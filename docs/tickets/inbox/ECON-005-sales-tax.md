# ECON-005: Sales Tax on Commercial Revenue
**Priority:** T2
**Complexity:** S
**Dependencies:** ECON-001
**Source:** economic_simulation.md, section 1.3

## Description
Add sales tax as a percentage of commercial building revenue. This replaces the crude "occ * 1.0" commercial income with a revenue-based model where commercial buildings generate gross revenue from foot traffic and sales, and the city collects a percentage.

- Commercial building gross revenue = f(occupants, foot_traffic, land_value)
- Sales tax rate: 0-10% (default 6%, adjustable by player)
- Tax collected = gross_revenue * sales_tax_rate
- Higher sales tax reduces commercial attractiveness (fewer businesses open)
- Tourist spending also generates sales tax (tourism_visitors * avg_spending * tax_rate)

## Definition of Done
- [ ] Commercial buildings generate computed gross revenue
- [ ] Sales tax rate adjustable by player
- [ ] Sales tax revenue tracked separately in budget
- [ ] High sales tax rate reduces commercial demand

## Test Plan
- Unit: Commercial building revenue scales with occupants
- Unit: Sales tax at 10% generates double the revenue of 5%
- Integration: Raise sales tax, verify commercial demand decreases

## Pitfalls
- Must not make commercial income formula too complex (keep it comprehensible to player)
- Interaction with commercial occupancy -- empty stores should generate no revenue
- Tourist spending creates tax revenue even if no residents shop there

## Relevant Code
- `crates/simulation/src/economy.rs:collect_taxes` -- replace commercial income formula
- `crates/simulation/src/budget.rs` -- add sales_tax_rate field
- `crates/simulation/src/tourism.rs` -- tourist spending feeds sales tax

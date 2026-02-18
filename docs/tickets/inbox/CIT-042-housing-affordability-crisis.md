# CIT-042: Housing Affordability Crisis Mechanics

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** CIT-004 (household), CIT-041 (gentrification)
**Source:** social_agent_simulation.md Section 11, master_architecture.md Section 1.20

## Description

Housing affordability crisis as emergent late-game challenge. Housing supply/demand imbalance drives up rents. Housing price index = demand / supply * base_price. Rent burden distribution tracks what fraction of households spend >30%, >50% of income on housing. Crisis triggers when >40% of households are rent-burdened. Crisis effects: homelessness spike, emigration of working class, skilled worker shortage. Policy responses: rent control, inclusionary zoning, public housing, density bonuses.

## Definition of Done

- [ ] Housing supply tracking (total dwelling units)
- [ ] Housing demand tracking (total households + incoming)
- [ ] Housing price index from supply/demand ratio
- [ ] Rent burden distribution (% at 0-30%, 30-50%, >50%)
- [ ] Crisis flag when >40% rent-burdened
- [ ] Crisis notification and advisor warning
- [ ] Policy tools: rent control, inclusionary zoning, public housing
- [ ] Worker shortage from housing crisis (can't attract workers)

## Test Plan

- Unit test: demand > supply raises prices
- Unit test: >40% rent-burdened triggers crisis
- Unit test: rent control reduces rent burden
- Integration test: growing city without housing investment triggers crisis

## Pitfalls

- Housing crisis should be preventable with proactive zoning
- Rent control must have tradeoffs (reduced construction, as in GOV-005)

## Relevant Code

- `crates/simulation/src/buildings.rs` (Building.capacity)
- `crates/simulation/src/zones.rs` (ZoneDemand)

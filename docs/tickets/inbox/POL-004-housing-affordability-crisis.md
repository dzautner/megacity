# POL-004: Housing Affordability Crisis Mechanic
**Priority:** T3
**Complexity:** M
**Dependencies:** ECON-011, ECON-012, ECON-017
**Source:** master_architecture.md, section M4

## Description
Model housing affordability as a crisis mechanic. When rents exceed citizen incomes, affordable housing shortage triggers homelessness, protests, and emigration. Player must respond with policy tools.

- Affordability ratio: rent / income (healthy < 0.3, stressed 0.3-0.5, crisis > 0.5)
- When city-wide affordability ratio > 0.4: crisis indicator appears
- Crisis effects: homelessness increases, low-income emigration, protests
- Player tools: public housing construction, rent control, inclusionary zoning, LVT
- Crisis severity scales with duration (mild at 1 month, severe at 12 months)

## Definition of Done
- [ ] Affordability ratio tracked per income bracket
- [ ] Crisis indicator when ratio exceeds threshold
- [ ] Crisis effects (homelessness, emigration) active
- [ ] Policy mitigation tools available
- [ ] Crisis visible in advisor warnings

## Test Plan
- Integration: Rapid land value increase with no new housing, verify crisis triggers
- Integration: Build public housing, verify affordability improves

## Pitfalls
- homelessness.rs already exists -- integrate with affordability system
- Crisis must be solvable but not trivially (real cities struggle with this)
- Must interact with all housing-related systems

## Relevant Code
- `crates/simulation/src/homelessness.rs` -- homelessness tracking
- `crates/simulation/src/market.rs` -- rent/affordability
- `crates/simulation/src/advisors.rs` -- crisis warnings

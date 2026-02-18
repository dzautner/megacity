# CIT-041: Gentrification Model

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** CIT-019 (segregation), CIT-035 (relative deprivation)
**Source:** social_agent_simulation.md Section 6

## Description

Emergent gentrification from land value + demographics interaction. When land value rises in low-income area (due to transit investment, new amenities): rents rise, existing low-income residents face rent burden, gradual displacement as they can't afford rent, higher-income residents move in, businesses change (coffee shops replace bodegas), character of neighborhood transforms. Track displacement rate, income composition change, cultural facility turnover per district.

## Definition of Done

- [ ] Land value increase triggers rent increase in area
- [ ] Rent increase causes rent burden for existing low-income residents
- [ ] Rent-burdened residents added to relocation queue
- [ ] Higher-income residents fill vacancies
- [ ] District income composition tracking over time
- [ ] Displacement rate metric per district
- [ ] Commercial building type changes with income composition
- [ ] Gentrification overlay showing neighborhoods undergoing change

## Test Plan

- Unit test: transit station increases nearby land value
- Unit test: land value increase raises rents
- Unit test: rent increase displaces low-income residents
- Integration test: new subway station triggers neighborhood gentrification over 5-10 years

## Pitfalls

- Gentrification should be emergent, not scripted; avoid hardcoding
- Must be observable but not instant (gradual over game-years)

## Relevant Code

- `crates/simulation/src/land_value.rs`
- `crates/simulation/src/homelessness.rs`
- `crates/simulation/src/districts.rs`

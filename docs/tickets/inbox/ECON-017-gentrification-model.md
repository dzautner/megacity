# ECON-017: Gentrification and Displacement Model
**Priority:** T3
**Complexity:** L
**Dependencies:** ECON-010, ECON-012
**Source:** economic_simulation.md, section 3.1; master_architecture.md, section 3

## Description
Model gentrification as the reverse of housing filtering. When land value increases rapidly in a low-income neighborhood, property owners renovate, rents increase, and lower-income residents are displaced by higher-income residents.

- Gentrification trigger: land value increase rate > 5%/year for 3+ years in low-income district
- Effect: building quality increases (renovation), rent increases, target income bracket shifts up
- Displaced citizens must find new housing (may become homeless or leave city)
- Political opposition from existing residents (happiness penalty, protest events)
- Player can mitigate via rent control policy, inclusionary zoning (ZONE-010), public housing

## Definition of Done
- [ ] Gentrification detected via land value trend analysis
- [ ] Building renovation increases quality and rent
- [ ] Low-income citizens displaced when rent exceeds budget
- [ ] Displacement visible as emigration or homelessness
- [ ] Policy tools available to mitigate

## Test Plan
- Integration: Rapidly improve services in low-income area, verify gentrification effects
- Integration: Enable rent control, verify displacement reduced

## Pitfalls
- Gentrification is a controversial topic -- game should present mechanics, not judge
- Must balance: preventing gentrification entirely makes improvement pointless
- Displacement cascade: displaced citizens increase demand elsewhere, potentially displacing others

## Relevant Code
- `crates/simulation/src/land_value.rs` -- trend detection
- `crates/simulation/src/citizen.rs` -- income vs rent affordability
- `crates/simulation/src/homelessness.rs` -- displacement outcome
- `crates/simulation/src/policies.rs` -- rent control policy

# CIT-097: Informal Settlements / Slums

**Priority:** T3 (Differentiation)
**Complexity:** Medium-High (3-4 person-weeks)
**Dependencies:** CIT-039 (homelessness)
**Source:** historical_demographics_services.md Section 2.5

## Description

When housing demand greatly exceeds supply and affordable housing is unavailable, informal settlements spontaneously appear. Informal settlements: no building permits, no utilities, low quality, high density, fire risk, disease risk, crime risk. Can appear on unzoned land near jobs. Formalization mechanic: player can upgrade informal settlements by providing utilities and services, converting to official residential. If ignored, informal settlements grow.

## Definition of Done

- [ ] Informal settlement spawning when housing deficit > 20%
- [ ] Informal buildings on unzoned land near employment
- [ ] No utilities, low quality, high risk
- [ ] Fire risk +50% in informal areas
- [ ] Disease risk +30% in informal areas
- [ ] Formalization tool: add utilities, upgrade to official
- [ ] Growth mechanic: informal settlement expands if not addressed
- [ ] Visual distinction (different mesh/color)

## Test Plan

- Unit test: housing deficit triggers informal settlement
- Unit test: formalization provides utilities
- Integration test: city with housing crisis develops slums

## Pitfalls

- Sensitive topic; handle with respect for real-world slum residents
- Must be preventable with proactive housing policy

## Relevant Code

- `crates/simulation/src/buildings.rs`
- `crates/simulation/src/homelessness.rs`

# CIT-019: Schelling Segregation Model

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** CIT-001 (demographics)
**Source:** social_agent_simulation.md Section 6

## Description

Implement Schelling's segregation model for emergent neighborhood formation. Citizens have a tolerance threshold (0.0-1.0) for the proportion of different-group neighbors. When proportion of different-group neighbors exceeds tolerance, citizen becomes "dissatisfied" and seeks to relocate to a more compatible neighborhood. Even mild preferences (tolerance 0.7) produce strong segregation. Groups defined by income class and/or ethnicity.

## Definition of Done

- [ ] Tolerance threshold per citizen (from Personality, default 0.5-0.8)
- [ ] Neighborhood composition sampled from spatial grid (chunk-level)
- [ ] Dissatisfaction flag when neighbor composition exceeds tolerance
- [ ] Dissatisfied citizens added to relocation queue
- [ ] Relocation prefers chunks with compatible composition
- [ ] Segregation index metric tracked (0 = integrated, 1 = fully segregated)
- [ ] Overlay visualization showing neighborhood ethnic/income composition

## Test Plan

- Unit test: tolerance 0.3 with 50% different neighbors = dissatisfied
- Unit test: tolerance 0.8 with 50% different neighbors = satisfied
- Integration test: starting from random placement, segregation index increases over time
- Integration test: neighborhoods visibly cluster by income/ethnicity after 50 game-years

## Pitfalls

- Segregation system must run infrequently (every 20s) to avoid performance issues
- Relocation must respect housing availability and affordability
- Must not force relocation of citizens who cannot afford alternatives

## Relevant Code

- `crates/simulation/src/citizen.rs` (Personality, position)
- `crates/simulation/src/movement.rs` (DestinationCache)
- `crates/simulation/src/districts.rs` (district composition)

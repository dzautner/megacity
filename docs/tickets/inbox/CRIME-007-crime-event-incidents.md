# CRIME-007: Discrete Crime Incidents and Response

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** CRIME-001 (crime types), SVC-003 (vehicle dispatch)
**Source:** social_agent_simulation.md Section 8.3

## Description

Generate discrete crime incidents from crime probability grid. Each incident has: location (grid cell), type (from CRIME-001), severity (minor/moderate/serious), response_needed (bool). Police vehicles dispatched to respond. Response time affects: case resolution probability, neighborhood fear, deterrence. Unreported crimes (proportion based on police trust) don't generate incidents. Crime incidents visible on map as alert icons.

## Definition of Done

- [ ] `CrimeIncident` event struct (location, type, severity, timestamp)
- [ ] Incident generation rate from crime grid probability
- [ ] Police dispatch to incident location
- [ ] Response time calculation from nearest police station via road network
- [ ] Incident resolution probability based on response time
- [ ] Unresolved incidents increase local crime rate
- [ ] Crime incident icons on map (brief display)
- [ ] Event journal entries for serious crimes

## Test Plan

- Unit test: high crime cell generates more incidents per game-day
- Unit test: faster response = higher resolution probability
- Integration test: police station near crime area reduces incidents

## Pitfalls

- Too many incidents overwhelm display; need rate limiting and filtering
- Crime incidents require service vehicle dispatch system (SVC-003)

## Relevant Code

- `crates/simulation/src/crime.rs`
- `crates/simulation/src/services.rs`

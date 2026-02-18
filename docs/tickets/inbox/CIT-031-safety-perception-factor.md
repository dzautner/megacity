# CIT-031: Happiness Factor -- Safety Perception

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** CRIME-001 (crime pipeline)
**Source:** social_agent_simulation.md Section 5.2

## Description

Safety perception happiness component: based on crime grid at home and work locations, fire coverage, disaster recent occurrence. Formula: safety = 1.0 - crime_exposure*0.5 - recent_disaster_penalty*0.2 + police_coverage*0.2 + fire_coverage*0.1. Crime exposure = average of crime at home cell and work cell, normalized 0-1. Weight in overall happiness: 0.10.

## Definition of Done

- [ ] Crime exposure computed from home and work cell crime levels
- [ ] Recent disaster penalty (decays over 30 game-days)
- [ ] Police coverage bonus at home location
- [ ] Fire coverage bonus at home location
- [ ] Combined safety formula
- [ ] Weight of 0.10 in overall happiness
- [ ] Replace flat CRIME_PENALTY_MAX

## Test Plan

- Unit test: zero crime + full police + fire = ~0.8 safety
- Unit test: max crime + no police = ~0.0 safety
- Unit test: recent disaster provides temporary penalty

## Pitfalls

- Crime grid values are u8 (0-255); normalize carefully
- Disaster penalty needs game-day tracking (not tick-based)

## Relevant Code

- `crates/simulation/src/happiness.rs` (CRIME_PENALTY_MAX, line 169)
- `crates/simulation/src/crime.rs` (CrimeGrid)
- `crates/simulation/src/disasters.rs` (ActiveDisaster)

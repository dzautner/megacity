# CIT-071: Disaster Preparedness Mechanics

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** SVC-011 (emergency management)
**Source:** master_architecture.md Section 1.14

## Description

Disaster severity modified by preparedness level. Earthquake: fault line mapping (terrain feature), building codes reduce damage by 50%, emergency shelters reduce casualties by 30%. Flood: flood plain mapping, levees reduce flood area, elevation advantage. Fire: firebreak effectiveness, sprinkler code reduces building fire spread. Tornado: warning system reduces casualties by 40%, shelter access. Disaster insurance: premiums collected, payouts after disaster, reduces financial impact.

## Definition of Done

- [ ] Building code policy: reduces earthquake/fire damage
- [ ] Flood plain mapping overlay
- [ ] Levee building type for flood prevention
- [ ] Warning system for tornado/storm
- [ ] Disaster insurance system (premiums + payouts)
- [ ] Preparedness score per disaster type
- [ ] Recovery grants after disaster (federal aid)
- [ ] Disaster history tracking

## Test Plan

- Unit test: building codes reduce earthquake damage by 50%
- Unit test: levees reduce flood area
- Unit test: insurance pays out after disaster
- Integration test: prepared city recovers faster from disaster

## Pitfalls

- Current disaster system is random; should be more terrain-aware
- Insurance costs must be meaningful (not free protection)

## Relevant Code

- `crates/simulation/src/disasters.rs`
- `crates/simulation/src/fire.rs`

# CIT-096: Era Progression System

**Priority:** T3 (Differentiation)
**Complexity:** High (5-7 person-weeks)
**Dependencies:** CIT-086 (data-driven parameters)
**Source:** master_architecture.md Section 1.20

## Description

City progresses through eras: Settlement (0-500 pop), Town (500-5K), City (5K-50K), Metropolis (50K-500K), Megacity (500K+). Each era unlocks: new building types, new policies, new service tiers, visual style changes. Era transitions are milestone events with celebration. Available technologies change with era (no highway at Settlement, no metro at Town). Visual era progression: building styles evolve.

## Definition of Done

- [ ] `CityEra` enum with 5 stages
- [ ] Era transition conditions (population thresholds)
- [ ] Per-era building unlocks
- [ ] Per-era policy unlocks
- [ ] Per-era service tier availability
- [ ] Visual style changes per era
- [ ] Era transition event and celebration
- [ ] Current era displayed in UI

## Test Plan

- Unit test: era transitions at correct population thresholds
- Unit test: buildings locked until appropriate era
- Integration test: city progresses through eras during gameplay

## Pitfalls

- Era system must not feel like arbitrary gates; transitions should feel earned
- Must not break existing unlocks system

## Relevant Code

- `crates/simulation/src/unlocks.rs` (UnlockState)
- `crates/simulation/src/events.rs` (MilestoneTracker)

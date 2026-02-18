# CIT-094: Mega-Projects (Endgame Goals)

**Priority:** T4 (Polish)
**Complexity:** High (5-7 person-weeks)
**Dependencies:** CIT-078 (construction materials)
**Source:** master_architecture.md Section 1.20

## Description

Aspirational endgame construction projects. Arcology (self-contained city-building, 50,000 residents), Space Elevator (trade bonus), World's Fair (massive tourism event), Olympic Stadium (prestige + event). Each mega-project requires: massive budget, specific prerequisites (education level, population size, specialization), multi-stage construction (foundation, structure, finishing), years to complete. Completing mega-project is a game achievement.

## Definition of Done

- [ ] 4-6 mega-project types defined
- [ ] Prerequisites per project (population, budget, specialization)
- [ ] Multi-stage construction (3-5 stages, months each)
- [ ] Resource requirements per stage
- [ ] Completion achievement and city-wide bonus
- [ ] Mega-project visible on map (large structure)
- [ ] Progress tracking in UI

## Test Plan

- Unit test: prerequisites checked before construction starts
- Unit test: multi-stage construction progresses correctly
- Unit test: completion applies bonus
- Integration test: mega-project visible and functional

## Pitfalls

- Mega-projects must feel worthwhile (not just expensive vanity)
- Construction time should be long but progress visible

## Relevant Code

- `crates/simulation/src/achievements.rs`
- `crates/simulation/src/events.rs`

# SVC-008: Death Care Capacity and Cemetery Fill

**Priority:** T2 (Depth)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 3.6

## Description

Cemeteries have finite capacity (1000 per acre for traditional burial). When full, city needs new cemetery land or crematorium (unlimited capacity, higher energy cost). Unprocessed deceased reduce happiness and health in neighborhood. Cemetery land is permanent (can't rezone), creating interesting land-use tension. Crematorium requires 3 hours per cremation, gas utility. Current death_care.rs exists but needs capacity limits.

## Definition of Done

- [ ] Cemetery capacity tracking (plots used / total)
- [ ] Cemetery fill rate based on death rate
- [ ] Full cemetery can't accept new burials (overflow to next nearest)
- [ ] No available death care = happiness and health penalty in area
- [ ] Crematorium: unlimited long-term capacity but per-cremation time cost
- [ ] Cemetery land permanently occupied (can't bulldoze/rezone)
- [ ] Warning notification when cemetery reaches 80% capacity

## Test Plan

- Unit test: cemetery fills at expected rate given death rate
- Unit test: full cemetery redirects to alternate facility
- Unit test: no death care available applies health penalty
- Integration test: aging population fills cemeteries faster

## Pitfalls

- Cemetery permanence means player must plan ahead; poor placement wastes valuable land
- Must handle case where all cemeteries are full and no crematorium exists

## Relevant Code

- `crates/simulation/src/death_care.rs` (DeathCareGrid, DeathCareStats)
- `crates/simulation/src/services.rs` (Cemetery, Crematorium)

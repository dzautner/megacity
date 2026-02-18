# INFRA-065: Fix LifeSimTimer Serialization
**Priority:** T0
**Complexity:** S (hours)
**Dependencies:** none
**Source:** master_architecture.md, M2; MEMORY.md known issues

## Description
Fix the known bug where `LifeSimTimer` is not serialized. On load, all life events (aging, marriage, children, death) fire simultaneously because the timer resets to zero. Serialize the timer's remaining duration and elapsed time per citizen. This is a critical save/load bug.

## Definition of Done
- [ ] `LifeSimTimer` serialized in save file per citizen
- [ ] On load, timers resume from saved state
- [ ] No burst of life events on load
- [ ] Tests pass

## Test Plan
- Unit: Save with timer at 50% elapsed, load, verify timer resumes at 50%
- Integration: Load a save, verify no mass death/marriage/birth events in first tick

## Pitfalls
- Timer may be a Bevy `Timer` component; need custom serialization
- Citizens added after a save point need fresh timers (handled by spawner)

## Relevant Code
- `crates/simulation/src/life_simulation.rs` -- LifeSimTimer
- `crates/save/src/serialization.rs` -- citizen serialization

# INFRA-064: Autosave with Configurable Interval
**Priority:** T1
**Complexity:** S (hours)
**Dependencies:** INFRA-063
**Source:** master_architecture.md, M2

## Description
Implement autosave that triggers at configurable intervals (default 5 minutes). Autosave writes to rotating slots (3 slots, overwriting oldest). Runs on a timer, does not block gameplay (async write if possible). Notification shown when autosave occurs. Player can configure interval or disable autosave in settings.

## Definition of Done
- [ ] Autosave timer with configurable interval
- [ ] 3 rotating autosave slots
- [ ] Non-blocking save (or at least fast enough to not stutter)
- [ ] UI notification on autosave
- [ ] Settings to configure interval or disable
- [ ] Tests pass

## Test Plan
- Unit: Autosave triggers after configured interval
- Unit: Slot rotation overwrites oldest save
- Integration: Loading from autosave restores correct game state

## Pitfalls
- Save during simulation tick may capture inconsistent state; save between ticks
- Large save files may cause frame drops; profile and optimize

## Relevant Code
- `crates/save/src/lib.rs` -- save function
- `crates/app/src/main.rs` -- timer system registration

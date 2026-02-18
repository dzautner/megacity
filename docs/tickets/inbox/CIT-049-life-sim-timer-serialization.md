# CIT-049: Fix LifeSimTimer Serialization

**Priority:** T1 (Core -- bugfix)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.17, MEMORY.md Known Issues

## Description

`LifeSimTimer` is not serialized, causing all life events (marriage, children, divorce, education advancement, personality evolution) to fire immediately on game load. This is a known bug listed in MEMORY.md. Fix by adding LifeSimTimer to save data with its current tick values. On load, restore timer state so life events continue from where they left off.

## Definition of Done

- [ ] `LifeSimTimer` fields added to SaveData struct
- [ ] Timer values serialized on save
- [ ] Timer values restored on load
- [ ] Life events do not fire immediately after load
- [ ] Round-trip test: save -> load -> verify timer state preserved

## Test Plan

- Unit test: LifeSimTimer serializes/deserializes correctly
- Integration test: save -> load -> no life event spam
- Regression test: new game still starts with correct timer defaults

## Pitfalls

- Save migration needed for old saves without LifeSimTimer data
- Default values for missing timer data must be safe (not trigger events)

## Relevant Code

- `crates/simulation/src/life_simulation.rs` (LifeSimTimer or equivalent)
- `crates/save/src/serialization.rs` (SaveData)

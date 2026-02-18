# SAVE-002: Autosave with Configurable Interval
**Priority:** T1
**Complexity:** S
**Dependencies:** SAVE-001
**Source:** master_architecture.md, section M2

## Description
Implement periodic autosave that saves game state without player action. Configurable interval (default 5 minutes). Rotate autosave slots to prevent overwriting a single save.

- Autosave every N minutes (configurable: 1, 2, 5, 10, 15 minutes)
- Keep 3 rotating autosave slots (autosave_1, autosave_2, autosave_3)
- Autosave should not freeze gameplay (async or fast enough to be imperceptible)
- Option to disable autosave
- Autosave notification ("City saved") briefly displayed

## Definition of Done
- [ ] Autosave triggers on configurable interval
- [ ] 3 rotating save slots
- [ ] Save operation fast enough to not freeze game
- [ ] Autosave toggle and interval in settings
- [ ] Visual notification on autosave

## Test Plan
- Integration: Wait for autosave interval, verify save file created
- Integration: Verify 4th autosave overwrites oldest slot

## Pitfalls
- Save operation blocks simulation -- must be fast or async
- If save fails (disk full, permissions), must not crash
- Autosave slot naming must be predictable for load menu

## Relevant Code
- `crates/save/src/lib.rs` -- autosave timer and rotation
- `crates/ui/src/toolbar.rs` -- autosave notification
- `crates/app/src/main.rs` -- settings for autosave

# CIT-073: Autosave System with Slot Rotation

**Priority:** T2 (Depth)
**Complexity:** Low (1 person-week)
**Dependencies:** None
**Source:** master_architecture.md Section 1.17

## Description

Automatic save at configurable interval (default: every 5 minutes). 3 rotating autosave slots (overwrites oldest). Autosave indicator in UI (brief "saving..." text). Autosave disabled during disasters or major events (to prevent saving in unrecoverable state). Manual save always available. Autosave settings in options menu.

## Definition of Done

- [ ] Autosave timer (configurable, default 5 min)
- [ ] 3 rotating save slots (autosave_1, autosave_2, autosave_3)
- [ ] UI indicator during autosave
- [ ] Disabled during active disasters
- [ ] Settings: interval, enable/disable, slot count
- [ ] Load menu shows autosave slots with timestamp

## Test Plan

- Unit test: autosave triggers at configured interval
- Unit test: slots rotate correctly (oldest overwritten)
- Unit test: autosave disabled during disaster
- Integration test: autosave files created and loadable

## Pitfalls

- Autosave must not cause frame stutter; consider async IO
- Must not autosave while in middle of player action (road placement)

## Relevant Code

- `crates/save/src/lib.rs` (SaveGameEvent, handle_save)

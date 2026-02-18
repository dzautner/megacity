# SAVE-003: Fix LifeSimTimer Serialization
**Priority:** T0
**Complexity:** S
**Dependencies:** none
**Source:** master_architecture.md, section M2 (known issue)

## Description
The `LifeSimTimer` is not serialized, causing all life events (aging, marriage, children, death) to fire simultaneously on save load. This is a known bug documented in MEMORY.md.

- Add LifeSimTimer to save/load serialization
- Preserve timer state (current tick count, last fire time)
- On load, resume timer from saved state instead of starting at 0
- Stagger events if timer was at 0 to prevent simultaneous fire

## Definition of Done
- [ ] LifeSimTimer serialized in save file
- [ ] Loading save resumes timer from saved state
- [ ] No burst of life events on load
- [ ] Backward compatible with old saves (default timer value if missing)

## Test Plan
- Integration: Save, load, verify no mass aging/death/marriage event burst
- Unit: LifeSimTimer round-trips through serialization

## Pitfalls
- Old saves won't have LifeSimTimer -- need default fallback
- Timer state may be out of sync with citizen ages (need reconciliation?)

## Relevant Code
- `crates/simulation/src/life_simulation.rs` -- LifeSimTimer definition
- `crates/save/src/serialization.rs` -- add to save/load

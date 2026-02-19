# SAVE-003: Fix LifeSimTimer Serialization
**Priority:** T0
**Complexity:** S
**Dependencies:** none
**Source:** master_architecture.md, section M2 (known issue)
**Status:** IMPLEMENTED

## Description
The `LifeSimTimer` is not serialized, causing all life events (aging, marriage, children, death) to fire simultaneously on save load. This is a known bug documented in MEMORY.md.

- Add LifeSimTimer to save/load serialization
- Preserve timer state (current tick count, last fire time)
- On load, resume timer from saved state instead of starting at 0
- Stagger events if timer was at 0 to prevent simultaneous fire

## Definition of Done
- [x] LifeSimTimer serialized in save file
- [x] Loading save resumes timer from saved state
- [x] No burst of life events on load
- [x] Backward compatible with old saves (default timer value if missing)

## Test Plan
- Integration: Save, load, verify no mass aging/death/marriage event burst
- Unit: LifeSimTimer round-trips through serialization

## Pitfalls
- Old saves won't have LifeSimTimer -- need default fallback
- Timer state may be out of sync with citizen ages (need reconciliation?)

## Relevant Code
- `crates/simulation/src/lifecycle.rs` -- LifecycleTimer definition (note: ticket originally referenced `life_simulation.rs` but the actual type is `LifecycleTimer` in `lifecycle.rs`)
- `crates/save/src/serialization.rs` -- add to save/load
- `crates/save/src/lib.rs` -- wire up in save/load/new-game handlers

## Implementation Notes

### Naming
The ticket referenced `LifeSimTimer` but the actual type is `LifecycleTimer` in `crates/simulation/src/lifecycle.rs`. It has two fields:
- `last_aging_day: u32` -- tracks the last game day aging was processed
- `last_emigration_tick: u32` -- tracks the emigration check interval counter

### Changes Made

**`crates/save/src/serialization.rs`:**
- Added `use simulation::lifecycle::LifecycleTimer` import
- Added `SaveLifecycleTimer` struct with `last_aging_day` and `last_emigration_tick` fields, implementing `Default` for backward compatibility
- Added `lifecycle_timer: Option<SaveLifecycleTimer>` field to `SaveData` with `#[serde(default)]` for backward compat with old saves
- Updated `create_save_data()` to accept `lifecycle_timer: Option<&LifecycleTimer>` parameter and serialize it
- Added `restore_lifecycle_timer()` function to reconstruct `LifecycleTimer` from saved data
- Added `test_lifecycle_timer_roundtrip` unit test
- Updated existing tests (roundtrip, v2_full, backward_compat) to include the new parameter and assertions

**`crates/save/src/lib.rs`:**
- Added `restore_lifecycle_timer` to imports from serialization
- Added `use simulation::lifecycle::LifecycleTimer` import
- `handle_save`: Added `lifecycle_timer: Res<LifecycleTimer>` parameter; passes `Some(&lifecycle_timer)` to `create_save_data`
- `handle_load`: Added `mut lifecycle_timer: ResMut<LifecycleTimer>` parameter; restores from saved data or sets `last_aging_day = clock.day` for old saves (prevents immediate aging burst)
- `handle_new_game`: Added `mut lifecycle_timer: ResMut<LifecycleTimer>` parameter; resets to `LifecycleTimer::default()`

### Backward Compatibility (Old Saves)
When loading an old save that lacks `lifecycle_timer`, the load handler sets `last_aging_day` to the current `clock.day`. This prevents the aging system from immediately firing (since `clock.day < timer.last_aging_day + AGING_INTERVAL_DAYS` will be true). The `last_emigration_tick` is set to 0, which is safe since emigration only fires every 30 ticks.

### Build and Test Results
- `cargo check` -- compiles successfully (no errors)
- `cargo test -p save` -- 14/14 tests pass including the new `test_lifecycle_timer_roundtrip`

# SAVE-007: Serialize LifecycleTimer

## Priority: T1 (Short-Term Fix)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Issue 1: LifecycleTimer Not Serialized

## Description
On load, `last_aging_day` and `last_emigration_tick` default to 0, causing all life events to fire immediately. Add these fields to `SaveData` with fallback to `clock.day`.

## Acceptance Criteria
- [ ] `SaveData` includes `lifecycle_last_aging_day: Option<u32>` and `lifecycle_last_emigration_tick: Option<u32>`
- [ ] On load, uses `unwrap_or(clock.day)` for old saves
- [ ] No burst of aging/emigration on load

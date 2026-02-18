# SAVE-003: Serialize Citizen Gender

## Priority: T1 (Short-Term Fix)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Issue 3: Citizen Details Data Loss

## Description
Gender is currently reconstructed from `age % 2` on load, which changes on birthday and is incorrect half the time. Add `gender: u8` to `SaveCitizen`.

## Acceptance Criteria
- [ ] `SaveCitizen` has `gender: u8` field
- [ ] Gender roundtrips correctly through save/load
- [ ] Old saves without gender field fall back to current behavior

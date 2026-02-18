# SAVE-004: Serialize Citizen Health

## Priority: T1 (Short-Term Fix)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Issue 3: Citizen Details Data Loss

## Description
Health is hardcoded to 80.0 on load. A citizen at 20% health loads as healthy. Add `health: f32` to `SaveCitizen`.

## Acceptance Criteria
- [ ] `SaveCitizen` has `health: f32` field
- [ ] Health roundtrips correctly through save/load
- [ ] Old saves default to 80.0

# SAVE-006: Serialize Citizen Savings

## Priority: T1 (Short-Term Fix)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Issue 3: Citizen Details Data Loss

## Description
Savings is hardcoded to `salary * 2.0` on load. A citizen with $50K savings loads with $3K. Add `savings: f32` to `SaveCitizen`.

## Acceptance Criteria
- [ ] `SaveCitizen` has `savings: f32` field
- [ ] Savings roundtrips correctly
- [ ] Old saves default to `salary * 2.0`

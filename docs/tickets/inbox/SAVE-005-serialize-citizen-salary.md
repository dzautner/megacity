# SAVE-005: Serialize Citizen Salary

## Priority: T1 (Short-Term Fix)
## Effort: Small (0.5 day)
## Source: save_system_architecture.md -- Issue 3: Citizen Details Data Loss

## Description
Salary is recalculated from education level on load, losing job-match modifiers. Add `salary: f32` to `SaveCitizen`.

## Acceptance Criteria
- [ ] `SaveCitizen` has `salary: f32` field
- [ ] Salary roundtrips correctly
- [ ] Old saves recalculate from education

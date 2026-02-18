# MOD-025: Extract Citizen Behavior Parameters to Data Files

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Citizen parameters (commute tolerance, aging rate, death probability by age, education duration, salary ranges) are hardcoded. Extract to data files.

## Acceptance Criteria
- [ ] `CitizenConfig` struct: commute_tolerance, age_rate, salary_by_education, etc.
- [ ] `assets/data/citizens.ron` with all citizen parameters
- [ ] Life simulation reads from data file
- [ ] Modders can adjust citizen behavior

# ECON-012: Citizen Income Brackets and Wealth
**Priority:** T3
**Complexity:** M
**Dependencies:** none
**Source:** economic_simulation.md, section 1.2; master_architecture.md, section 3

## Description
Add income/wealth tracking to citizens. Currently citizens have no economic state beyond employment status. Citizens should have an income level that affects housing choice, spending patterns, and tax contribution.

- IncomeBracket enum: Low, LowerMiddle, Middle, UpperMiddle, High
- Income determined by: education level, job type, workplace zone type, experience (age)
- Citizens seek housing matching their income bracket
- High-income citizens pay more tax, spend more at commercial, expect better services
- Income distribution: Low 10%, LowerMiddle 25%, Middle 30%, UpperMiddle 25%, High 10%
- Immigration income bracket determined by city attractiveness to that bracket

## Definition of Done
- [ ] IncomeBracket on citizen component
- [ ] Income determined by education and job
- [ ] Citizens prefer housing matching their bracket
- [ ] Income affects tax contribution
- [ ] Income distribution visible in demographics panel

## Test Plan
- Unit: University-educated citizen in office zone = UpperMiddle or High income
- Unit: Uneducated citizen in industrial zone = Low or LowerMiddle income
- Integration: Build high-education pipeline, verify income distribution shifts upward

## Pitfalls
- Must not require all income tracking to be precise (simplified model is fine)
- Income bracket mismatch with housing creates homelessness or vacancies
- Immigration system needs bracket-aware logic

## Relevant Code
- `crates/simulation/src/citizen.rs:Citizen` -- add income bracket
- `crates/simulation/src/education.rs` -- education affects income
- `crates/simulation/src/citizen_spawner.rs` -- income bracket assignment
- `crates/simulation/src/immigration.rs` -- bracket-aware immigration

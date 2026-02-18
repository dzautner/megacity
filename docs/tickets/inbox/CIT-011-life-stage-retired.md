# CIT-011: Life Stage -- Retired (65+) Aging and End-of-Life

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 2.2, historical_demographics_services.md Section 2.2

## Description

Retired (65+) citizens no longer work. Income from savings/pension only. Highest healthcare demand (5x base). Death probability rises with age using Gompertz curve: P(death|age) = 0.0001 * exp(0.085 * age). Current system uses linear probability after 70; replace with Gompertz. Nursing home demand for 80+. Social isolation risk for widowed seniors.

## Definition of Done

- [ ] Retired citizens draw from savings, no salary
- [ ] Pension system: base pension + years_worked * contribution_rate
- [ ] Healthcare utilization 5x base rate
- [ ] Gompertz mortality curve replacing linear death probability
- [ ] Nursing home demand for 80+ with low health
- [ ] Social isolation penalty for widowed/childless seniors
- [ ] Death properly triggers death care system

## Test Plan

- Unit test: Gompertz curve yields <1% death at 65, ~5% at 80, ~50% at 95
- Unit test: pension correctly calculated from work history
- Integration test: aging population increases healthcare demand visibly

## Pitfalls

- Current death check in lifecycle.rs starts at 70, not 65; must align
- Gompertz can produce extreme values at high ages; clamp to 1.0

## Relevant Code

- `crates/simulation/src/citizen.rs` (LifeStage::Retired, line 23)
- `crates/simulation/src/lifecycle.rs` (age_citizens, death check lines 43-66)
- `crates/simulation/src/death_care.rs` (death processing)

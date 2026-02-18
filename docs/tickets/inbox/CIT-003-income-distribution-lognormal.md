# CIT-003: Log-Normal Income Distribution with Gini Tracking

**Priority:** T3 (Differentiation)
**Complexity:** Low-Medium (1-2 person-weeks)
**Dependencies:** CIT-002 (education levels)
**Source:** social_agent_simulation.md Section 1.1 Income Distribution

## Description

Replace flat salary assignment with log-normal income generation. Income = education_multiplier * CITY_MEDIAN_INCOME * experience_bonus, drawn from log-normal distribution with sigma=0.4. Track city-wide Gini coefficient (0.0=equality, 1.0=inequality). US cities typically 0.40-0.55. Gini feeds into crime motivation, social mobility, and political systems.

## Definition of Done

- [ ] `generate_income()` function using log-normal distribution
- [ ] Experience bonus: 2% per year of experience
- [ ] City-level Gini coefficient computed and stored as a resource
- [ ] Gini updated periodically (every ~100 ticks)
- [ ] Gini displayed in city stats UI
- [ ] Income class derived from generated income (6 tiers with thresholds)

## Test Plan

- Unit test: 10K generated incomes follow log-normal shape (KS test or histogram check)
- Unit test: Gini of perfectly equal incomes = 0.0
- Unit test: Gini of one-person-has-all = ~1.0
- Integration test: city Gini settles in 0.35-0.55 range with default parameters

## Pitfalls

- Log-normal can generate extreme outliers; clamp to reasonable range
- Gini computation is O(n log n) due to sorting; use sampling for 100K+ citizens

## Relevant Code

- `crates/simulation/src/citizen.rs` (CitizenDetails.salary)
- `crates/simulation/src/wealth.rs` (WealthStats, existing Gini tracking)
- `crates/simulation/src/life_simulation.rs` (salary_payment)

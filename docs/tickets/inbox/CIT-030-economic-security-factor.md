# CIT-030: Happiness Factor -- Economic Security

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** CIT-003 (income distribution)
**Source:** social_agent_simulation.md Section 5.2

## Description

Economic security happiness component: employment_score (employed=0.7, unemployed=0.0, good job match=1.0), income_adequacy (salary / cost_of_living), savings_buffer (months of expenses in savings). Formula: econ_security = employment*0.4 + income_adequacy*0.35 + savings_buffer*0.25. Job match quality: education match +0.3, overqualified -0.1. Weight in overall happiness: 0.15.

## Definition of Done

- [ ] Employment score: 0.0 (unemployed), 0.7 (employed), 1.0 (good match)
- [ ] Income adequacy: salary / estimated cost of living
- [ ] Savings buffer: months_of_savings = savings / monthly_expenses
- [ ] Job match quality bonus/penalty
- [ ] Combined economic security formula
- [ ] Weight of 0.15 in overall happiness
- [ ] Replace flat EMPLOYED_BONUS

## Test Plan

- Unit test: employed with good match and savings = ~0.9
- Unit test: unemployed with no savings = ~0.0
- Unit test: employed but overqualified = ~0.6

## Pitfalls

- Cost of living not currently modeled; use city average rent + expenses
- Must not penalize retirees for unemployment

## Relevant Code

- `crates/simulation/src/happiness.rs` (EMPLOYED_BONUS, line 155)
- `crates/simulation/src/citizen.rs` (salary, savings)

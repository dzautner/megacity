# CIT-092: Savings Accumulation and Depletion Model

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 1.1

## Description

Enhance savings model. Monthly savings change = salary - expenses - rent - taxes. Expenses = base_cost_of_living * (1 + lifestyle_inflation). Lifestyle inflation from materialism personality. Savings depleted during unemployment. Savings buffer metric = savings / monthly_expenses. Buffer < 3 months = financial stress. Buffer = 0 + unemployment = homelessness risk. Retirement savings needed for pension (savings at retirement / years_remaining).

## Definition of Done

- [ ] Monthly savings delta calculation
- [ ] Cost of living estimate per income class
- [ ] Lifestyle inflation from materialism
- [ ] Financial stress at buffer < 3 months
- [ ] Homelessness risk at buffer = 0 + no income
- [ ] Retirement adequacy metric
- [ ] Savings visible in citizen detail panel

## Test Plan

- Unit test: employed citizen accumulates savings
- Unit test: unemployed citizen depletes savings
- Unit test: buffer < 3 months triggers stress
- Integration test: economic downturn depletes savings across city

## Pitfalls

- Savings already exists on CitizenDetails; enhance model, don't replace field

## Relevant Code

- `crates/simulation/src/citizen.rs` (CitizenDetails.savings)
- `crates/simulation/src/life_simulation.rs` (salary_payment)

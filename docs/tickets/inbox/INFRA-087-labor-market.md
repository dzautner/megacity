# INFRA-087: Labor Market with Wage Determination
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-074
**Source:** master_architecture.md, M4

## Description
Implement labor market simulation. Wages determined by supply and demand: more workers than jobs -> wages fall; labor shortage -> wages rise. Education level affects job eligibility. Skilled workers earn more. Minimum wage policy option. Wages affect citizen happiness, spending, and housing affordability. Unemployment rate tracked as key metric.

## Definition of Done
- [ ] Wage calculation from labor supply/demand ratio
- [ ] Education-based job eligibility
- [ ] Skilled vs unskilled wage differential
- [ ] Minimum wage policy
- [ ] Unemployment rate metric
- [ ] Wages affect happiness and spending
- [ ] Tests pass

## Test Plan
- Unit: Labor shortage (more jobs than workers) -> wages increase
- Unit: Minimum wage prevents wages below threshold
- Integration: Education investment leads to higher average wages over time

## Pitfalls
- Wage-price spiral: higher wages -> higher costs -> higher prices -> demand for higher wages
- Current `education_jobs.rs` has some job matching; extend
- Must interact with immigration (high wages attract immigrants)

## Relevant Code
- `crates/simulation/src/education_jobs.rs` -- job matching
- `crates/simulation/src/economy.rs` -- wage economy
- `crates/simulation/src/wealth.rs` -- citizen income

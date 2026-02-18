# SVC-020: Service Budget Framework (Realistic Proportions)

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 6.5

## Description

Municipal budget should follow real-world proportions: Police 25-35%, Fire/EMS 10-15%, Public works 15-25%, Education 15-30%, Healthcare 5-10%, Parks 3-5%, Libraries 1-2%, Social services 5-10%, Administration 5-10%, Debt service 5-15%. Revenue: property tax 30-45%, sales tax 15-25%, income tax 10-20%, user fees 10-20%, transfers 10-20%. Per capita spending target: $1,500-$4,000. Player must force trade-offs between competing departments.

## Definition of Done

- [ ] Department budget allocation UI
- [ ] Recommended budget proportions as guidelines
- [ ] Over/under-funding effects per department
- [ ] Revenue source breakdown (property, sales, income tax)
- [ ] Per capita spending metric
- [ ] Budget deficit/surplus tracking
- [ ] Budget affects service quality (already partially in ExtendedBudget)
- [ ] Realistic expense scaling with city size

## Test Plan

- Unit test: per capita spending in $1500-4000 range at default settings
- Unit test: cutting police budget reduces crime prevention
- Integration test: balanced budget maintains all services

## Pitfalls

- ExtendedBudget already has ServiceBudgets; enhance rather than replace
- Budget proportions should be flexible (player choice) not rigid

## Relevant Code

- `crates/simulation/src/budget.rs` (ExtendedBudget, ServiceBudgets)
- `crates/simulation/src/economy.rs` (CityBudget)

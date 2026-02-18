# INFRA-054: Detailed Budget Categories
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-053
**Source:** master_architecture.md, M2

## Description
Expand budget with separate income/expense lines for each service type. Income: property tax, commercial tax, transit fares, parking revenue, loans. Expenses: police, fire, education, health, road maintenance, transit operations, water/sewer, power, garbage, parks, administration. Each category separately trackable and adjustable. Budget breakdown panel in UI showing all categories with bars/graphs.

## Definition of Done
- [ ] `BudgetCategory` enum with 10+ income and 12+ expense categories
- [ ] Per-category tracking in `Budget` resource
- [ ] Category-level spending adjustment (percentage slider)
- [ ] Budget breakdown UI panel with bar charts
- [ ] Monthly/yearly budget summary
- [ ] Tests pass

## Test Plan
- Unit: All expenses sum to total expenses; all income sums to total income
- Unit: Cutting police budget to 50% reduces police coverage
- Integration: Budget panel shows all categories with meaningful values

## Pitfalls
- Too many categories overwhelms player; group into sections (safety, utilities, transport)
- Budget cuts should affect service quality proportionally
- Deficit spending should trigger warnings and loan suggestions

## Relevant Code
- `crates/simulation/src/budget.rs` -- budget resource
- `crates/simulation/src/economy.rs` -- revenue/expense collection
- `crates/ui/src/info_panel.rs` -- budget UI

# ECON-002: Detailed Budget Categories
**Priority:** T1
**Complexity:** M
**Dependencies:** ECON-001
**Source:** master_architecture.md, section M2; cities_skylines_analysis.md, section 3

## Description
Expand budget tracking to show separate income and expense lines for each category. Current IncomeBreakdown has residential/commercial/industrial/office tax + trade. Need finer granularity and proper expense categories.

Income categories:
- Residential property tax
- Commercial property tax
- Industrial property tax
- Office property tax
- Tourism revenue
- Transit fare revenue (future)
- Toll revenue (future)
- Park entry fees (future)

Expense categories:
- Road maintenance (per road cell)
- Fire department
- Police department
- Healthcare
- Education (elementary + high school + university)
- Sanitation (garbage, recycling)
- Parks & recreation
- Public transit (future)
- Policy costs
- Loan payments
- Administrative overhead (scales with city size)

## Definition of Done
- [ ] All income categories tracked separately
- [ ] All expense categories tracked separately
- [ ] Service budgets from ServiceBudgets scale actual expense
- [ ] Budget panel UI shows full breakdown
- [ ] Monthly balance clearly displayed

## Test Plan
- Unit: Each service type maps to correct expense category
- Integration: Build fire station, verify fire expense category increases
- Integration: Budget panel shows correct breakdown summing to total

## Pitfalls
- Expense per service building already exists (ServiceBuilding::monthly_maintenance)
- Need to aggregate by ServiceType category, not individual building
- Service budget level (0-1.5) should scale effectiveness AND cost

## Relevant Code
- `crates/simulation/src/budget.rs:IncomeBreakdown` -- expand fields
- `crates/simulation/src/budget.rs:ExpenseBreakdown` -- expand fields
- `crates/simulation/src/economy.rs:collect_taxes` -- populate all categories
- `crates/ui/src/info_panel.rs` -- budget breakdown display

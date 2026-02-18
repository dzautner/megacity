# UI-001: Budget Breakdown Panel
**Priority:** T1
**Complexity:** M
**Dependencies:** ECON-002
**Source:** master_architecture.md, section M2

## Description
Create comprehensive budget panel showing all income and expense categories with current values, percentages, and trends. Player needs clear visibility into city finances.

- Income section: residential tax, commercial tax, industrial tax, office tax, tourism, transit fares, tolls, park fees
- Expense section: road maintenance, fire, police, healthcare, education, sanitation, parks, transit, policies, loans
- Net income/expense with positive/negative coloring
- Historical chart: last 12 months of income vs expenses
- Budget slider controls: per-service funding level (0-150%)
- Tax rate sliders: per-zone and per-income-bracket

## Definition of Done
- [ ] Budget panel shows all categories
- [ ] Service budget sliders functional
- [ ] Tax rate sliders functional
- [ ] Historical trend chart
- [ ] Net balance clearly displayed

## Test Plan
- Integration: All budget categories sum to total income/expense
- Integration: Changing service budget slider affects expense and service effectiveness

## Pitfalls
- info_panel.rs needs significant expansion
- Must not overwhelm with numbers -- use progressive disclosure (summary -> detail)
- Real-time updates as tax/budget changes applied

## Relevant Code
- `crates/ui/src/info_panel.rs` -- budget panel UI
- `crates/simulation/src/budget.rs:ExtendedBudget` -- data source
- `crates/simulation/src/economy.rs` -- income/expense data

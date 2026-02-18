# MOD-005: Extract Economy Constants to Data Files

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Tax rates, income modifiers, expense categories, and economic cycle parameters are hardcoded. Extract to data files.

## Acceptance Criteria
- [ ] `EconomyConfig` struct: base_tax_rates, expense_categories, cycle_params
- [ ] `assets/data/economy.ron` with all economy constants
- [ ] Economy system reads from data file
- [ ] Balance changes possible without recompilation

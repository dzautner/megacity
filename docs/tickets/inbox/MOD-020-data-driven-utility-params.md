# MOD-020: Extract Utility Parameters to Data Files

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Utility parameters (power plant output, water tower capacity, costs, coverage range) are hardcoded. Extract to data files.

## Acceptance Criteria
- [ ] `UtilityDef` struct: type, capacity, cost, maintenance, output, range
- [ ] `assets/data/utilities.ron` with all utility definitions
- [ ] Utility placement and capacity calculations use data file values

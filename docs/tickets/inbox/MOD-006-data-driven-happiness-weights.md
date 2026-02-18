# MOD-006: Extract Happiness Weights to Data Files

## Priority: T2 (Depth)
## Effort: Small (1 day)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Happiness factor weights (commute, services, pollution, crime, noise, etc.) are hardcoded. Extract to data files for easy balancing.

## Acceptance Criteria
- [ ] `HappinessConfig` struct: factor weights, thresholds, penalties
- [ ] `assets/data/happiness.ron` with all happiness weights
- [ ] `update_happiness` reads from config resource
- [ ] Modders can adjust happiness balance

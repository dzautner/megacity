# MOD-030: Extract Tourism Parameters to Data Files

## Priority: T2 (Depth)
## Effort: Small (0.5 day)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Tourism parameters (attraction values, seasonal multipliers, tourist behavior) are hardcoded. Extract to data files.

## Acceptance Criteria
- [ ] `TourismConfig` struct: attraction_values, seasonal_multipliers
- [ ] `assets/data/tourism.ron` with tourism parameters
- [ ] Tourism system reads from data file

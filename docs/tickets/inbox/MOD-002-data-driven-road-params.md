# MOD-002: Extract Road Parameters to Data Files

## Priority: T2 (Depth)
## Effort: Medium (2-3 days)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Road widths, speeds, capacities, costs, and maintenance rates are hardcoded in `RoadType::half_width()` and elsewhere. Extract to data files.

## Acceptance Criteria
- [ ] `RoadDef` struct: width, speed_limit, capacity, cost, maintenance, lanes
- [ ] `assets/data/roads.ron` with all road type definitions
- [ ] `RoadType::half_width()` reads from data file
- [ ] Road cost calculation uses data file values

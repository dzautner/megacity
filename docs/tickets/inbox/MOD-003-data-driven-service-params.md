# MOD-003: Extract Service Parameters to Data Files

## Priority: T2 (Depth)
## Effort: Medium (2-3 days)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Service radii, costs, capacities, and effects are hardcoded. Extract to data files. Includes fire, police, health, education, garbage, death care, postal.

## Acceptance Criteria
- [ ] `ServiceDef` struct: type, radius, cost, maintenance, capacity, happiness_bonus
- [ ] `assets/data/services.ron` with all service definitions
- [ ] `ServiceBuilding::coverage_radius()` reads from data file
- [ ] Service placement cost uses data file values

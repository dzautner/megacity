# MOD-021: Extract Zone Parameters to Data Files

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Zone type parameters (demand modifiers, max building level, growth rate, compatible building types) are hardcoded. Extract to data files.

## Acceptance Criteria
- [ ] `ZoneDef` struct: type, max_level, demand_modifiers, compatible_buildings
- [ ] `assets/data/zones.ron` with all zone definitions
- [ ] Building spawner uses zone data for level limits
- [ ] Modders can add new zone types via data files

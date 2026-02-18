# MOD-022: Extract Disaster Parameters to Data Files

## Priority: T2 (Depth)
## Effort: Small (1 day)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Disaster parameters (probability, damage radius, duration, damage per tick) are hardcoded. Extract to data files.

## Acceptance Criteria
- [ ] `DisasterDef` struct: type, probability, radius, duration, damage_per_tick
- [ ] `assets/data/disasters.ron` with all disaster definitions
- [ ] Disaster system reads from data file

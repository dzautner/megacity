# MOD-024: Extract Milestone/Unlock Parameters to Data Files

## Priority: T2 (Depth)
## Effort: Small (1 day)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Milestone thresholds (population targets, unlock requirements, achievement conditions) are hardcoded. Extract to data files.

## Acceptance Criteria
- [ ] `MilestoneDef` struct: name, population_threshold, unlocks
- [ ] `assets/data/milestones.ron` with all milestone definitions
- [ ] Unlock system reads from data file
- [ ] Achievement conditions defined in data

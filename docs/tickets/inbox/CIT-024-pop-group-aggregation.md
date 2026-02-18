# CIT-024: Pop-Group Aggregation for Statistical LOD

**Priority:** T2 (Depth)
**Complexity:** Medium-High (3 person-weeks)
**Dependencies:** CIT-023 (behavioral LOD)
**Source:** social_agent_simulation.md Section 15.2

## Description

For 80K+ citizens in Statistical tier, aggregate into ~200-1000 PopGroups defined by (chunk, income_class, education). Each group stores: count, average_happiness, average_health, employment_rate, average_income, crime_exposure, service_coverage, emigration_pressure. Systems operate on groups instead of individuals. Re-aggregation runs every ~50s. This is the key performance trick enabling 100K+ citizen simulation.

## Definition of Done

- [ ] `PopulationGroups` resource with Vec<PopGroup>
- [ ] `PopGroupKey` = (chunk_x, chunk_y, income_class, education)
- [ ] Re-aggregation system every 500 ticks
- [ ] Statistical happiness computed per group (not per citizen)
- [ ] Group-level emigration pressure
- [ ] Employment rate per group
- [ ] Migration between groups when citizens change income/education

## Test Plan

- Unit test: 10K citizens produce ~100-500 groups
- Unit test: group average_happiness matches individual average within 5%
- Performance test: 100K citizens with pop-groups < 1ms per tick
- Integration test: city stats derived from groups match entity-derived stats

## Pitfalls

- Groups must be rebuilt when citizens move between chunks
- Empty groups should be pruned
- Group stats must be consistent with individual entity state

## Relevant Code

- `crates/simulation/src/lod.rs`
- `crates/simulation/src/virtual_population.rs`

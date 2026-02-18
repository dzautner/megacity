# CIT-023: Behavioral LOD System (Full/Simplified/Statistical)

**Priority:** T2 (Depth)
**Complexity:** High (4-5 person-weeks)
**Dependencies:** None (extends existing LOD)
**Source:** social_agent_simulation.md Section 15.2

## Description

Extend existing visual LOD (Full/Simplified/Abstract) with behavioral LOD. Full (~500-2000 citizens near camera): full pathfinding, individual decision making, thought stack, schedule execution. Simplified (~5K-15K visible but distant): lerp movement, probability-table decisions, current mood only. Statistical (~80K+ off-screen): no individual movement, pop-group level simulation (Victoria 3 style), aggregate happiness from neighborhood conditions. LOD assigned by camera distance. Transitions preserve state.

## Definition of Done

- [ ] `BehaviorLOD` enum (Full, Simplified, Statistical)
- [ ] LOD assignment based on camera distance (Full < 512 units, Simplified < 2048, Statistical beyond)
- [ ] Full tier: existing individual simulation
- [ ] Simplified tier: lerp movement, probability tables, no pathfinding
- [ ] Statistical tier: pop-group aggregation, no per-entity iteration
- [ ] PopulationGroups resource for statistical tier
- [ ] State preservation on LOD transitions (no data loss)
- [ ] Per-system LOD filtering (skip Statistical in movement, skip Full in pop-group)

## Test Plan

- Unit test: LOD correctly assigned based on distance
- Unit test: citizen transitioning Full->Statistical->Full preserves happiness
- Integration test: 100K citizens with LOD stays above 10 FPS
- Integration test: city stats consistent regardless of camera position

## Pitfalls

- LOD transitions cause observable behavior changes; minimize visible artifacts
- Pop-group aggregation must stay synchronized with individual entities
- Camera pan during pathfinding could cause mid-path LOD change

## Relevant Code

- `crates/simulation/src/lod.rs` (existing visual LOD)
- `crates/simulation/src/virtual_population.rs` (VirtualPopulation)
- `crates/simulation/src/movement.rs` (pathfinding)

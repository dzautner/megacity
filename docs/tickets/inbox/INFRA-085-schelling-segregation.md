# INFRA-085: Schelling Segregation Model for Neighborhoods
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-084
**Source:** master_architecture.md, M4

## Description
Implement Schelling segregation model: citizens have a mild preference for neighbors similar to themselves (income class, education level). Even mild preferences (30% same-type threshold) produce strong segregation patterns. Citizens above threshold stay; below threshold seek to move. This creates emergent neighborhood character without scripting.

## Definition of Done
- [ ] Neighbor similarity metric per citizen (income class, education level)
- [ ] Satisfaction threshold (configurable, default 30% similar neighbors)
- [ ] Unsatisfied citizens seek to relocate
- [ ] Segregation patterns emerge organically
- [ ] Segregation index metric in stats
- [ ] Tests pass

## Test Plan
- Unit: Area with 80% similar neighbors: citizen stays
- Unit: Area with 10% similar neighbors: citizen seeks to move
- Integration: After N game-years, distinct neighborhoods emerge

## Pitfalls
- Segregation is politically sensitive; frame as "neighborhood character" formation
- Too-strong segregation makes city management difficult; tune threshold carefully
- Citizens moving frequently is expensive; rate-limit moves

## Relevant Code
- `crates/simulation/src/citizen.rs` -- citizen demographics
- `crates/simulation/src/wealth.rs` -- income class
- `crates/simulation/src/lifecycle.rs` -- citizen movement/relocation

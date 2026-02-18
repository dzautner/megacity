# INFRA-102: Mega-Projects (Aspirational Goals)
**Priority:** T4
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Implement mega-projects as late-game aspirational goals. Examples: Space Elevator (huge cost, massive tourism), Arcology (self-contained city-in-a-building), Fusion Power Plant (unlimited clean energy), Hyperloop Hub (inter-city travel). Each mega-project requires population threshold, research investment, and massive construction cost. Provides unique gameplay bonuses and serves as win conditions.

## Definition of Done
- [ ] Mega-project framework: prerequisites, cost, construction time, bonuses
- [ ] At least 3 mega-projects defined
- [ ] Multi-phase construction (partial completion visible)
- [ ] Unique bonuses per mega-project
- [ ] Mega-project appears on map as landmark building
- [ ] Tests pass

## Test Plan
- Unit: Mega-project cannot start without prerequisites met
- Unit: Construction deducts correct budget over construction period
- Integration: Completed mega-project provides expected bonuses

## Pitfalls
- Mega-projects must not make the game trivially easy after completion
- Construction period must be meaningful (not instant)
- Need unique mesh/rendering for each mega-project

## Relevant Code
- `crates/simulation/src/buildings.rs` -- special building types
- `crates/simulation/src/unlocks.rs` -- prerequisite checking

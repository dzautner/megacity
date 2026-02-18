# INFRA-133: River Deltas and Estuaries
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-006, INFRA-008
**Source:** procedural_terrain.md, Section 3.5

## Description
Generate river deltas where rivers meet the ocean and estuaries where rivers widen near the coast. Deltas: river splits into multiple channels (distributaries) spreading sediment. Estuaries: wide, shallow water bodies where fresh and salt water mix. Both create interesting terrain features for city building (port potential, fertile land, flood risk).

## Definition of Done
- [ ] Delta detection where river meets ocean
- [ ] River channel splitting into 2-4 distributaries
- [ ] Estuary widening near coast
- [ ] Delta/estuary areas marked as fertile land
- [ ] Port placement favored in estuary areas
- [ ] Tests pass

## Test Plan
- Unit: River terminating at ocean creates delta feature
- Unit: Delta cells marked as fertile

## Pitfalls
- Delta generation on 256x256 grid may look too small; adjust scale
- Delta changes river mouth significantly; may affect existing water cells

## Relevant Code
- `crates/simulation/src/terrain.rs` -- delta/estuary generation

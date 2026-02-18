# INFRA-017: Player Terrain Modification (Flatten/Raise/Lower)
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-016
**Source:** procedural_terrain.md, Section 8.3

## Description
Allow players to modify terrain elevation using flatten, raise, and lower tools. Flatten sets a region to a target elevation (average of selection). Raise/lower adjusts by a delta. Terrain modification has a cost proportional to volume of earth moved. Modified cells need mesh regeneration. Store original elevation for cost calculation.

## Definition of Done
- [ ] Flatten tool: sets region to average elevation
- [ ] Raise/lower tool: adjusts by configurable delta
- [ ] Cost proportional to volume changed (sum of |old_elevation - new_elevation|)
- [ ] Terrain mesh regenerates for affected chunks
- [ ] Modified elevations serialize/deserialize correctly
- [ ] Tests pass

## Test Plan
- Unit: Flatten a 5x5 area, verify all cells at same elevation
- Unit: Cost calculation matches expected volume

## Pitfalls
- Terrain modification can strand buildings above/below grade
- Need to re-check building eligibility and road connectivity after modification
- Slope recalculation needed after modification

## Relevant Code
- `crates/rendering/src/input.rs` -- new terrain tools
- `crates/simulation/src/terrain.rs` -- modification functions
- `crates/rendering/src/terrain_render.rs` -- chunk mesh regeneration

# CIT-048: Spatial Hash for Citizen Neighbor Queries

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 15.4

## Description

Spatial hash grid for O(1) citizen neighbor queries. Required for: Schelling model (neighbor composition), disease spread (proximity contact), social networks (nearby citizens), crime (victim proximity). Hash cell size = 64 world units (4x4 grid cells). Rebuilt every tick from citizen positions. Provides `citizens_in_radius(pos, radius)` query. Replaces brute-force scans.

## Definition of Done

- [ ] `CitizenSpatialHash` resource
- [ ] Cell size 64 world units
- [ ] Rebuilt every tick from citizen positions
- [ ] `citizens_in_radius(pos, radius) -> Vec<Entity>` method
- [ ] `citizens_in_cell(x, y) -> &[Entity]` method
- [ ] Used by: segregation, disease, social networks
- [ ] Performance: O(1) lookups, O(n) rebuild

## Test Plan

- Unit test: citizen at (100, 100) found in correct cell
- Unit test: radius query returns only citizens within distance
- Performance test: 100K citizen rebuild < 2ms
- Integration test: disease spread uses spatial hash

## Pitfalls

- SpatialIndex already exists on DestinationCache; evaluate reuse vs new structure
- Hash rebuild every tick may be expensive; consider incremental updates

## Relevant Code

- `crates/simulation/src/movement.rs` (DestinationCache, SpatialIndex)
- `crates/simulation/src/citizen.rs` (Position)

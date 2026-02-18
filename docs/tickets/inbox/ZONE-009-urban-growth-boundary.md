# ZONE-009: Urban Growth Boundary
**Priority:** T3
**Complexity:** S
**Dependencies:** none
**Source:** urban_planning_zoning.md, section 6.4

## Description
Allow player to draw an urban growth boundary (UGB) on the map. Development is restricted outside the boundary. This encourages density within the boundary and prevents sprawl. Modeled after Portland, Oregon's UGB system.

- Player draws boundary line on the map (polygon)
- Cells outside boundary cannot be zoned (except agricultural/rural if those zone types exist)
- Existing buildings outside boundary remain but cannot be upgraded
- Boundary can be expanded by player at any time
- Land value inside boundary gets slight premium (scarcity drives up prices)
- Land value outside boundary drops (development restriction reduces value)

## Definition of Done
- [ ] Player can draw UGB polygon on map
- [ ] Zoning tool blocked for cells outside UGB
- [ ] Land value differential exists inside vs outside boundary
- [ ] Boundary expandable

## Test Plan
- Integration: Draw UGB, attempt to zone outside, verify failure
- Integration: Verify land value inside UGB > outside after boundary set

## Pitfalls
- Polygon inside/outside test needed (point-in-polygon algorithm)
- Must not break existing cities without UGB (default = no boundary = everything allowed)
- UGB interaction with districts (can a district span the boundary?)

## Relevant Code
- `crates/simulation/src/policies.rs` -- UGB as city-wide policy
- `crates/rendering/src/input.rs` -- UGB drawing tool
- `crates/simulation/src/land_value.rs` -- UGB boundary premium

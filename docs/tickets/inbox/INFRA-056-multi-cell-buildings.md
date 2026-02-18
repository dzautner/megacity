# INFRA-056: Multi-Cell Buildings (2x2 and Larger)
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-055
**Source:** master_architecture.md, M3

## Description
Support buildings that occupy multiple grid cells (2x2 minimum, up to 4x4 for landmarks). Multi-cell buildings have a primary cell (anchor) and secondary cells. Building spawner must check all cells are same zone type, unbuildable, and have compatible elevation. Rendering uses single mesh spanning multiple cells. Multi-cell buildings have higher capacity and visual impact than single-cell.

## Definition of Done
- [ ] `BuildingFootprint` enum or field: 1x1, 2x2, 3x3, 4x4
- [ ] Building spawner checks all cells in footprint are eligible
- [ ] Secondary cells marked as occupied (no double-building)
- [ ] Rendering: single mesh covers footprint area
- [ ] Demolition removes all cells of multi-cell building
- [ ] Tests pass

## Test Plan
- Unit: 2x2 building cannot spawn if one cell is occupied
- Unit: Demolishing any cell of a multi-cell building removes the whole building
- Integration: High-level commercial zone shows 2x2 buildings

## Pitfalls
- Pathfinding must treat all cells as occupied/impassable
- Save/load must serialize the anchor cell and mark secondaries
- Building spawner iterates grid; multi-cell check adds O(footprint_size) per candidate

## Relevant Code
- `crates/simulation/src/buildings.rs` -- building_spawner, Building component
- `crates/rendering/src/building_render.rs` -- mesh positioning
- `crates/rendering/src/building_meshes.rs` -- larger meshes

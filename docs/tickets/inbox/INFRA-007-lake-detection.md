# INFRA-007: Lake Detection and Filling
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-006
**Source:** procedural_terrain.md, Section 3.2

## Description
Detect local elevation minima (depressions) in the heightmap and fill them to create lakes. Use flood-fill from the lowest point upward until water reaches an overflow elevation (the lowest saddle point on the depression rim). Mark all filled cells as `CellType::Water`. Lakes provide aesthetic value and affect land value for nearby properties.

## Definition of Done
- [ ] Depression detection algorithm finds closed basins
- [ ] Flood-fill creates lakes with correct water level
- [ ] Lake cells marked as `CellType::Water` with elevation set to water surface level
- [ ] Multiple lakes can exist on one map
- [ ] Tests pass

## Test Plan
- Unit: Create a bowl-shaped heightmap, verify lake fills to rim height
- Unit: Verify no infinite loops when multiple depressions overlap

## Pitfalls
- All cells on a flat map are a single depression; need minimum depth threshold
- Lakes near map edges should drain off-map rather than filling

## Relevant Code
- `crates/simulation/src/terrain.rs` -- lake detection
- `crates/simulation/src/grid.rs` -- `CellType::Water`

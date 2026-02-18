# INFRA-008: Ocean and Coastline Generation
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 3.3

## Description
Support coastal maps where one or more map edges are ocean. Use a sea level parameter; cells below sea level become ocean `CellType::Water`. Coastline should be irregular (not a straight line) using noise-based edge masking. Support configurable coast side (north, south, east, west, or combinations). Create beaches (narrow band of low-slope cells at coast edge).

## Definition of Done
- [ ] Sea level configurable parameter
- [ ] Coastal maps have irregular shoreline
- [ ] Beach/shore zone detectable (cells within N distance of water-land boundary)
- [ ] Map generation seed produces consistent coastlines
- [ ] Tests pass

## Test Plan
- Unit: With sea_level=0.35, verify ~35% of cells are water
- Integration: Coastal map is visually appealing with natural shoreline

## Pitfalls
- Uniform sea level can flood river valleys entirely; rivers should be generated before ocean fill
- Tiny isolated water cells look wrong; filter out water bodies < 10 cells

## Relevant Code
- `crates/simulation/src/terrain.rs` -- sea level application
- `crates/simulation/src/grid.rs` -- `CellType::Water`

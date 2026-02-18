# BLDG-002: Multi-Cell Buildings (2x2, 3x3, 4x4)
**Priority:** T2
**Complexity:** XL
**Dependencies:** BLDG-001
**Source:** urban_planning_zoning.md, section 2.1-2.4; master_architecture.md, section 1.4

## Description
Currently all buildings occupy 1 grid cell. Large buildings (apartment towers, malls, factories) should occupy 2x2, 3x3, or 4x4 cells. This creates more realistic city morphology.

- Add `footprint_width` and `footprint_height` fields to Building component
- Grid allocation: when spawning multi-cell building, claim all cells and set building_id on each
- All claimed cells must be same zone type, road-adjacent (at least one cell), and free
- Demolition must clear all claimed cells
- Building mesh generation scales to footprint size
- High-density R-4 L3+: 2x2 cells. I-3: 3x3 to 5x5. C-3 (big box): 4x4+. C-4 L4+: 2x2

## Definition of Done
- [ ] Buildings can occupy 2x2, 3x3, or 4x4 cell footprints
- [ ] Grid correctly marks all cells with building_id
- [ ] Demolition clears all cells
- [ ] Building spawner finds contiguous available zones for multi-cell
- [ ] Building meshes scale to footprint
- [ ] Save/load handles multi-cell buildings

## Test Plan
- Unit: 2x2 building marks 4 cells with same entity
- Unit: Cannot spawn 2x2 if any cell is occupied/wrong zone/water
- Integration: Zone 4x4 area, verify large building spawns
- Integration: Demolish multi-cell building, verify all cells cleared

## Pitfalls
- Contiguous cell finding algorithm is non-trivial (flood fill in zoned area)
- Building_id on multiple cells means deletion must find ALL cells for an entity
- Pathfinding to multi-cell buildings: which cell is the "entrance"? Use road-adjacent cell
- Performance: building spawner scans for rectangular free areas each tick

## Relevant Code
- `crates/simulation/src/buildings.rs:Building` -- add footprint fields
- `crates/simulation/src/buildings.rs:building_spawner` -- multi-cell allocation
- `crates/simulation/src/grid.rs:WorldGrid` -- helper for contiguous free area search
- `crates/rendering/src/building_meshes.rs` -- scaled mesh generation
- `crates/rendering/src/building_render.rs` -- multi-cell placement

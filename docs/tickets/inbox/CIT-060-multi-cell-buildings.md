# CIT-060: Multi-Cell Buildings (2x2, 3x3, 4x4)

**Priority:** T2 (Depth)
**Complexity:** High (4-5 person-weeks)
**Dependencies:** CIT-059 (five-level buildings)
**Source:** master_architecture.md Section 1.4

## Description

High-level buildings occupy multiple grid cells. Level 3+: 2x2 plots. Level 4+: 3x3 plots. Level 5: up to 4x4 plots. Grid allocation system finds contiguous free cells of correct zone type. All cells in multi-cell building share building_id. Rendering uses single mesh covering multiple cells. Higher capacity per plot cell than single-cell (density bonus). Multi-cell buildings look like proper skyscrapers/complexes.

## Definition of Done

- [ ] Grid allocation for contiguous NxN free cells
- [ ] All cells in multi-cell building linked to same building entity
- [ ] Building.footprint_size field (1, 2, 3, or 4)
- [ ] Capacity: footprint^2 * base_capacity * density_bonus
- [ ] Rendering: single mesh covering NxN cells
- [ ] Bulldozing multi-cell building frees all cells
- [ ] Zone demand considers multi-cell availability
- [ ] Corner-lot buildings for irregularly-shaped plots

## Test Plan

- Unit test: 2x2 building occupies all 4 cells
- Unit test: insufficient contiguous space prevents multi-cell spawn
- Unit test: bulldoze frees all cells
- Integration test: level 4+ buildings visibly larger

## Pitfalls

- Finding contiguous cells is O(n^2) in worst case; use eligible-cell list
- Multi-cell buildings near grid edge need boundary checks

## Relevant Code

- `crates/simulation/src/buildings.rs` (building_spawner)
- `crates/simulation/src/grid.rs` (WorldGrid, Cell.building_id)
- `crates/rendering/src/building_meshes.rs`

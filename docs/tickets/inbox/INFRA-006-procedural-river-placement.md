# INFRA-006: Procedural River Placement with Width Variation
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-004
**Source:** procedural_terrain.md, Section 3.1, 3.4

## Description
After erosion, detect natural drainage paths and widen them into rivers. Find cells where accumulated water flow exceeds a threshold (from erosion particle paths or flow accumulation analysis). Set those cells to `CellType::Water` with width proportional to accumulated flow. Support river meanders via slight randomized offsets. Rivers should have variable width (1-4 cells for streams to major rivers).

## Definition of Done
- [ ] Flow accumulation grid computed from heightmap gradients
- [ ] Cells exceeding flow threshold marked as `CellType::Water`
- [ ] River width varies with accumulated flow (narrow upstream, wide downstream)
- [ ] At least one continuous river path exists per generated map
- [ ] Tests pass

## Test Plan
- Unit: On a tilted plane, flow accumulation produces a single drainage channel
- Integration: Generated maps have visually coherent river systems

## Pitfalls
- Flow accumulation in flat areas creates sheet flow rather than channels; add small random perturbation
- Rivers must connect to map edge or lake/ocean; orphan rivers look wrong
- Current WATER_THRESHOLD=0.35 logic in terrain.rs may conflict; unify the approaches

## Relevant Code
- `crates/simulation/src/terrain.rs` -- river detection and marking
- `crates/simulation/src/grid.rs` -- `CellType::Water`, `Cell.elevation`

# INFRA-012: Slope Effects on Construction and Roads
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 7.1

## Description
Calculate slope for each cell from elevation differences with neighbors. Slope affects: road construction cost (steeper = more expensive), building placement eligibility (max buildable slope ~15%), vehicle speed reduction on steep grades, and erosion/landslide risk. Store slope as a derived field per cell. Add `Cell.slope: f32` to grid.

## Definition of Done
- [ ] Slope computed per cell from 8-neighbor elevation differences
- [ ] Slope stored in `Cell` struct or as a separate `SlopeGrid`
- [ ] Road construction cost multiplied by slope factor (1.0 at flat, up to 3.0 at max buildable)
- [ ] Buildings cannot be placed on cells with slope > 0.15 (15%)
- [ ] Vehicle speed reduced by slope factor in pathfinding
- [ ] Tests pass

## Test Plan
- Unit: Flat terrain has slope ~0, steep terrain has slope proportional to elevation difference
- Unit: Building spawner rejects cells above slope threshold
- Integration: Roads cost more on hillsides

## Pitfalls
- Slope calculation at grid edges needs boundary handling
- Slope affects both construction and operational costs -- separate the two
- `neighbors8()` already exists for 8-connectivity

## Relevant Code
- `crates/simulation/src/grid.rs` -- `Cell` struct, `neighbors8()`
- `crates/simulation/src/buildings.rs` -- building_spawner placement checks
- `crates/simulation/src/road_segments.rs` -- construction cost

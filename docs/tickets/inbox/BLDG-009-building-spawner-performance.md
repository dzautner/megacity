# BLDG-009: Building Spawner Performance Optimization
**Priority:** T1
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section 5.1 (known issue)

## Description
The current building_spawner iterates the full 256x256 grid (65,536 cells) for each zone type (6 types = 393K iterations per spawn tick). Replace with an eligible-cell list that tracks zoned cells without buildings.

- Maintain `EligibleCells` resource: Vec<(usize, usize)> of cells that are zoned, have no building, adjacent to road, have power+water
- Update list when: cell zoned/unzoned, building placed/demolished, utility coverage changes
- Building spawner iterates eligible list instead of full grid
- Shuffle eligible list periodically to avoid spatial bias
- Expected improvement: O(eligible) instead of O(grid_size * zone_types)

## Definition of Done
- [ ] EligibleCells resource maintained incrementally
- [ ] Building spawner uses eligible list
- [ ] Same spawn behavior as before (correctness preserved)
- [ ] Measurable performance improvement for cities with < 50% zoned area

## Test Plan
- Unit: Adding zone to cell adds it to eligible list
- Unit: Placing building removes cell from eligible list
- Integration: Spawn behavior identical to full-grid scan (regression test)
- Benchmark: Compare spawn tick time before/after

## Pitfalls
- Must update eligible list on ALL relevant changes (zone, building, road, utility)
- Race condition if utility propagation runs after eligible list update
- Shuffling needed to prevent deterministic spatial patterns

## Relevant Code
- `crates/simulation/src/buildings.rs:building_spawner` -- replace grid scan
- `crates/simulation/src/grid.rs` -- integration points for list maintenance
- `crates/rendering/src/input.rs` -- zone painting updates eligible list

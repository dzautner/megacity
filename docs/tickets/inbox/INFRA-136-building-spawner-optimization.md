# INFRA-136: Building Spawner Eligible-Cell List Optimization
**Priority:** T1
**Complexity:** S (hours)
**Dependencies:** none
**Source:** MEMORY.md known issues

## Description
The `building_spawner` iterates the full 256x256 grid per zone type to find eligible cells. Replace with a maintained list of eligible cells per zone type. When zoning changes or buildings are placed/demolished, update the eligible cell list. This reduces O(65536 * zone_types) per tick to O(eligible_cells).

## Definition of Done
- [ ] `EligibleCellList` resource per zone type
- [ ] List updated on zone change, building placement, building demolition
- [ ] Building spawner reads from list instead of iterating grid
- [ ] Performance improvement measurable
- [ ] Tests pass

## Test Plan
- Unit: After zoning 100 cells, eligible list contains those 100 cells
- Unit: After building placed on cell, cell removed from eligible list
- Integration: Building spawning behavior unchanged, just faster

## Pitfalls
- Must handle all events that change eligibility (zone change, demolition, road removal)
- List must stay synchronized with grid state; stale entries cause bugs
- Consider using a set instead of Vec for O(1) removal

## Relevant Code
- `crates/simulation/src/buildings.rs` -- `building_spawner` system
- `crates/simulation/src/zones.rs` -- zone change events

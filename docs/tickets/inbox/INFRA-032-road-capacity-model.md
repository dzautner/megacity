# INFRA-032: Road Capacity Model per Road Type
**Priority:** T1
**Complexity:** S (hours)
**Dependencies:** none
**Source:** transportation_simulation.md, Section 1.3

## Description
Define capacity per lane per hour for each `RoadType` based on Highway Capacity Manual values. Local: 600 veh/lane/hr, Avenue (collector): 700, Boulevard (arterial): 800, Highway: 1800, OneWay: 900, Path: 0 (no vehicles). Total road capacity = capacity_per_lane * lane_count * direction_count. Add `capacity_per_lane()` and `total_capacity()` methods to `RoadType`. This is the denominator for V/C ratio calculations.

## Definition of Done
- [ ] `RoadType::capacity_per_lane()` returns HCM-based capacity
- [ ] `RoadType::total_capacity()` accounts for lane count and directionality
- [ ] Capacity values documented with HCM source references
- [ ] Used by BPR function (INFRA-030) for V/C computation
- [ ] Tests pass

## Test Plan
- Unit: Local road (2 lanes) has capacity 1200 veh/hr
- Unit: Highway (4 lanes) has capacity 7200 veh/hr
- Unit: Path has 0 vehicle capacity

## Pitfalls
- `RoadType::half_width()` centralizes road geometry; capacity should follow same pattern
- One-way roads have double capacity in one direction but zero in the other
- Must distinguish direction-specific capacity from bidirectional total

## Relevant Code
- `crates/simulation/src/grid.rs` -- `RoadType` enum, `half_width()`
- `crates/simulation/src/road_graph_csr.rs` -- uses capacity for edge weights

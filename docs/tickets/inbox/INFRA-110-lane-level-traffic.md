# INFRA-110: Lane-Level Traffic Simulation
**Priority:** T5
**Complexity:** XL (1-2 weeks)
**Dependencies:** INFRA-030, INFRA-033
**Source:** master_architecture.md, M6; transportation_simulation.md, Section 7

## Description
Upgrade from link-level to lane-level traffic simulation. Each vehicle tracks which lane it occupies. Lane changes based on upcoming turns and congestion. This solves the "all vehicles in one lane" problem from CS1. Lane assignment at intersections based on turn direction. Lane-change logic: MOBIL model (gap acceptance + incentive criterion). Visual rendering of vehicles in correct lanes.

## Definition of Done
- [ ] Per-lane vehicle tracking
- [ ] Lane change decision model (MOBIL)
- [ ] Lane assignment at intersections based on turn
- [ ] Multi-lane utilization (all lanes used when road is busy)
- [ ] Visual vehicle rendering in correct lane position
- [ ] Tests pass

## Test Plan
- Unit: Vehicle approaching right turn moves to right lane
- Unit: All lanes utilized when traffic is heavy
- Unit: Lane change only when gap is sufficient

## Pitfalls
- Lane-level tracking dramatically increases computation per vehicle
- Need careful performance optimization (spatial partitioning per lane)
- Lane markings rendering needed to make lanes visible

## Relevant Code
- `crates/simulation/src/road_graph_csr.rs` -- lane-level graph
- `crates/simulation/src/movement.rs` -- vehicle movement
- `crates/rendering/src/road_render.rs` -- lane marking rendering

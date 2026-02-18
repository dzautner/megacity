# INFRA-033: Intersection Delay Model
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-032
**Source:** transportation_simulation.md, Section 1.4

## Description
Add intersection delay to pathfinding edge costs. Unsignalized intersections: minor road yields to major road (delay 5-15 seconds). Signalized intersections: average delay from signal timing (10-30 seconds depending on cycle length and green ratio). Roundabouts: lower delay than signals at moderate volumes (5-10 seconds). Delay formula: `d = 0.5 * C * (1-g/C)^2 / (1 - min(1.0, V/C) * g/C)` (Webster's formula). Intersection type determined by road hierarchy at junction.

## Definition of Done
- [ ] Intersection type classification (unsignalized, signalized, roundabout)
- [ ] Delay per intersection type
- [ ] Signal timing parameters (cycle length, green ratio)
- [ ] Intersection delay added to CSR graph edge weights
- [ ] Tests pass

## Test Plan
- Unit: Minor-major intersection has 10-15s average delay
- Unit: Signalized intersection with 60s cycle, 50% green has ~17s delay
- Integration: Routes prefer roads with fewer intersections for long trips

## Pitfalls
- Every road cell is currently a potential intersection; need to identify actual intersection nodes
- Intersection detection from `road_segments.rs` already exists; reuse it
- Webster's formula requires V/C < 1.0; cap at 1.0 and add overflow penalty

## Relevant Code
- `crates/simulation/src/road_graph_csr.rs` -- intersection nodes in CSR graph
- `crates/simulation/src/road_segments.rs` -- intersection detection

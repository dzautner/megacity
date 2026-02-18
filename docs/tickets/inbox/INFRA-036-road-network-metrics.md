# INFRA-036: Road Network Quality Metrics
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** INFRA-032
**Source:** transportation_simulation.md, Section 3.6

## Description
Compute and display road network quality metrics: Connectivity Index (gamma = e / (3*(v-2)), range 0-1, perfect grid ~0.67), Circuity (average network_distance / straight_line_distance, grid ~1.27), Lane-km per capita (dense: 2-4, suburban: 6-10, sprawl: 12-20 per 1000 people), Intersection density (per km2: grid downtown 100-200, suburban 30-60). Display in stats panel and advisor feedback.

## Definition of Done
- [ ] Connectivity index computed from CSR graph (edges/nodes)
- [ ] Circuity computed by sampling OD pairs
- [ ] Lane-km per capita from road network total
- [ ] Intersection density per km2
- [ ] All metrics shown in stats panel
- [ ] Tests pass

## Test Plan
- Unit: Perfect grid network has gamma ~0.67
- Unit: Tree network has gamma < 0.3
- Integration: Stats panel shows meaningful values for typical cities

## Pitfalls
- Circuity requires solving many shortest paths; sample 100-200 random OD pairs for performance
- Metrics need to be recomputed periodically, not every frame

## Relevant Code
- `crates/simulation/src/road_graph_csr.rs` -- graph structure for metrics
- `crates/simulation/src/stats.rs` -- stats display

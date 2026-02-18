# INFRA-030: BPR Travel Time Function in Pathfinding
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** transportation_simulation.md, Section 1.6-1.7; master_architecture.md, M2

## Description
Implement the Bureau of Public Roads (BPR) function for computing congestion-dependent travel time: `t = t_free * (1 + alpha * (V/C)^beta)` where alpha=0.15, beta=4.0 (standard HCM values). Integrate into `csr_find_path_with_traffic()` so that edge weights reflect actual congestion. Each road segment needs `free_flow_time` and `capacity`. Volume/Capacity ratio drives nonlinear delay increase. This replaces the current simple traffic penalty.

## Definition of Done
- [ ] `bpr_travel_time(free_flow_time, volume, capacity, alpha, beta)` function
- [ ] Road capacity derived from `RoadType` and lane count
- [ ] Edge weights in CSR graph use BPR function
- [ ] Pathfinding routes avoid congested roads (higher cost edges)
- [ ] Tests pass

## Test Plan
- Unit: At V/C=0, travel time equals free_flow_time
- Unit: At V/C=1, travel time = t_free * 1.15 (15% increase)
- Unit: At V/C=1.5, travel time = t_free * 1.76 (76% increase)
- Integration: Citizens avoid heavily congested roads in favor of longer but faster routes

## Pitfalls
- V/C > 2.0 produces extreme delays; cap at reasonable maximum
- Volume must be tracked per-edge, not just per-cell
- BPR is a link-level function; need to aggregate cell traffic to road segment level
- Alpha and beta should be configurable for tuning

## Relevant Code
- `crates/simulation/src/road_graph_csr.rs` -- `csr_find_path_with_traffic()`, edge weights
- `crates/simulation/src/traffic.rs` -- traffic density tracking
- `crates/simulation/src/road_segments.rs` -- segment capacity from `RoadType`

# TRAF-001: BPR Travel Time Function in Pathfinding
**Priority:** T1
**Complexity:** M
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 6; master_architecture.md, section M2

## Description
Implement Bureau of Public Roads (BPR) function for travel time estimation. Currently pathfinding uses simple distance-based cost. BPR models congestion nonlinearly: travel_time = free_flow_time * (1 + alpha * (volume/capacity)^beta).

- Standard BPR: alpha=0.15, beta=4.0
- Free flow time = segment_length / road_speed
- Volume = traffic density on road cell
- Capacity = road_type.lane_count() * 1800 vehicles/hour
- Integrate into CSR graph edge weights
- Update edge weights periodically (not every tick -- too expensive)
- Commute time in happiness uses BPR-computed time, not raw distance

## Definition of Done
- [ ] BPR function implemented
- [ ] CSR graph edge weights include congestion cost
- [ ] Pathfinding routes around congested roads
- [ ] Commute time in happiness uses BPR time
- [ ] Congested roads visible in traffic overlay

## Test Plan
- Unit: BPR at 0 volume = free_flow_time
- Unit: BPR at capacity = free_flow_time * 1.15
- Unit: BPR at 2x capacity = free_flow_time * 3.8
- Integration: Create bottleneck, verify traffic routes around it

## Pitfalls
- Updating all edge weights every tick is O(edges) -- use periodic update (every 10 ticks)
- Traffic-aware routing already partially exists (csr_find_path_with_traffic) -- extend, don't replace
- BPR can create instability (all traffic shifts at once) -- needs damping

## Relevant Code
- `crates/simulation/src/road_graph_csr.rs:csr_find_path_with_traffic` -- add BPR weights
- `crates/simulation/src/traffic.rs` -- volume per road cell
- `crates/simulation/src/happiness.rs` -- commute time from BPR

# CIT-062: Level of Service (LOS) Grading for Roads

**Priority:** T2 (Depth)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.2

## Description

Map traffic density to Level of Service grades (A-F). LOS A: free flow (V/C < 0.35). B: stable flow (0.35-0.55). C: stable flow with delays (0.55-0.75). D: approaching unstable (0.75-0.90). E: unstable flow (0.90-1.00). F: breakdown (>1.00). V/C = volume / capacity per road type. BPR travel time function: t = t_free * (1 + 0.15 * (V/C)^4). LOS affects: visual feedback (color-coded roads), traffic overlay, pathfinding edge weights, commute time calculation.

## Definition of Done

- [ ] V/C ratio calculation per road segment
- [ ] LOS grading (A-F) from V/C ratio
- [ ] BPR travel time function for pathfinding
- [ ] Traffic overlay shows LOS colors (green A through red F)
- [ ] Commute time uses BPR function (not static edge weights)
- [ ] LOS statistics per road type
- [ ] Bottleneck detection (segments at LOS E/F)
- [ ] Road capacity per type (from RoadType.lane_count * 1600 veh/hr/lane)

## Test Plan

- Unit test: low traffic = LOS A
- Unit test: V/C = 1.0 = LOS E
- Unit test: BPR function doubles travel time at V/C = 1.0
- Integration test: traffic overlay shows meaningful LOS patterns

## Pitfalls

- Traffic density is u16 per cell; need volume/hour not just count
- BPR function in pathfinding increases path computation cost

## Relevant Code

- `crates/simulation/src/traffic.rs` (TrafficGrid)
- `crates/simulation/src/road_graph_csr.rs` (csr_find_path_with_traffic)
- `crates/rendering/src/overlay.rs` (traffic overlay)

# POLL-015: Road Traffic Vehicle Emission Pollution Source

## Priority: T2 (Depth)

## Description
Implement traffic-based air pollution where each road cell emits pollution proportional to its traffic volume. Line sources (roads) emit lower per-cell Q than point sources but cumulatively dominate in urban areas. Traffic type matters: highway > arterial > local road.

## Current State
- Roads add a flat +2 pollution per road cell regardless of traffic volume.
- No road type distinction for pollution.
- `TrafficGrid` exists and tracks vehicle counts per cell.

## Definition of Done
- [ ] Road emission formula: `Q_road = base_Q * (traffic_volume / road_capacity)`.
- [ ] Base Q by road type: highway=8.0, arterial=4.0, collector=2.0, local=1.0.
- [ ] Traffic scaling: empty road = 0.1x, moderate = 0.5x, congested = 1.0x, over-capacity = 1.2x.
- [ ] EV mandate policy (POLL-004) reduces road Q by 30-60% when active.
- [ ] Road emissions contribute to `AirPollutionGrid` via dispersion.

## Test Plan
- [ ] Unit test: highway at full traffic emits 8x more than empty local road.
- [ ] Unit test: EV mandate reduces emissions by specified percentage.
- [ ] Integration test: busy highway corridor shows elevated air pollution.

## Pitfalls
- Traffic volume may not be available per-cell if `TrafficGrid` is only per-road-segment.
- Must sum line source contributions efficiently (many road cells).
- Night traffic should be lower, reducing emissions.

## Code References
- `crates/simulation/src/pollution.rs`: road pollution
- `crates/simulation/src/traffic.rs`: `TrafficGrid`
- Research: `environment_climate.md` section 1.1.2

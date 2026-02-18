# POLL-002: Per-Building-Type Air Pollution Emission Rates

## Priority: T2 (Depth)

## Description
Implement distinct emission rates (Q values) for each building/source type instead of the current uniform industrial calculation. The research doc defines 18+ source types ranging from Coal Power Plant (Q=100) to Solar Farm (Q=0), including area sources (residential, commercial) and line sources (roads scaled by traffic).

## Current State
- Only industrial buildings emit air pollution (fixed formula: `5 + level*3`).
- Roads add a flat +2 pollution per road cell regardless of type or traffic.
- No emissions from power plants, commercial buildings, residential heating, airports, or seaports.
- No source category distinction (point vs area vs line).

## Definition of Done
- [ ] `PollutionSource` struct with `source_type`, `base_q`, `stack_height`, `grid_x`, `grid_y`.
- [ ] Emission table matching research doc values for all 18+ building types.
- [ ] Road emissions scale with traffic volume: `Q_road = base_Q * (traffic_volume / road_capacity)`.
- [ ] Area sources (residential, commercial) emit low amounts per cell.
- [ ] Source list rebuilt each slow tick from building/service/road queries.
- [ ] Policy multiplier applied per source category (not globally).

## Test Plan
- [ ] Unit test: Coal plant Q=100 > Gas plant Q=35 > Solar Q=0.
- [ ] Unit test: Highway road segment with full traffic emits more than empty road.
- [ ] Integration test: city with only solar/wind power has near-zero air pollution.
- [ ] Integration test: industrial district has higher pollution than commercial district.

## Pitfalls
- Many source types do not exist as placeable buildings yet (Chemical Plant, Refinery, etc.); use placeholder building types or skip missing ones.
- Traffic-based emissions require reading from `TrafficGrid`, creating a dependency.
- Must handle the case where the same cell has multiple sources (building + road).

## Code References
- `crates/simulation/src/pollution.rs`: `update_pollution`
- `crates/simulation/src/buildings.rs`: `Building` component
- `crates/simulation/src/services.rs`: `ServiceBuilding`, `ServiceType`
- `crates/simulation/src/traffic.rs`: `TrafficGrid`
- Research: `environment_climate.md` section 1.1.2

# INFRA-034: Road Hierarchy Enforcement and Bottleneck Warnings
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-031, INFRA-032
**Source:** transportation_simulation.md, Section 3.1-3.3; master_architecture.md, M3

## Description
Implement road hierarchy analysis. Detect violations: local roads carrying arterial-level traffic, highways without collector connections, missing collector links between local and arterial. Generate advisor warnings when hierarchy violations cause bottlenecks. Rules: local roads should carry <1000 veh/day, collectors 1000-8000, arterials 8000-30000, highways 30000+. A local road at 5000 veh/day should trigger "road needs upgrade" warning.

## Definition of Done
- [ ] Road hierarchy tier classification based on `RoadType`
- [ ] Traffic volume thresholds per tier
- [ ] Hierarchy violation detection (local road over threshold)
- [ ] Advisor warnings for bottlenecks and hierarchy violations
- [ ] Visual indicator on roads exceeding their tier threshold
- [ ] Tests pass

## Test Plan
- Unit: Local road with 5000 veh/day triggers violation
- Unit: Highway with 25000 veh/day does not trigger violation
- Integration: Advisor suggests upgrading overloaded local roads

## Pitfalls
- Daily traffic volume must be extrapolated from per-tick counts
- Upgrading road type changes capacity, which changes V/C ratio
- Warning fatigue if too many violations shown; prioritize worst ones

## Relevant Code
- `crates/simulation/src/traffic.rs` -- traffic volume tracking
- `crates/simulation/src/advisors.rs` -- warning system

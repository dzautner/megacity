# SVC-022: Stormwater Management and Flooding

**Priority:** T3 (Differentiation)
**Complexity:** High (4-5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.11

## Description

Stormwater drainage capacity per area. Drainage capacity = storm_drain_coverage * capacity_per_drain. When rainfall * area > drainage_capacity, flooding occurs. Flood damage to buildings, roads, citizen displacement. Flood probability increases with: impervious surface coverage (more buildings = more runoff), low elevation, proximity to water bodies. Green infrastructure (rain gardens, green roofs, bioswales) reduces runoff. Combined sewer systems risk overflow (sewage + stormwater mixing).

## Definition of Done

- [ ] Stormwater capacity per area (from drainage infrastructure)
- [ ] Rainfall runoff calculation (rainfall * impervious_fraction)
- [ ] Flooding when runoff > capacity
- [ ] Flood damage to buildings and roads
- [ ] Citizen displacement from flooded buildings
- [ ] Green infrastructure reduces runoff
- [ ] Flood risk overlay (elevation + impervious + drainage)
- [ ] Storm drain service building type

## Test Plan

- Unit test: heavy rain + no drainage = flooding
- Unit test: green infrastructure reduces runoff by 30%
- Unit test: low elevation floods first
- Integration test: visible flooding during heavy rain events

## Pitfalls

- Flooding requires elevation data (currently terrain is flat y=0)
- Stormwater is complex; start with simplified model

## Relevant Code

- `crates/simulation/src/weather.rs` (precipitation)
- `crates/simulation/src/disasters.rs` (flood events)

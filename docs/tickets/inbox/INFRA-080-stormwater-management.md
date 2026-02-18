# INFRA-080: Stormwater Management and Flood Risk
**Priority:** T3
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-020, INFRA-014
**Source:** master_architecture.md, M4

## Description
Implement stormwater system. Impervious surfaces (roads, buildings) generate runoff. Storm drains collect runoff and route to water bodies. Without adequate drainage, heavy rain causes street flooding. Flood risk = runoff_volume / drainage_capacity. Green infrastructure (permeable pavement, rain gardens, retention ponds) reduces runoff. Separate storm sewer from sanitary sewer optional upgrade.

## Definition of Done
- [ ] Impervious surface tracking per cell
- [ ] Runoff calculation from rain intensity and impervious area
- [ ] Storm drain capacity tracking
- [ ] Flooding when runoff > drainage capacity
- [ ] Green infrastructure options to reduce runoff
- [ ] Stormwater risk overlay
- [ ] Tests pass

## Test Plan
- Unit: 100% impervious area generates more runoff than 50% impervious
- Unit: Storm drain at capacity handles normal rain but overflows in heavy rain
- Integration: Dense downtown floods without storm drains during storms

## Pitfalls
- Complex interaction with terrain elevation (water flows downhill)
- Green infrastructure needs new building/zone types
- Combined sewer overflow is a real-world problem; model it

## Relevant Code
- `crates/simulation/src/weather.rs` -- rain events
- `crates/simulation/src/disasters.rs` -- flood damage
- `crates/simulation/src/groundwater.rs` -- water table interaction

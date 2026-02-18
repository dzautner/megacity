# TRAF-013: Bicycle Lanes and Infrastructure
**Priority:** T2
**Complexity:** M
**Dependencies:** TRAF-007
**Source:** cities_skylines_analysis.md, section 14.1; master_architecture.md, section M3

## Description
Add bicycle infrastructure as a transportation mode. Bike lanes can be part of road types or standalone paths. Citizens choose biking for short-medium trips when infrastructure exists.

- Bike lane road variants: Local+bike, Avenue+bike
- Standalone bike path: separate from road, no vehicles
- Cyclist speed: 15 km/h, practical range ~5km
- Citizens choose biking when: bike infrastructure exists, distance < 5km, age 15-65
- Biking reduces car trips, improves health, reduces traffic
- Encourage Biking policy: +15% cycling rate when bike infrastructure exists
- Bike racks at transit stations enable bike-to-transit mode

## Definition of Done
- [ ] Bike lane road variants available
- [ ] Standalone bike path tool
- [ ] Citizens choose biking in mode choice
- [ ] Cycling reduces car traffic
- [ ] Encourage Biking policy functional

## Test Plan
- Integration: Build bike lanes, verify cycling mode share > 0
- Integration: Remove bike lanes, verify cycling drops to near zero

## Pitfalls
- Bike infrastructure needs to be connected (gaps break the network)
- Cycling in rain/snow should be reduced (weather interaction)
- Bike mode only practical for relatively flat terrain

## Relevant Code
- `crates/simulation/src/grid.rs:RoadType` -- add bike lane variants
- `crates/simulation/src/movement.rs` -- bike mode in mode choice
- `crates/simulation/src/policies.rs` -- Encourage Biking policy

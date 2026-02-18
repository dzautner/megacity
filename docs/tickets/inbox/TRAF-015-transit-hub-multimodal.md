# TRAF-015: Transit Hub / Multi-Modal Stations
**Priority:** T3
**Complexity:** M
**Dependencies:** TRAF-005, TRAF-006
**Source:** cities_skylines_analysis.md, section 14.4

## Description
Implement multi-modal transit stations that combine multiple transit types (bus-metro hub, train-metro hub). These are critical for efficient transit network design as they serve as transfer points.

- Bus-Metro Hub: combined bus stop and metro station
- Train-Metro Hub: combined train station and metro
- Multi-Modal Hub: all transit types at one location
- Transfer penalty reduced at hubs (3 min -> 1 min between co-located modes)
- Hub buildings are larger footprint and higher cost
- Hub generates higher land value boost than individual stations

## Definition of Done
- [ ] 3+ transit hub types available
- [ ] Reduced transfer penalty at hubs
- [ ] Hubs serve multiple transit modes
- [ ] Higher land value boost than individual stations

## Test Plan
- Integration: Build bus-metro hub, verify transfers between modes
- Integration: Verify transfer time lower at hub than separate stations

## Pitfalls
- Requires both bus (TRAF-005) and metro (TRAF-006) to be functional
- Hub placement needs sufficient space (multi-cell)
- Transfer logic needs to detect co-located transit stops

## Relevant Code
- `crates/simulation/src/services.rs` -- hub service types
- `crates/simulation/src/movement.rs` -- transfer penalty at hubs

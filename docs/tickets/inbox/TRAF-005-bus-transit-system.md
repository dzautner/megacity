# TRAF-005: Bus Public Transit System
**Priority:** T2
**Complexity:** XL
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 11.1; master_architecture.md, section M3

## Description
Implement bus lines as the first public transit type. Buses follow player-drawn routes with stops, pick up citizens, and reduce car traffic. This is the most important T2 feature for traffic management.

- Player draws bus route on road network (sequence of road cells)
- Player places bus stops along route
- Bus depot building spawns buses on route
- Buses follow route, stop at stops for pickup/dropoff
- Citizens evaluate transit: walk_to_stop + wait + ride + walk_from_stop vs drive
- Citizens choose transit when transit time < drive_time * 1.3
- Each bus: 30 passenger capacity, limited by road speed (stuck in traffic)
- Cost: $400/week per line + $100/week per bus

## Definition of Done
- [ ] Player can draw bus routes and place stops
- [ ] Buses follow routes and pick up/drop off citizens
- [ ] Citizens choose bus vs car based on time comparison
- [ ] Bus ridership tracked per line
- [ ] Transit costs and fare revenue tracked in budget
- [ ] Bus depot required for line operation

## Test Plan
- Integration: Create bus line between residential and commercial, verify ridership > 0
- Integration: Remove bus line, verify car traffic increases
- Integration: Bus stuck in traffic has longer trip time, reducing attractiveness

## Pitfalls
- Bus routing on road network requires ordered waypoint sequence
- Bus bunching: multiple buses on same line cluster together -- need spacing logic
- Citizens walking to/from stops needs pathfinding on pedestrian network
- Transit line UI is complex (CS2's biggest UI regression was transit management)

## Relevant Code
- `crates/simulation/src/movement.rs` -- citizen mode choice
- `crates/simulation/src/services.rs:ServiceType::BusDepot` -- already exists
- `crates/rendering/src/input.rs` -- route drawing tool
- `crates/ui/src/toolbar.rs` -- transit tool category

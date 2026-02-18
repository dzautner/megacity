# INFRA-037: Bus Route and Stop System
**Priority:** T2
**Complexity:** XL (1-2 weeks)
**Dependencies:** none
**Source:** transportation_simulation.md, Section 4.3-4.4; master_architecture.md, M3

## Description
Implement bus transit system. `BusRoute` struct with ordered stop list, headway, vehicle type, dedicated lane flag, signal priority flag. `BusStop` placed along roads with catchment radius (400m/25 cells for local, 800m for express). Bus types: Standard (70-80 pass, $300K), Articulated (100-120 pass, $700K), BiArticulated (150-200 pass, $1.2M), Electric (60-70 pass, $600K). Operating cost per vehicle-hour ($100-180). Route drawing tool for player. Route performance tracking (ridership, revenue, cost).

## Definition of Done
- [ ] `BusRoute` struct with stops, headway, vehicle type, options
- [ ] `BusStop` entities placed along roads
- [ ] `BusType` enum with capacity and cost per type
- [ ] Route drawing tool in UI (click stops on roads)
- [ ] Bus vehicles move along route at computed speed
- [ ] Ridership tracking per route
- [ ] Operating cost and fare revenue per route
- [ ] Tests pass

## Test Plan
- Unit: Standard bus at 5-min headway = 12 vehicles/hr, 840 pass/hr capacity
- Unit: Operating cost for 10 buses * $120/hr = $1200/hr
- Integration: Placing bus stops and drawing route creates functioning transit line

## Pitfalls
- Stop spacing affects speed: 200m=14.7km/h avg, 400m=19.7km/h, 800m=23.8km/h
- Route must be valid (stops on road network, connected path between stops)
- Headway determines fleet size: vehicles_needed = cycle_time / headway

## Relevant Code
- `crates/simulation/src/movement.rs` -- citizen movement, add bus riding state
- `crates/simulation/src/economy.rs` -- transit revenue/expense
- `crates/rendering/src/input.rs` -- route drawing tool

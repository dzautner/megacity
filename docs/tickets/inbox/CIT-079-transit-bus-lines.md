# CIT-079: Bus Transit Lines

**Priority:** T2 (Depth)
**Complexity:** High (6-8 person-weeks)
**Dependencies:** CIT-038 (mode choice)
**Source:** master_architecture.md Section 1.7

## Description

Bus system: player draws routes on road network, places bus stops, assigns buses. Buses follow routes at set headways (frequency). Citizens choose bus if total travel time (walk to stop + wait + ride + walk from stop) is competitive with driving. Bus stop catchment: 500m walk radius. Headway affects wait time (avg wait = headway/2). Route profitability tracked (fare revenue vs operating cost). Bus lanes policy reduces bus travel time.

## Definition of Done

- [ ] Bus route drawing tool (sequence of road segments)
- [ ] Bus stop placement on routes
- [ ] Bus vehicle entities following routes
- [ ] Headway setting per route (5-30 min)
- [ ] Citizen ridership based on mode choice
- [ ] Bus stop catchment radius (500m)
- [ ] Route profitability (revenue - cost)
- [ ] Bus lane policy (speed bonus)
- [ ] Transit overlay showing routes and ridership

## Test Plan

- Unit test: bus follows route correctly
- Unit test: wait time = headway / 2
- Unit test: citizens choose bus when faster than driving
- Integration test: bus route reduces car traffic on parallel road

## Pitfalls

- First transit system; major feature requiring new drawing tools
- Bus vehicles on road network interact with traffic

## Relevant Code

- `crates/simulation/src/services.rs` (BusDepot)
- `crates/simulation/src/movement.rs`

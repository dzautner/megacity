# TRAF-014: Tram/Light Rail Transit System
**Priority:** T3
**Complexity:** L
**Dependencies:** TRAF-005
**Source:** cities_skylines_analysis.md, section 11.1, 14.2

## Description
Implement tram/light rail system. Trams run on tracks that can share road space or have dedicated right-of-way. Higher capacity than buses (90 passengers), immune to traffic on dedicated tracks.

- Road+tram combo road types (tram tracks embedded in road surface)
- Dedicated tram-only tracks
- Tram capacity: 90 passengers, speed 40 km/h
- Tram depot for vehicle maintenance
- Cost: $600/week per line + track construction
- Better than buses for medium-density corridors

## Definition of Done
- [ ] Tram tracks placeable (on-road and dedicated)
- [ ] Tram lines with stops
- [ ] Tram depot required for operation
- [ ] Citizens use trams in mode choice
- [ ] Tram costs tracked in budget

## Test Plan
- Integration: Build tram line, verify ridership

## Pitfalls
- Tram tracks on roads need special road type variants
- Tram stops different from bus stops (platform needed)
- Dedicated tram tracks are separate from road network but share surface space

## Relevant Code
- `crates/simulation/src/grid.rs:RoadType` -- add tram road variants
- `crates/simulation/src/movement.rs` -- tram mode in mode choice
- `crates/simulation/src/services.rs` -- TramDepot service type

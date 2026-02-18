# CIT-081: Tram/Light Rail System

**Priority:** T2 (Depth)
**Complexity:** High (6-8 person-weeks)
**Dependencies:** CIT-079 (bus lines as template)
**Source:** master_architecture.md Section 1.7

## Description

Tram/light rail: dedicated track or shared road, fixed stations. Higher capacity than buses (15,000 pax/hr vs 5,000), faster than buses but slower than metro. Track placed on existing roads or dedicated right-of-way. Tram stops every 300-500m. Tram priority at intersections (signal priority). Tram feeds metro system (hierarchical transit).

## Definition of Done

- [ ] Tram line drawing on road network
- [ ] Tram stop placement
- [ ] Tram vehicles following tracks
- [ ] Shared road or dedicated track option
- [ ] Signal priority at intersections
- [ ] Higher capacity than bus
- [ ] Integration with bus (transfer)
- [ ] Tram stop catchment (400m)

## Test Plan

- Unit test: tram follows track correctly
- Unit test: signal priority reduces tram travel time
- Integration test: tram corridor reduces car traffic

## Pitfalls

- Tram on shared road competes with traffic; dedicated track more expensive but faster

## Relevant Code

- `crates/simulation/src/services.rs` (TramDepot)

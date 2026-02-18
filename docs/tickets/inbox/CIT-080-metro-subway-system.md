# CIT-080: Metro/Subway System

**Priority:** T2 (Depth)
**Complexity:** High (6-8 person-weeks)
**Dependencies:** CIT-079 (bus lines as template)
**Source:** master_architecture.md Section 1.7

## Description

Underground rail system. Player draws metro lines with stations. Stations provide coverage radius (800m). Metro has highest capacity (60,000 passengers/hour/direction) but highest cost. Underground layer for tunnel visualization. Station construction requires surface access point. Metro dramatically reduces traffic on parallel surface routes. Transfer hubs where metro meets bus create transit network.

## Definition of Done

- [ ] Metro line drawing tool (underground path)
- [ ] Station placement with surface access
- [ ] Underground tunnel visualization
- [ ] Metro vehicle entities following lines
- [ ] Headway setting per line
- [ ] Station catchment radius (800m)
- [ ] Passenger capacity tracking
- [ ] Transfer hubs between metro and bus
- [ ] Metro construction cost (highest per km)

## Test Plan

- Unit test: metro line correctly placed underground
- Unit test: station provides coverage radius
- Unit test: citizens choose metro when available
- Integration test: metro line reduces parallel road traffic

## Pitfalls

- Underground layer is a significant rendering challenge
- Metro is the most expensive transit; must provide proportional benefit

## Relevant Code

- `crates/simulation/src/services.rs` (SubwayStation)
- New metro module needed

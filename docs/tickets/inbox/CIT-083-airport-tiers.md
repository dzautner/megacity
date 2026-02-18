# CIT-083: Airport Tier System

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.7

## Description

Three airport tiers with distinct functions. SmallAirstrip: 1 runway, domestic flights, minimal tourism, low noise. RegionalAirport: 2 runways, domestic + limited international, moderate tourism, moderate noise. InternationalAirport: 4+ runways, full international, high tourism capacity, high noise, massive land use. Airports generate noise pollution in approach path. Airport capacity limits tourism and trade connections. Airport construction requires large flat area.

## Definition of Done

- [ ] SmallAirstrip: domestic only, low capacity
- [ ] RegionalAirport: domestic + limited international
- [ ] InternationalAirport: full international, high capacity
- [ ] Noise contour from approach paths
- [ ] Passenger capacity per tier
- [ ] Tourism capacity linked to airport tier
- [ ] Trade connection capacity from cargo flights
- [ ] Large footprint requirement

## Test Plan

- Unit test: each tier has correct passenger capacity
- Unit test: noise generation from approach path
- Integration test: airport upgrade enables more tourism

## Pitfalls

- Airport noise affects nearby residential land value significantly
- Airports already exist in ServiceType; add functional differentiation

## Relevant Code

- `crates/simulation/src/airport.rs` (AirportStats)
- `crates/simulation/src/services.rs` (SmallAirstrip, RegionalAirport, InternationalAirport)
- `crates/simulation/src/noise.rs`

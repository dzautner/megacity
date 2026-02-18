# CIT-085: Freight Movement on Road Network

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** CIT-075 (production chains)
**Source:** master_architecture.md Section 1.6

## Description

Goods trucks travel road network from industrial to commercial zones and between production chain stages. Trucks are heavier, slower, and take more road capacity than cars. Freight volume = production output + imports. Truck routes prefer highways and arterials. Truck traffic on residential streets reduces happiness. Freight terminals as transfer points (rail-to-truck). Night-time delivery policy reduces daytime truck traffic.

## Definition of Done

- [ ] Freight demand from production and imports
- [ ] Truck entities on road network
- [ ] Trucks slower and larger than cars (more road capacity)
- [ ] Truck route preference for highways/arterials
- [ ] Truck traffic on residential streets = happiness penalty
- [ ] Freight terminal building type
- [ ] Night delivery policy option
- [ ] Freight volume metrics

## Test Plan

- Unit test: trucks prefer highways over residential
- Unit test: truck traffic reduces residential happiness
- Unit test: freight terminal reduces last-mile truck traffic
- Integration test: industrial zone generates visible truck traffic

## Pitfalls

- Freight adds to traffic congestion; player must plan industrial access

## Relevant Code

- `crates/simulation/src/production.rs`
- `crates/simulation/src/traffic.rs`

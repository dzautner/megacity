# INFRA-147: Commuter Rail System
**Priority:** T3
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-037
**Source:** transportation_simulation.md, Section 4.2

## Description
Implement commuter rail for long-distance suburban-to-downtown service. 8-car trains: 1000-2000 passengers. Stop spacing 2-5 km. Capital cost $20-80M/km. Capacity: 6,000-24,000 pphpd. Operates on dedicated right-of-way (separate from road network). Park-and-ride at suburban stations. Low frequency (6-12 trains/hr) but high capacity.

## Definition of Done
- [ ] Commuter rail track placement (separate from road network)
- [ ] Station placement with park-and-ride option
- [ ] Train entities with high capacity
- [ ] Wide stop spacing (2-5 km)
- [ ] Operating schedule with lower frequency than metro
- [ ] Ridership from mode choice with P&R integration
- [ ] Tests pass

## Test Plan
- Unit: 8-car train carries 1500 passengers
- Unit: P&R station captures suburban riders
- Integration: Commuter rail connects suburban residential to downtown employment

## Pitfalls
- Dedicated right-of-way needs significant land
- Low frequency reduces attractiveness (must compensate with high capacity)
- 256x256 grid may be too small for meaningful commuter rail distances

## Relevant Code
- `crates/simulation/src/movement.rs` -- rail movement
- `crates/simulation/src/buildings.rs` -- rail station building

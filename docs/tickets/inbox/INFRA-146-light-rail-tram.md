# INFRA-146: Light Rail/Tram System
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-037, INFRA-045
**Source:** transportation_simulation.md, Section 4.2

## Description
Implement light rail/tram as surface transit mode. Tram runs on tracks embedded in or alongside streets. 2-car: 200-300 passengers, 4-car: 400-600 passengers. Capital cost $30-100M/km. Capacity: 4,000-12,000 pphpd. Track placement along existing roads. Dedicated right-of-way option (shared with traffic or separated). Stop spacing 400-800m. Unlocks at 50K population.

## Definition of Done
- [ ] Light rail track placement tool (along roads)
- [ ] Tram vehicle entities moving on tracks
- [ ] Shared/dedicated right-of-way option
- [ ] Stop placement with 400-800m recommended spacing
- [ ] Tram capacity and operating cost
- [ ] Ridership from mode choice model
- [ ] Tests pass

## Test Plan
- Unit: Light rail capacity matches specification per car count
- Unit: Shared right-of-way reduces road auto capacity
- Integration: Light rail corridor carries 5000+ pphpd

## Pitfalls
- Track in shared right-of-way affects road capacity for cars
- Track construction requires road modification
- Tram speed limited by traffic signals in shared mode

## Relevant Code
- `crates/simulation/src/road_segments.rs` -- track along roads
- `crates/simulation/src/movement.rs` -- tram vehicle movement

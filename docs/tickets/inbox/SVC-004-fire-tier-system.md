# SVC-004: Fire Service Multi-Tier System

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** SVC-002 (capacity limits)
**Source:** historical_demographics_services.md Section 3.1

## Description

Differentiate fire service tiers functionally. FireHouse: 2 trucks, 8 firefighters, 2-mile radius, basic response. FireStation: 5 trucks, 25 firefighters, 4-mile radius, standard response. FireHQ: 10 trucks, 60 firefighters, 6-mile radius, coordinates multi-station response, HazMat capability. NFPA target: 4-minute response for 90% of calls. Response time from road distance. Fire suppression effectiveness based on response time: <4min = 80% contained, 4-8min = 50%, >8min = 20%.

## Definition of Done

- [ ] Functional differentiation of FireHouse/FireStation/FireHQ
- [ ] Truck and staffing counts per tier
- [ ] Response time calculation from road network distance
- [ ] Suppression effectiveness curve based on response time
- [ ] HazMat capability for FireHQ only
- [ ] NFPA response time metric tracked in stats
- [ ] Fire coverage overlay shows response time zones (green <4min, yellow 4-8, red >8)

## Test Plan

- Unit test: FireHouse has 2 trucks capacity
- Unit test: <4min response = 80% suppression
- Unit test: >8min response = 20% suppression
- Integration test: FireHQ coordinates response from multiple stations

## Pitfalls

- Response time depends on traffic; rush hour fires are harder to reach
- Multiple simultaneous fires can overwhelm a single station

## Relevant Code

- `crates/simulation/src/fire.rs` (start_random_fires, extinguish_fires)
- `crates/simulation/src/services.rs` (FireStation, FireHouse, FireHQ)

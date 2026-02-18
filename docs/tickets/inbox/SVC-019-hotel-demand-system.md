# SVC-019: Hotel Demand and Capacity

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** SVC-018 (tourism attraction)
**Source:** historical_demographics_services.md Section 4.2

## Description

Hotels as commercial zone sub-type serving tourist demand. Hotel demand = tourist_arrivals * avg_stay_duration / hotel_capacity. Occupancy rate target: 60-80%. Over 90% = room shortage (tourists can't visit, revenue lost). Under 50% = hotel financial stress (closures). Hotel star rating based on land value and service quality. Budget hotels in low land value, luxury in high. Hotels employ service workers.

## Definition of Done

- [ ] Hotel building type (commercial zone variant)
- [ ] Room capacity per hotel (based on building level)
- [ ] Occupancy rate tracking
- [ ] Over-capacity: tourism capped
- [ ] Under-capacity: hotel financial stress
- [ ] Hotel revenue flows to commercial income
- [ ] Hotel employment (staff per room ratio)

## Test Plan

- Unit test: tourist demand correctly fills hotel rooms
- Unit test: over-capacity caps tourist arrivals
- Integration test: building hotels enables more tourism revenue

## Pitfalls

- Hotels are commercial buildings but serve tourists, not residents; separate demand tracking

## Relevant Code

- `crates/simulation/src/tourism.rs`
- `crates/simulation/src/buildings.rs`

# CIT-084: Parking Simulation

**Priority:** T3 (Differentiation)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** CIT-038 (mode choice)
**Source:** master_architecture.md Section 1.6

## Description

Parking availability affects driving utility. Street parking along roads (capacity from road type). Parking garages as service buildings. Parking search time added to car commute when parking scarce. Parking cost in commercial districts. Parking minimums/maximums as zone policy. Reduced parking requirements incentivize transit use. Parking scarcity visible in overlay.

## Definition of Done

- [ ] Parking capacity per road segment and zone
- [ ] Parking garage building type
- [ ] Parking search time = f(occupancy_rate)
- [ ] Parking cost in commercial areas
- [ ] Parking minimum/maximum policy per zone
- [ ] Parking scarcity adds to car commute time
- [ ] Parking overlay showing occupancy
- [ ] Parking revenue for city budget

## Test Plan

- Unit test: full parking increases search time
- Unit test: parking cost added to car commute cost
- Unit test: no-parking-minimum policy reduces car mode share
- Integration test: parking crunch in downtown visible

## Pitfalls

- Parking simulation adds significant complexity; start simplified

## Relevant Code

- `crates/simulation/src/movement.rs`
- `crates/simulation/src/traffic.rs`

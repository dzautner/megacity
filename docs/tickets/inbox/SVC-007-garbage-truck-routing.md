# SVC-007: Garbage Truck Routing and Collection

**Priority:** T2 (Depth)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** SVC-003 (vehicle dispatch)
**Source:** historical_demographics_services.md Section 3.5, master_architecture.md Section 1.10

## Description

Garbage collection using vehicle routing. Each landfill/recycling center/incinerator has truck pool. Trucks follow routes visiting residential and commercial cells, collecting garbage. Collection reduces GarbageGrid values along route. Uncollected garbage accumulates, reduces land value and health. Truck capacity limits require return trips to disposal facility. Waste hierarchy: recycling diverts from landfill (reduces landfill fill rate).

## Definition of Done

- [ ] Garbage trucks as service vehicles
- [ ] Route generation covering all zoned cells within service radius
- [ ] Collection reduces garbage grid values
- [ ] Truck capacity (tons per trip)
- [ ] Return to facility when full
- [ ] Landfill capacity tracking (fills up over time)
- [ ] Recycling rate metric (% diverted from landfill)
- [ ] Incinerator: unlimited capacity but pollution output
- [ ] Uncollected garbage penalty on land value and health

## Test Plan

- Unit test: truck collects garbage along route
- Unit test: full landfill stops accepting garbage
- Unit test: recycling center diverts percentage from landfill
- Integration test: garbage overlay clears after trucks make rounds

## Pitfalls

- Route optimization is NP-hard (traveling salesman); use greedy/nearest-neighbor heuristic
- Landfill capacity exhaustion is a critical failure; must warn player

## Relevant Code

- `crates/simulation/src/garbage.rs` (GarbageGrid, update_garbage)
- `crates/simulation/src/services.rs` (Landfill, RecyclingCenter, Incinerator)

# INFRA-059: Garbage Collection Vehicle Routing
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-058
**Source:** master_architecture.md, M3

## Description
Implement garbage collection with vehicle routing. Garbage trucks depart from landfill/processing facility, follow route along roads, collect garbage from buildings, return when full. Each residential building generates garbage proportional to population. Collection frequency affects cleanliness/happiness. Uncollected garbage increases pollution and decreases land value.

## Definition of Done
- [ ] Garbage generation per residential/commercial building
- [ ] Garbage truck entities with capacity and route
- [ ] Collection route generated (TSP-like approximation)
- [ ] Uncollected garbage accumulates, causes pollution and land value drop
- [ ] Garbage collection overlay mode
- [ ] Tests pass

## Test Plan
- Unit: Building generates garbage proportional to population
- Unit: Full garbage truck returns to facility before continuing route
- Integration: Areas without garbage service show pollution increase

## Pitfalls
- TSP routing is NP-hard; use simple nearest-neighbor heuristic
- Garbage trucks are slow and block traffic; model as heavy vehicle
- Current `garbage.rs` module already exists; extend rather than replace

## Relevant Code
- `crates/simulation/src/garbage.rs` -- existing garbage module
- `crates/simulation/src/movement.rs` -- vehicle routing
- `crates/simulation/src/pollution.rs` -- garbage pollution

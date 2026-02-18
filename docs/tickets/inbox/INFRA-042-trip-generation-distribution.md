# INFRA-042: Trip Generation and Gravity Distribution Model
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** none
**Source:** transportation_simulation.md, Section 4.1 (Steps 1-2)

## Description
Implement the first two steps of the four-step travel demand model. Step 1 (Trip Generation): residential buildings produce trips (6.6-9.5 per household/day), commercial/industrial buildings attract trips (2.5-12 per employee/day). Step 2 (Trip Distribution): gravity model `T_ij = P_i * A_j * f(c_ij) / sum(A_j * f(c_ij))` with exponential friction function `f(c) = exp(-beta * c)`. Beta varies by trip purpose (work: 0.08-0.12, shopping: 0.15-0.25, school: 0.20-0.30).

## Definition of Done
- [ ] Trip production rates per building type
- [ ] Trip attraction rates per building type
- [ ] Gravity model function computing OD trip matrix
- [ ] Beta parameter per trip purpose
- [ ] Travel time matrix from pathfinding (sampled, not all-pairs)
- [ ] OD matrix feeds into mode choice and traffic assignment
- [ ] Tests pass

## Test Plan
- Unit: 100 households at 8 trips/day = 800 total productions
- Unit: Gravity model concentrates trips on nearby attractive zones
- Unit: Higher beta = more local trips

## Pitfalls
- Full OD matrix for 256x256 grid is too large; use zone-based aggregation (districts)
- Travel time matrix is expensive to compute; sample representative zone pairs
- Must balance productions and attractions so totals match

## Relevant Code
- `crates/simulation/src/buildings.rs` -- building population/employment
- `crates/simulation/src/districts.rs` -- zone aggregation
- `crates/simulation/src/movement.rs` -- trip destination selection

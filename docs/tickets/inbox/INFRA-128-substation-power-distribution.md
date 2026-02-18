# INFRA-128: Substation-Based Power Distribution
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-021
**Source:** infrastructure_engineering.md, Section 7 (Transmission/Distribution)

## Description
Implement substations that step voltage down for local distribution. Power plants connect to substations via transmission lines. Each substation serves an area with limited capacity. Overloaded substations cause brownouts. Transmission line losses proportional to distance. Players must build substations to distribute power to neighborhoods.

## Definition of Done
- [ ] Substation building with capacity (MW)
- [ ] Transmission line connection from plant to substation
- [ ] Substation coverage area
- [ ] Overload detection and brownout triggering
- [ ] Transmission loss proportional to distance
- [ ] Power distribution overlay
- [ ] Tests pass

## Test Plan
- Unit: Substation serving 5MW to 6MW demand area causes brownout
- Unit: 10km transmission line has 3% loss
- Integration: Building new substation resolves brownout in neighborhood

## Pitfalls
- Current power BFS does not use substations; significant architecture change
- Transmission lines need visual rendering (towers, cables)
- Substation NIMBY effect (noise, visual impact)

## Relevant Code
- `crates/simulation/src/utilities.rs` -- power system
- `crates/simulation/src/buildings.rs` -- substation building type

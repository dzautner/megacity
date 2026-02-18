# ECON-021: Toll Booth Revenue System
**Priority:** T2
**Complexity:** S
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 14.1

## Description
Implement placeable toll booths on roads that charge per vehicle passing. Revenue source for expensive infrastructure (bridges, highways). Modeled after CS1's After Dark DLC.

- Toll booth: placeable on any road (service building type)
- Revenue: $X per vehicle per passage (configurable: $1-$10 per vehicle)
- Traffic effect: some vehicles reroute to avoid toll (reduces demand on tolled road)
- Toll booth slows traffic slightly (vehicles stop to pay)
- Revenue tracked in budget under "toll revenue" category

## Definition of Done
- [ ] Toll booth placeable as service building on roads
- [ ] Revenue generated per vehicle passing through
- [ ] Some traffic diverts to avoid toll
- [ ] Revenue tracked in budget
- [ ] Toll rate configurable

## Test Plan
- Unit: Toll booth on high-traffic road generates positive revenue
- Integration: Place toll, verify some traffic reroutes, net revenue positive

## Pitfalls
- Need to count vehicles passing through toll cell (currently no per-cell vehicle counting)
- Traffic diversion requires toll cost in pathfinding weights
- Toll revenue should not be so high it replaces tax revenue entirely

## Relevant Code
- `crates/simulation/src/services.rs:ServiceType` -- add TollBooth variant
- `crates/simulation/src/economy.rs` -- toll revenue collection
- `crates/simulation/src/road_graph_csr.rs` -- toll cost in path weights

# INFRA-126: Transit Depot Placement and Deadheading
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** INFRA-037
**Source:** infrastructure_engineering.md, Section 2 (Fleet Management)

## Description
Implement transit depot buildings where buses are stored and maintained. Depot placement affects operating costs: distance from depot to route start = deadheading (empty vehicle travel). Deadheading typically 10-20% of total vehicle-miles. Vehicles need periodic maintenance (every 3,000-6,000 miles). Strategic depot placement near route endpoints reduces costs.

## Definition of Done
- [ ] Transit depot building type
- [ ] Deadheading cost from depot to route start
- [ ] Vehicle maintenance scheduling
- [ ] Depot capacity (number of vehicles)
- [ ] Operating cost includes deadheading
- [ ] Tests pass

## Test Plan
- Unit: Depot 5 km from route adds deadheading cost
- Unit: Depot adjacent to route has minimal deadheading
- Integration: Player learns to place depots near route endpoints

## Pitfalls
- Multiple depots can serve multiple routes; assignment optimization
- NIMBY effect for bus depots (noise, pollution)

## Relevant Code
- `crates/simulation/src/buildings.rs` -- depot building type
- `crates/simulation/src/economy.rs` -- transit operating costs

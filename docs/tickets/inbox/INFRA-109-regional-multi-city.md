# INFRA-109: Regional/Multi-City Play
**Priority:** T5
**Complexity:** XL (1-2 weeks)
**Dependencies:** none
**Source:** master_architecture.md, M6

## Description
Support multiple city maps connected in a region. Each city is a separate 256x256 grid. Cities trade resources, share workforce (commuters), and compete for immigration. Regional view shows all cities as tiles. Player can switch between cities. Inter-city connections (highway, rail) affect trade and commuting. Regional economy: one city's industrial output supplies another's commercial demand.

## Definition of Done
- [ ] Multiple city maps in one save
- [ ] Regional overview map
- [ ] Inter-city connections (road, rail)
- [ ] Trade between cities (resources, goods)
- [ ] Commuter flow between cities
- [ ] Regional population and economy stats
- [ ] Tests pass

## Test Plan
- Unit: Two cities can trade goods via connection
- Unit: Commuters travel between connected cities
- Integration: Regional play creates emergent specialization

## Pitfalls
- Memory: multiple 256x256 grids with full simulation is expensive
- Simulation of non-active cities must be simplified (background ticking)
- Save file size increases dramatically with multiple cities

## Relevant Code
- `crates/simulation/src/outside_connections.rs` -- inter-city connections
- `crates/save/src/lib.rs` -- multi-map save format

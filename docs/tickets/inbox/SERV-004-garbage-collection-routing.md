# SERV-004: Garbage Collection Routing
**Priority:** T2
**Complexity:** M
**Dependencies:** SERV-002
**Source:** cities_skylines_analysis.md, section 8.3; master_architecture.md, section M3

## Description
Implement garbage collection as a routing problem. Garbage trucks follow collection routes through residential/commercial areas. Buildings accumulate garbage; uncollected garbage reduces happiness and land value.

- Buildings accumulate garbage at rate proportional to occupancy
- Garbage truck capacity: 20 units, collects from buildings along route
- When full, returns to landfill/incinerator to dump
- Uncollected garbage: -5 happiness, -10 land value per 100 units uncollected
- Garbage pile visual at building when uncollected for > 30 game-days
- Recycling center: reduces garbage generation by 20% in district
- Recycling policy: costs more but reduces total waste

## Definition of Done
- [ ] Buildings accumulate garbage
- [ ] Garbage trucks collect along routes
- [ ] Uncollected garbage penalizes happiness and land value
- [ ] Landfill/incinerator processes collected garbage
- [ ] Garbage overlay shows collection coverage

## Test Plan
- Integration: No landfill, verify garbage accumulates and happiness drops
- Integration: Add landfill, verify garbage collected and happiness recovers

## Pitfalls
- Collection routing is a TSP variant -- use nearest-unvisited heuristic
- Garbage truck pathfinding on road network (same as other service vehicles)
- Landfill has finite capacity (eventually full -- need incinerator or recycling)

## Relevant Code
- `crates/simulation/src/services.rs:ServiceType::Landfill` -- already exists
- `crates/simulation/src/services.rs:ServiceType::RecyclingCenter` -- already exists
- `crates/simulation/src/buildings.rs:Building` -- add garbage_level field

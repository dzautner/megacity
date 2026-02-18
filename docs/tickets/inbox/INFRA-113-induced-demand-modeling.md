# INFRA-113: Induced Demand Modeling for Road Expansion
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-030
**Source:** infrastructure_engineering.md, Section 1 (Induced Demand)

## Description
Model induced demand: a 1% increase in road capacity generates approximately 1% more vehicle-miles traveled (Duranton & Turner). Road expansion provides temporary congestion relief (1-3 game-years) then fills back up. Sources: route diversion, time shifting, mode shifting, destination shifting, new trip generation. Highway expansion should show diminishing returns over time.

## Definition of Done
- [ ] Induced demand factor applied after road capacity increase
- [ ] Traffic volume grows proportionally to new capacity over 1-3 game-years
- [ ] Temporary relief visible followed by return to congestion
- [ ] Advisor hints about transit alternatives when induced demand kicks in
- [ ] Tests pass

## Test Plan
- Unit: Doubling highway capacity initially halves V/C, then V/C climbs back to ~1.0 over 3 years
- Integration: Player experiences diminishing returns from repeated highway widening

## Pitfalls
- Pure 1:1 elasticity may make roads feel useless; reduce to 0.7-0.8 for fun factor
- Induced demand should not apply to low-volume rural roads
- Must track "latent demand" that activates with new capacity

## Relevant Code
- `crates/simulation/src/traffic.rs` -- traffic volume tracking
- `crates/simulation/src/road_graph_csr.rs` -- capacity changes trigger demand growth

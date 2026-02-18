# ECON-024: Tourist Economy Model
**Priority:** T2
**Complexity:** M
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 14.1; master_architecture.md, section 2

## Description
Expand tourism from simple income number to a full tourist economy. Tourists visit attractions, spend money at commercial buildings, use hotels, and generate sales tax.

- Tourist count based on: attractions (landmarks, parks, unique buildings), transit connections (airport, outside connections), hotel capacity
- Tourist spending: $50-200/visit at commercial buildings
- Hotels: commercial specialization that houses tourists (Tourism commercial)
- Tourist attractions: unique buildings, high-level parks, historic districts
- Tourism seasonal variation (summer peak, winter low)
- tourism.rs already exists -- expand with spending model

## Definition of Done
- [ ] Tourist count computed from attractors
- [ ] Tourist spending at commercial buildings
- [ ] Hotel capacity affects tourist count
- [ ] Seasonal tourism variation
- [ ] Tourism revenue in budget breakdown

## Test Plan
- Integration: Build landmark + hotel, verify tourist count > 0
- Integration: Verify tourist spending generates sales tax revenue

## Pitfalls
- tourism.rs already has basic implementation
- Tourist pathing: do tourists actually visit attractions? (simplified: just count and spend)
- Airport (airport.rs) should boost tourist arrivals significantly

## Relevant Code
- `crates/simulation/src/tourism.rs` -- expand tourism model
- `crates/simulation/src/airport.rs` -- airport tourist connection
- `crates/simulation/src/economy.rs` -- tourist spending revenue

# SVC-018: Tourism Attraction Formula

**Priority:** T2 (Depth)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 4.1

## Description

Tourism attraction = cultural_facilities * 0.3 + natural_beauty * 0.2 + hotel_capacity * 0.15 + transport_access * 0.15 + safety * 0.1 + entertainment * 0.1. Tourists arrive based on attraction score, stay 1-5 days, spend money at commercial businesses. Tourist spending boosts commercial zone income. Seasonal variation: summer +50%, winter -30% (modifiable by ski resorts/beach). Convention center adds business tourism (separate from leisure). Airport size limits total tourist capacity.

## Definition of Done

- [ ] Tourism attraction formula with weighted components
- [ ] Tourist arrival rate based on attraction score
- [ ] Tourist duration (1-5 days, varies by attraction type)
- [ ] Tourist spending at commercial buildings
- [ ] Seasonal tourism modifiers
- [ ] Convention/business tourism from convention center
- [ ] Airport capacity limits on tourist volume
- [ ] Tourism revenue tracked in budget

## Test Plan

- Unit test: more cultural buildings = higher attraction
- Unit test: seasonal modifiers apply correctly
- Unit test: tourist spending boosts commercial income
- Integration test: tourism city generates visible revenue

## Pitfalls

- Tourism system already partially exists; enhance, don't duplicate
- Tourist entities should be lightweight (not full citizen simulation)

## Relevant Code

- `crates/simulation/src/tourism.rs` (existing tourism system)
- `crates/simulation/src/airport.rs` (AirportStats)

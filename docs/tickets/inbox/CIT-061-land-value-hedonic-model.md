# CIT-061: Hedonic Land Value Model

**Priority:** T2 (Depth)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.9

## Description

Replace simple land value recalculation with hedonic pricing model. Land value = f(accessibility, amenities, negative_externalities). Accessibility: distance to jobs (via road network), distance to transit, distance to highway ramp. Amenities: parks, schools, water views, cultural facilities. Negative externalities: pollution, noise, crime, industrial adjacency. Neighborhood spillover: high-value buildings raise neighbor values (diffusion). View corridors: water/park views provide bonus. Historical tracking of land value changes.

## Definition of Done

- [ ] Accessibility component: jobs reachable within 30 min
- [ ] Amenity component: parks, schools, water within radius
- [ ] Negative externality component: pollution, noise, crime
- [ ] Neighborhood spillover diffusion
- [ ] View corridor bonus for water/park-facing cells
- [ ] Incremental update (not full reset each cycle)
- [ ] Historical land value tracking per chunk
- [ ] Land value determines building level-up, rent, property tax

## Test Plan

- Unit test: cell near park has higher land value
- Unit test: cell near factory has lower land value
- Unit test: spillover effect raises neighbor values
- Integration test: transit station increases nearby land values over time

## Pitfalls

- Current system resets to base 50 each cycle; must switch to incremental
- Accessibility calculation is expensive; precompute and cache

## Relevant Code

- `crates/simulation/src/land_value.rs` (LandValueGrid, update_land_value)

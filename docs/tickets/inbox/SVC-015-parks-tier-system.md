# SVC-015: Parks Multi-Tier System

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 3.7

## Description

Differentiate park tiers functionally. SmallPark (1x1): +5 happiness, +3 land value, minor green space. Playground: +5 happiness for families, children exercise. LargePark (2x2+): +10 happiness, +8 land value, pollution reduction, exercise/health bonus. SportsField: +5 happiness, exercise bonus, youth activities. Plaza: +3 happiness, commercial boost, social gathering. Parks per 1000 pop target: 10 acres (NRPA standard). Park deficit reduces happiness.

## Definition of Done

- [ ] SmallPark: local happiness and land value
- [ ] Playground: family-specific happiness
- [ ] LargePark: extended radius, health bonus, pollution absorption
- [ ] SportsField: exercise bonus, youth crime reduction
- [ ] Plaza: commercial adjacency bonus
- [ ] Parks per capita metric (acres per 1000 pop)
- [ ] Park deficit penalty
- [ ] Green space percentage metric

## Test Plan

- Unit test: large park provides wider radius than small
- Unit test: playground provides family-specific bonus
- Unit test: park deficit reduces happiness
- Integration test: parks improve nearby land value visibly

## Pitfalls

- Multi-cell buildings (large park 2x2) depend on multi-cell building system
- Park maintenance cost should be low but nonzero

## Relevant Code

- `crates/simulation/src/services.rs` (SmallPark, LargePark, Playground, SportsField, Plaza)

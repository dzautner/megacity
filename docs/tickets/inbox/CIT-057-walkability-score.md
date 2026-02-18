# CIT-057: Walkability Score and 15-Minute City Metric

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.3

## Description

Per-cell walkability score (0-100) based on: amenities within 15-minute walk (commercial, parks, schools, health), intersection density (more intersections = more walkable), road type (pedestrian paths score highest, highways score 0), building density. City-wide 15-minute city score = fraction of residential cells with walkability > 60. Walkability affects: car ownership (lower in walkable areas), health bonus (walking citizens), land value (+10 per 20 walkability points), happiness (+5 for walkable neighborhoods).

## Definition of Done

- [ ] Per-cell walkability score (0-100)
- [ ] Amenity proximity within 15-min walk radius
- [ ] Intersection density factor
- [ ] Pedestrian path bonus, highway penalty
- [ ] 15-minute city score (city-wide metric)
- [ ] Walkability overlay visualization
- [ ] Walkability -> car ownership reduction
- [ ] Walkability -> health, land value, happiness bonuses

## Test Plan

- Unit test: cell near shops, park, school = high walkability
- Unit test: cell in highway-only area = low walkability
- Integration test: walkability overlay shows expected patterns

## Pitfalls

- 15-minute walk = ~1.2km radius; BFS on road graph needed for accurate calculation
- Walkability computation is expensive; cache and update only on road/building changes

## Relevant Code

- `crates/simulation/src/land_value.rs`
- `crates/simulation/src/happiness.rs`

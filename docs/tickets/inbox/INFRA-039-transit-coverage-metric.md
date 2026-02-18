# INFRA-039: Transit Coverage Metric (% Buildings within Walking Distance)
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** INFRA-037
**Source:** transportation_simulation.md, Section 4.5

## Description
Compute transit coverage: percentage of buildings within 400m (5-min walk, ~25 cells) of a transit stop. Target: 80%+ coverage for well-served cities. Display as city-wide metric in stats panel. Overlay mode showing transit catchment areas. Advisor warning when coverage drops below 50%.

## Definition of Done
- [ ] Transit coverage percentage computed from stop locations
- [ ] Coverage displayed in stats panel
- [ ] Transit catchment overlay showing areas within 400m of stops
- [ ] Advisor warning at <50% coverage
- [ ] Tests pass

## Test Plan
- Unit: Building 20 cells from stop = covered; 30 cells from stop = not covered
- Unit: City with stops every 400m has ~100% coverage

## Pitfalls
- Walking distance should be network distance, not Euclidean (roads, not straight line)
- Multiple stops nearby don't double-count buildings

## Relevant Code
- `crates/simulation/src/stats.rs` -- metric tracking
- `crates/simulation/src/advisors.rs` -- advisor warning
- `crates/rendering/src/overlay.rs` -- catchment overlay

# ZONE-014: Neighborhood Quality Index
**Priority:** T3
**Complexity:** M
**Dependencies:** ZONE-013
**Source:** urban_planning_zoning.md, section 5.5

## Description
Compute a composite neighborhood quality index per district that combines walkability, service coverage, environment quality, safety, and aesthetics. This becomes a key metric for player feedback and citizen satisfaction.

- Components: walkability (20%), service coverage (20%), pollution/noise (20%), crime rate (15%), park access (15%), building quality average (10%)
- Computed per district on slow tick
- Displayed in district info panel
- Affects immigration attractiveness at district level
- High-quality neighborhoods attract higher-income citizens

## Definition of Done
- [ ] Neighborhood quality index computed per district
- [ ] Components weighted and combined correctly
- [ ] Displayed in district statistics panel
- [ ] Affects immigration/citizen sorting into districts

## Test Plan
- Unit: District with full service coverage, low crime, parks scores > 80
- Unit: Industrial district with high pollution scores < 40
- Integration: Improve services in district, verify quality index rises

## Pitfalls
- Need to aggregate per-cell values to district level (average? weighted?)
- Quality index should not be identical to happiness (related but distinct metrics)

## Relevant Code
- `crates/simulation/src/districts.rs:aggregate_districts` -- compute quality index
- `crates/simulation/src/happiness.rs:ServiceCoverageGrid` -- coverage data
- `crates/ui/src/info_panel.rs` -- display quality index

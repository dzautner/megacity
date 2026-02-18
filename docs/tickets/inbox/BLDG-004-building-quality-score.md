# BLDG-004: Continuous Building Quality Score
**Priority:** T2
**Complexity:** M
**Dependencies:** BLDG-001
**Source:** urban_planning_zoning.md, section 3.6; cities_skylines_analysis.md, section 1.3

## Description
Replace binary level-up with a continuous building quality score (0-100). Quality combines service coverage (40%), land value (20%), pollution+noise (15%), crime (10%), and infrastructure (15%). Building upgrades when quality exceeds threshold for N consecutive days. Downgrades when below threshold.

- Add `BuildingQuality` component: score (f32 0-100), trend (f32), last_upgrade (u32)
- Compute quality every slow tick from surrounding conditions
- Level-up thresholds: L2 at 30, L3 at 50, L4 at 70, L5 at 85 (matching CS reverse-engineering)
- Must sustain quality above threshold for 30+ game days before upgrade
- Downgrade if quality drops below current level threshold - 10 for 30+ days
- Quality trend visible in building info panel

## Definition of Done
- [ ] BuildingQuality component added to all buildings
- [ ] Quality computed from weighted factors
- [ ] Level-up occurs when quality sustained above threshold
- [ ] Level-down occurs when quality drops below threshold
- [ ] Quality visible in building inspection UI

## Test Plan
- Unit: Full services + low pollution + low crime = quality > 80
- Unit: No services + high pollution = quality < 30
- Integration: Provide services to area, verify buildings gradually upgrade
- Integration: Remove services, verify buildings gradually downgrade

## Pitfalls
- Must replace or integrate with existing land_value threshold system in building_upgrade.rs
- Quality calculation every tick for all buildings is O(buildings * radius) -- needs optimization
- Sustained quality check needs timer per building (not just instantaneous check)

## Relevant Code
- `crates/simulation/src/building_upgrade.rs:upgrade_buildings` -- rewrite with quality score
- `crates/simulation/src/buildings.rs:Building` -- add quality component or separate component
- `crates/simulation/src/happiness.rs:ServiceCoverageGrid` -- service coverage input

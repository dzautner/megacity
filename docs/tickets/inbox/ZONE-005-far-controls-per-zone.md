# ZONE-005: Floor Area Ratio (FAR) Controls Per Zone
**Priority:** T2
**Complexity:** M
**Dependencies:** ZONE-003
**Source:** urban_planning_zoning.md, section 2.6

## Description
Implement Floor Area Ratio as a constraint on building density. FAR = total floor area / lot area. This controls how tall/dense a building can be relative to its lot.

- Add FAR values per ZoneType: R-1=0.5, R-2=1.5, R-3=3.0, R-4=15.0, C-1=1.5, C-2=3.0, C-3=0.5, C-4=15.0, I-1=0.8, I-2=1.5, I-3=1.0, O-1=1.5, O-2=25.0
- Create `max_level_for_far(zone, transect)` function that finds highest level whose implied FAR does not exceed the limit
- Implied FAR = (capacity * 20 sq m per person) / 256 sq m per cell
- Building spawner and upgrade system both check FAR limit
- TransectZone can override zone-default FAR (lower or higher)
- Add FAR bonus mechanic for specific conditions (transit proximity, affordable housing)

## Definition of Done
- [ ] Each zone type has a default FAR value
- [ ] `max_level_for_far` correctly constrains building levels
- [ ] Building spawner respects FAR limits
- [ ] Building upgrades respect FAR limits
- [ ] TransectZone can override default FAR

## Test Plan
- Unit: max_level_for_far(ResidentialLow, TransectZone::T3Suburban) returns <= 2
- Unit: max_level_for_far(ResidentialHigh, TransectZone::T6Core) returns 5
- Property: For all zone/transect combos, implied FAR at returned level <= max_far

## Pitfalls
- FAR calculation depends on capacity table -- changes to capacity_for_level break FAR
- 20 sqm per person is a rough estimate; needs calibration per zone type
- FAR bonuses (transit proximity) add complexity and must be bounded

## Relevant Code
- `crates/simulation/src/grid.rs:ZoneType` -- add default_far() method
- `crates/simulation/src/buildings.rs:Building::capacity_for_level` -- used for implied FAR
- `crates/simulation/src/building_upgrade.rs` -- check FAR before upgrade

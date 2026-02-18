# ZONE-010: Inclusionary Zoning Requirements
**Priority:** T3
**Complexity:** M
**Dependencies:** ZONE-001, ECON-012
**Source:** urban_planning_zoning.md, section 6.5

## Description
Implement inclusionary zoning as a district policy requiring new residential developments to reserve a percentage of units as affordable housing. This creates mixed-income neighborhoods and addresses housing affordability.

- District policy toggle: Inclusionary Zoning (10-20% affordable units)
- New residential buildings in affected districts have reduced effective capacity (10-20% units generate less/no tax revenue)
- Affordable units house lower-income citizens who would otherwise be priced out
- Developer incentive: FAR bonus (+10-20%) to offset affordable unit cost
- Affects building profitability and construction rate

## Definition of Done
- [ ] District policy toggle for inclusionary zoning with configurable percentage
- [ ] New buildings in district have affordable unit allocation
- [ ] Lower-income citizens can access affordable units
- [ ] FAR bonus applied as offset
- [ ] Tax revenue reduced proportionally for affordable units

## Test Plan
- Unit: Building in inclusionary district has affordable unit count > 0
- Integration: Enable policy, verify lower-income citizens move into high-value districts

## Pitfalls
- Must track affordable vs market-rate units per building
- Interaction with happiness system (income diversity in neighborhoods)
- Developers may avoid building in inclusionary zones if profitability drops too much

## Relevant Code
- `crates/simulation/src/districts.rs:DistrictPolicies` -- add inclusionary flag
- `crates/simulation/src/buildings.rs:Building` -- track affordable unit count
- `crates/simulation/src/citizen_spawner.rs` -- match low-income citizens to affordable units

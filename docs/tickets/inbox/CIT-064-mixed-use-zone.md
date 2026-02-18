# CIT-064: Mixed-Use Zone Type

**Priority:** T3 (Differentiation)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.3

## Description

Add MixedUse zone type: commercial ground floor + residential above. Mixed-use buildings have both commercial capacity and residential capacity. Creates walkable neighborhoods where residents can walk to shops without driving. Land value bonus for mixed-use (+15 over pure residential). Reduces commute for some citizens (live-work proximity). Mixed-use buildings are 2+ levels (ground commercial, upper residential).

## Definition of Done

- [ ] MixedUse added to ZoneType enum
- [ ] Mixed-use buildings have commercial_capacity + residential_capacity
- [ ] Ground floor serves as commercial (shopping destination)
- [ ] Upper floors serve as residential (home location)
- [ ] Land value bonus (+15) for mixed-use zones
- [ ] Walkability bonus for nearby residential cells
- [ ] Zone demand calculation includes mixed-use
- [ ] Mixed-use building mesh showing shops at street level

## Test Plan

- Unit test: mixed-use building has both capacities
- Unit test: citizens can shop at ground floor of their building
- Unit test: land value bonus applied
- Integration test: mixed-use neighborhood has higher walkability

## Pitfalls

- Mixed-use adds complexity to building spawner (must handle dual capacity)
- Zone painting UI needs new tool for mixed-use

## Relevant Code

- `crates/simulation/src/grid.rs` (ZoneType enum)
- `crates/simulation/src/zones.rs` (ZoneDemand)
- `crates/simulation/src/buildings.rs` (Building)

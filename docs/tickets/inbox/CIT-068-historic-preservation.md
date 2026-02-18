# CIT-068: Historic Preservation Districts

**Priority:** T3 (Differentiation)
**Complexity:** Low-Medium (2-3 person-weeks)
**Dependencies:** CIT-058 (building age)
**Source:** master_architecture.md Section 1.3

## Description

Historic preservation designation for old buildings/districts. Historic buildings: cannot be demolished without penalty, cannot upgrade (maintain historic character), tourism attraction bonus, land value bonus, maintenance cost premium. Historic district: all buildings within district get preservation status. Demolishing historic building: -10 civic trust, NIMBY protest. Preserving: +5 tourism, +10 land value in district.

## Definition of Done

- [ ] Historic designation status for buildings
- [ ] Historic district tool (designate area)
- [ ] Cannot demolish historic buildings (or with trust penalty)
- [ ] Cannot upgrade historic buildings
- [ ] Tourism attraction bonus for historic buildings
- [ ] Land value bonus in historic district
- [ ] Higher maintenance cost for historic buildings
- [ ] Historic preservation as district policy (GOV-009)

## Test Plan

- Unit test: historic building cannot be upgraded
- Unit test: demolishing historic building costs civic trust
- Unit test: tourism bonus applied to historic buildings
- Integration test: historic district has higher land value

## Pitfalls

- Historic buildings that can't upgrade create a "museum district" issue; may conflict with growth

## Relevant Code

- `crates/simulation/src/buildings.rs`
- `crates/simulation/src/building_upgrade.rs`
- `crates/simulation/src/districts.rs`

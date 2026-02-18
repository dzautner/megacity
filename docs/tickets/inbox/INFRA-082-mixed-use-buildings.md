# INFRA-082: Mixed-Use Buildings
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-081
**Source:** master_architecture.md, M4

## Description
Implement mixed-use buildings with commercial ground floor and residential upper floors. These spawn in form-based T4-T6 zones or high-density commercial/residential overlap areas. Mixed-use buildings provide both jobs and housing, reducing commute distances. Building has split capacity: ground floor commercial (jobs), upper floors residential (population). Unique mesh with storefront ground floor.

## Definition of Done
- [ ] Mixed-use building type with commercial + residential capacity
- [ ] Spawns in appropriate form-based zones
- [ ] Provides both jobs and housing
- [ ] Citizens can live and work in same building (zero commute)
- [ ] Distinct mesh with commercial ground floor
- [ ] Tests pass

## Test Plan
- Unit: Mixed-use building has both jobs and housing capacity
- Unit: Citizen living and working in same building has zero commute
- Integration: Mixed-use district has lower average commute times

## Pitfalls
- Tax revenue calculation: which rate applies (residential or commercial)?
- Land value influenced by both residential and commercial factors
- Building info panel needs to show dual capacity

## Relevant Code
- `crates/simulation/src/buildings.rs` -- new building type
- `crates/rendering/src/building_meshes.rs` -- mixed-use mesh

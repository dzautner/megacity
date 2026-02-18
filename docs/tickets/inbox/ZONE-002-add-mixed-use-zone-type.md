# ZONE-002: Add MixedUse Zone Type
**Priority:** T3
**Complexity:** L
**Dependencies:** ZONE-001
**Source:** urban_planning_zoning.md, section 2.5; cities_skylines_analysis.md, section 1.6

## Description
Add `MixedUse` zone type where buildings have commercial ground floors and residential upper floors. This is the most-requested zoning feature from CS1/CS2 communities and reflects real urban development patterns.

- Add `ZoneType::MixedUse` variant to enum
- Create `MixedUseBuilding` component with separate commercial_capacity/occupants and residential_capacity/occupants
- MU buildings satisfy both commercial and residential demand simultaneously
- Capacity scaling per level: L1=(5 comm, 8 res), L2=(15, 30), L3=(20+20 office, 80), L4=(40+80, 200), L5=(80+200, 400)
- Building spawner must create entity with both Building and MixedUseBuilding components
- Citizens can have HomeLocation OR WorkLocation pointing to the same MixedUse building
- Tax collection must account for both commercial and residential components

## Definition of Done
- [ ] MixedUse zone paintable and buildings spawn
- [ ] Buildings serve both residential and commercial/office functions
- [ ] Citizens live and work in MixedUse buildings
- [ ] Tax revenue correctly includes both residential and commercial components
- [ ] Zone demand satisfied for both R and C when MixedUse buildings are occupied

## Test Plan
- Unit: MixedUseBuilding capacity returns correct split per level
- Integration: Zone MixedUse, verify both residents and workers occupy same building
- Integration: Verify zone demand decreases for both R and C when MixedUse fills

## Pitfalls
- Building rendering needs distinct model (ground floor storefront + upper floor apartments)
- Occupancy tracking is dual -- need to handle capacity for workers vs residents separately
- Zone demand calculation must count MixedUse toward both R and C saturation
- Save/load must serialize MixedUseBuilding component

## Relevant Code
- `crates/simulation/src/grid.rs:ZoneType` -- add variant
- `crates/simulation/src/buildings.rs:Building` -- extend or create MixedUseBuilding
- `crates/simulation/src/zones.rs:update_zone_demand` -- count toward both R and C
- `crates/simulation/src/economy.rs:collect_taxes` -- dual tax calculation

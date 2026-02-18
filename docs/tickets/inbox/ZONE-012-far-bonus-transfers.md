# ZONE-012: FAR Bonuses and Transfers (TDR)
**Priority:** T3
**Complexity:** M
**Dependencies:** ZONE-005
**Source:** urban_planning_zoning.md, section 6.7

## Description
Implement Transfer of Development Rights (TDR) and FAR bonuses. Allow developers to exceed FAR limits in exchange for public benefits (affordable housing, public plazas, transit contributions). Allow unused FAR from historic/park parcels to transfer to nearby development sites.

- FAR bonus triggers: affordable housing inclusion (+20% FAR), public plaza provision (+10%), transit contribution (+15%)
- TDR: historic preservation districts and parks have "unused" FAR that can transfer to nearby parcels
- Transfer radius: within same district or adjacent districts
- Creates market for development rights (gameplay mechanic)

## Definition of Done
- [ ] FAR bonuses applied for qualifying public benefits
- [ ] TDR system allows FAR transfer between parcels
- [ ] Bonus/transfer FAR visualized in overlay
- [ ] Development decisions account for FAR bonus availability

## Test Plan
- Unit: Building with affordable housing gets +20% FAR bonus
- Integration: Transfer FAR from park to adjacent development site

## Pitfalls
- TDR accounting must prevent double-counting (transferred FAR removed from source)
- Complex interaction with transect overlay
- Must not make FAR system too complex for players to understand

## Relevant Code
- `crates/simulation/src/grid.rs` -- track FAR bonuses per cell
- `crates/simulation/src/buildings.rs:building_spawner` -- apply FAR bonuses
- `crates/simulation/src/districts.rs` -- TDR accounting per district

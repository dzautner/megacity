# BLDG-001: Expand to 5-Level Building System
**Priority:** T1
**Complexity:** M
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 1.3-1.4; master_architecture.md, section 1.4

## Description
Expand ResidentialLow and CommercialLow from 3 levels to 5 levels, matching ResidentialHigh/CommercialHigh/Industrial/Office. This provides finer-grained visual progression and deeper gameplay.

- ResidentialLow: L1=4, L2=8, L3=15, L4=25, L5=40 (houses to small apartments)
- CommercialLow: L1=5, L2=12, L3=25, L4=40, L5=60 (corner store to medium retail)
- Update `max_level()` for ResidentialLow and CommercialLow to return 5
- Add level-up thresholds for L4 and L5 in building_upgrade.rs
- Create distinct building meshes for L4 and L5
- Update upgrade requirements: L4 needs services + parks + low pollution; L5 needs all services + multiple parks + zero pollution

## Definition of Done
- [ ] ResidentialLow.max_level() == 5
- [ ] CommercialLow.max_level() == 5
- [ ] Capacity table complete for all 5 levels
- [ ] Building upgrade thresholds defined for L4 and L5
- [ ] Distinct meshes for each level
- [ ] Save/load handles buildings at levels 4-5

## Test Plan
- Unit: capacity_for_level(ResidentialLow, 5) returns expected value
- Unit: max_level(ResidentialLow) == 5
- Integration: Provide all services, verify buildings reach level 5

## Pitfalls
- Existing level 3 buildings in save files must remain valid
- Building upgrade thresholds need careful tuning to feel achievable but not trivial
- More levels = more distinct meshes needed (visual variety ticket is separate)

## Relevant Code
- `crates/simulation/src/grid.rs:ZoneType::max_level` -- change return values
- `crates/simulation/src/buildings.rs:Building::capacity_for_level` -- add L4, L5 entries
- `crates/simulation/src/building_upgrade.rs` -- add L4, L5 thresholds
- `crates/rendering/src/building_meshes.rs` -- L4, L5 mesh generation

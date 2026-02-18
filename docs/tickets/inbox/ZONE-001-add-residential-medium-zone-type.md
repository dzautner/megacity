# ZONE-001: Add ResidentialMedium Zone Type
**Priority:** T1
**Complexity:** M
**Dependencies:** none
**Source:** urban_planning_zoning.md, section 1.5; master_architecture.md, section 1.3

## Description
The current `ZoneType` enum in `grid.rs` has `ResidentialLow` (max level 3) and `ResidentialHigh` (max level 5) but no medium density tier. Add `ResidentialMedium` to bridge the gap -- townhouses, duplexes, and small apartment buildings (3-6 stories). This corresponds to R-2/R-3 in the research docs.

- Add `ResidentialMedium` variant to `ZoneType` enum
- Set `max_level` to 4 for ResidentialMedium
- Define capacity_for_level: L1=15, L2=50, L3=120, L4=250
- Update `is_residential()` to include new variant
- Update `ZoneDemand::demand_for()` to map ResidentialMedium to residential demand
- Add zone painting tool entry in `input.rs`
- Update serialization in save crate
- Update building_spawner to handle new zone type
- Update zone overlay colors

## Definition of Done
- [ ] `ZoneType::ResidentialMedium` compiles and is usable
- [ ] Buildings spawn in ResidentialMedium zones with correct capacities
- [ ] Zone demand system accounts for new type
- [ ] Save/load round-trips correctly with new zone type
- [ ] Zone overlay renders distinct color for ResidentialMedium

## Test Plan
- Unit: `Building::capacity_for_level(ZoneType::ResidentialMedium, 1..4)` returns expected values
- Unit: `ZoneType::ResidentialMedium.is_residential()` returns true
- Unit: `ZoneDemand::demand_for(ResidentialMedium)` returns residential demand
- Integration: Place ResidentialMedium zone, verify buildings spawn and citizens move in

## Pitfalls
- Every match arm on ZoneType throughout the codebase must be updated (search for `ZoneType::` exhaustive matches)
- Save format changes need migration or backward-compatible deserialization
- Building meshes need distinct visual for medium density

## Relevant Code
- `crates/simulation/src/grid.rs:ZoneType` -- enum definition
- `crates/simulation/src/buildings.rs:Building::capacity_for_level` -- capacity table
- `crates/simulation/src/zones.rs:ZoneDemand::demand_for` -- demand mapping
- `crates/rendering/src/input.rs` -- zone painting tool
- `crates/save/src/serialization.rs` -- save/load

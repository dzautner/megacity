# ZONE-008: Historic Preservation Districts
**Priority:** T3
**Complexity:** M
**Dependencies:** ZONE-003
**Source:** urban_planning_zoning.md, section 6.3; cities_skylines_analysis.md, section 12.2

## Description
Allow player to designate districts as historic preservation zones. Buildings in historic districts cannot be demolished, upgraded, or replaced. They generate land value bonuses (+5-15%) but prevent density increases.

- Add `historic_preservation: bool` flag to district settings
- When enabled: buildings frozen at current level, no demolition, no replacement
- Land value bonus: +10% for cells in historic district
- Tourism attraction: historic districts generate tourism visits
- Equivalent to CS1's "Old Town" policy but district-level
- Player can remove preservation (at happiness cost from preservationists)

## Definition of Done
- [ ] District can be marked as historic preservation
- [ ] Buildings in district cannot upgrade/downgrade/demolish
- [ ] Land value receives preservation bonus
- [ ] Tourism system recognizes historic districts
- [ ] Removing preservation triggers happiness penalty

## Test Plan
- Unit: Building in historic district returns false for should_upgrade
- Integration: Mark district historic, verify buildings remain unchanged over time
- Integration: Verify land value increases after marking district historic

## Pitfalls
- Must interact correctly with abandonment system (abandoned buildings in historic districts?)
- Fire damage in historic districts -- can buildings still be destroyed by fire?
- Old buildings may need maintenance costs even if not upgradeable

## Relevant Code
- `crates/simulation/src/districts.rs:DistrictPolicies` -- add historic flag
- `crates/simulation/src/building_upgrade.rs` -- check district before upgrade
- `crates/simulation/src/abandonment.rs` -- check district before demolish
- `crates/simulation/src/tourism.rs` -- historic district tourism bonus

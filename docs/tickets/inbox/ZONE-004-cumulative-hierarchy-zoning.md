# ZONE-004: Cumulative Hierarchy Zoning Rules
**Priority:** T3
**Complexity:** M
**Dependencies:** ZONE-001, ZONE-002
**Source:** urban_planning_zoning.md, section 1.1

## Description
Implement Euclidean cumulative hierarchy where higher zones implicitly allow lower-intensity uses. Currently zones are exclusive (residential only grows residential). With cumulative hierarchy, a C-1 zone can contain houses, an M-1 zone can contain offices.

- Define zone hierarchy: R-1 < R-2 < R-3 < R-4 < C-1 < C-2 < M-1 < M-2
- Building spawner should check if the zone permits the building type via `zone.permits(building_type)` rather than exact match
- Add `permits_residential()`, `permits_commercial()`, `permits_industrial()` methods to ZoneType
- Make this a policy toggle (cumulative vs exclusive) so players can choose behavior
- When cumulative mode enabled, building spawner selects highest-value permitted use via market demand

## Definition of Done
- [ ] Cumulative hierarchy policy toggle exists
- [ ] When enabled, commercial zones can grow residential buildings
- [ ] Building type selection respects hierarchy order
- [ ] Player can toggle between exclusive and cumulative zoning

## Test Plan
- Unit: ZoneType::CommercialLow.permits_residential() returns true in cumulative mode
- Unit: ZoneType::ResidentialLow.permits_commercial() returns false
- Integration: Zone as CommercialLow in cumulative mode, verify both shops and houses can appear

## Pitfalls
- Must be backward-compatible (default to exclusive mode matching current behavior)
- Building spawner needs to decide WHICH permitted type to build -- use zone demand weights
- Zone overlay must still show the zone type, not the building type

## Relevant Code
- `crates/simulation/src/grid.rs:ZoneType` -- add hierarchy methods
- `crates/simulation/src/buildings.rs:building_spawner` -- check permits() instead of exact match
- `crates/simulation/src/policies.rs` -- add cumulative zoning toggle

# ZONE-003: Form-Based Transect Overlay System
**Priority:** T3
**Complexity:** L
**Dependencies:** ZONE-001, BLDG-001
**Source:** urban_planning_zoning.md, section 1.2; master_architecture.md, section 1.3

## Description
Implement form-based codes as an overlay on top of Euclidean zoning. The transect (T1-T6) controls physical building form (height, FAR, lot coverage, setbacks) independent of use. This allows players to say "I want medium-density here" without specifying residential vs commercial.

- Add `TransectZone` enum: None, T1Natural, T2Rural, T3Suburban, T4Urban, T5Center, T6Core
- Add `transect` field to `Cell` struct (default None = unconstrained)
- Implement `max_stories()`, `max_far()`, `max_lot_coverage()`, `front_setback_cells()` per transect tier
- Building spawner checks transect constraints before spawning (cap level based on FAR)
- Building upgrade system respects transect limits (cannot upgrade beyond transect-allowed level)
- Add transect painting tool to input system
- Add transect overlay to overlay system

## Definition of Done
- [ ] TransectZone enum with T1-T6 tiers implemented
- [ ] Cell struct carries transect data
- [ ] Building spawner caps building level based on transect FAR
- [ ] Player can paint transect zones on map
- [ ] Transect overlay shows color-coded zones
- [ ] T1Natural prevents all building

## Test Plan
- Unit: TransectZone::T3Suburban.max_stories() == 3
- Unit: TransectZone::T6Core.max_far() == 15.0
- Integration: Paint T3Suburban over ResidentialHigh zone, verify buildings cap at ~3 stories
- Integration: T1Natural zone prevents any building spawning

## Pitfalls
- Must work as overlay ON TOP of use-based zoning, not replacement
- Default None must not restrict existing behavior (backward compatible)
- FAR-to-level conversion needs careful calibration with building capacity table
- Serialization must handle new Cell field

## Relevant Code
- `crates/simulation/src/grid.rs:Cell` -- add transect field
- `crates/simulation/src/buildings.rs:building_spawner` -- check transect before spawn
- `crates/simulation/src/building_upgrade.rs` -- check transect before upgrade
- `crates/rendering/src/input.rs` -- add transect painting tool
- `crates/rendering/src/overlay.rs` -- add transect overlay

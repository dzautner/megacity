# CIT-065: ResidentialMedium Zone Type

**Priority:** T1 (Core)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.3

## Description

Add ResidentialMedium density tier between Low and High. ResidentialLow: single-family homes (2-4 units). ResidentialMedium: townhouses and small apartments (8-16 units). ResidentialHigh: apartment towers (50-200 units). Medium density fills the density gap and creates more realistic urban form. Missing from current ZoneType enum per master_architecture.md gap analysis.

## Definition of Done

- [ ] ResidentialMedium added to ZoneType enum
- [ ] Capacity per level for medium density
- [ ] Building mesh for medium density (townhouse/small apartment)
- [ ] Zone demand calculation includes medium density
- [ ] Zone painting tool for medium density
- [ ] Medium density has intermediate land value requirement
- [ ] Save migration for existing saves (backward compatible)

## Test Plan

- Unit test: medium density capacity between low and high
- Unit test: zone demand correctly factors medium density
- Visual test: medium density buildings look distinct from low/high

## Pitfalls

- Adding a zone type affects zone painting, building spawning, rendering, save/load
- Must update all zone type matches throughout codebase

## Relevant Code

- `crates/simulation/src/grid.rs` (ZoneType enum)
- `crates/simulation/src/zones.rs`
- `crates/simulation/src/buildings.rs`
- `crates/rendering/src/building_meshes.rs`

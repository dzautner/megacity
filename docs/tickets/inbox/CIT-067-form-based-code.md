# CIT-067: Form-Based Zoning Codes

**Priority:** T3 (Differentiation)
**Complexity:** High (4-5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.3

## Description

Alternative to Euclidean zoning: form-based codes regulate building form, not use. Parameters: max_height, setback_front, setback_side, lot_coverage_max, FAR (floor area ratio), ground_floor_use (any/commercial_required). Zones defined by form: Rural (1 story, 20% coverage), Suburban (2 story, 40%), General Urban (4 story, 70%), Urban Center (8 story, 80%), Urban Core (unlimited, 90%). This allows mixed-use naturally (form allows any use that fits the form).

## Definition of Done

- [ ] FormBasedCode struct with height, setback, coverage, FAR
- [ ] 5 form-based zones replacing/augmenting Euclidean zones
- [ ] Building spawner respects form parameters
- [ ] FAR calculation: total_floor_area / lot_area
- [ ] Ground floor commercial requirement in Urban Center/Core
- [ ] Form-based code as optional system (toggle in game settings)
- [ ] Visual guide showing form parameters

## Test Plan

- Unit test: FAR correctly limits total floor area
- Unit test: building height respects max_height
- Unit test: ground floor commercial enforced in center zones
- Integration test: form-based city has mixed-use character

## Pitfalls

- Form-based codes are complex; optional mode reduces risk
- Must coexist with Euclidean zoning (player chooses system)

## Relevant Code

- `crates/simulation/src/grid.rs` (ZoneType)
- `crates/simulation/src/buildings.rs` (building_spawner)

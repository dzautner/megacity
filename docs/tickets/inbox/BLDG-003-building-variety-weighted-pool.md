# BLDG-003: Building Variety via Weighted Random Pool
**Priority:** T2
**Complexity:** L
**Dependencies:** BLDG-001
**Source:** urban_planning_zoning.md, section 3.4; master_architecture.md, section 1.4

## Description
Replace the single building mesh per zone/level with a weighted random pool system. Each zone type + level has 4-15 building template entries with different weights, visual styles, and contextual constraints (land value range, corner lot, era).

- Define `BuildingPool` with `BuildingPoolEntry` structs per zone/level
- Entry contains: template, weight, min/max land value, requires_corner, max_per_district, era
- `BuildingPool::select()` picks weighted random entry filtered by context
- Building gets a `template_id` (u16) stored on Building component for mesh lookup
- Add `BuildingAppearance` component: material_variant, color_tint, roof_variant, weathering
- Randomize appearance at spawn time within zone-appropriate ranges

## Definition of Done
- [ ] BuildingPool defined for at least R-1, R-3, C-1, C-4, I-1 (one pool per zone/level)
- [ ] Building spawner uses weighted selection instead of fixed template
- [ ] At least 4 distinct visual variants per zone/level combo
- [ ] BuildingAppearance randomized per building
- [ ] Buildings visually distinct within same zone/level

## Test Plan
- Unit: BuildingPool::select returns None when no entries match constraints
- Unit: Weight distribution statistically matches over 1000 selections
- Integration: Zone area, verify visual variety in spawned buildings

## Pitfalls
- Need enough mesh variants to make variety visible (art pipeline concern)
- template_id must be stable across save/load (don't randomize on load)
- Corner lot detection needs intersection detection logic

## Relevant Code
- `crates/simulation/src/buildings.rs` -- add template_id, create BuildingPool
- `crates/rendering/src/building_meshes.rs` -- multiple mesh generators per template
- `crates/rendering/src/building_render.rs` -- select mesh by template_id

# BLDG-010: Building Appearance Variation (Material/Color/Roof)
**Priority:** T2
**Complexity:** M
**Dependencies:** BLDG-003
**Source:** urban_planning_zoning.md, section 3.4

## Description
Add per-building visual randomization to prevent the "copy-paste" look. Each building gets randomized material, color tint, roof variant, and window pattern within zone-appropriate ranges.

- `BuildingAppearance` component: material_variant (0-5), color_tint [f32;3], roof_variant (0-3), window_variant (0-3), weathering (0.0-1.0)
- Material palette per zone: residential=warm earth tones, commercial=warm, office=blue-gray glass, industrial=gray
- Color tint +/- 10% random variation from base
- Roof styles vary by land value (high value = more variety)
- Weathering increases with building age (0.0 at construction, +0.01/game-year)
- Randomized at spawn, saved, stable across reload

## Definition of Done
- [ ] BuildingAppearance component on all buildings
- [ ] Visual variation visible in rendered city
- [ ] Color palette appropriate per zone type
- [ ] Weathering increases over time
- [ ] Appearance serialized in save file

## Test Plan
- Unit: BuildingAppearance::random_for_zone produces values in valid ranges
- Integration: Spawn 10 buildings in same zone, verify no two look identical
- Save/load: Appearance preserved after round-trip

## Pitfalls
- Shader needs to support per-instance material variation
- Color tints must remain within aesthetically pleasing ranges (no neon houses)
- Bevy instance data may have limits on per-instance uniforms

## Relevant Code
- `crates/simulation/src/buildings.rs` -- add BuildingAppearance component
- `crates/rendering/src/building_render.rs` -- apply appearance to mesh
- `crates/rendering/src/building_meshes.rs` -- material/color support

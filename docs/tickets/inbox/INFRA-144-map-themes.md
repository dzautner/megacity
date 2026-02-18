# INFRA-144: Additional Map Themes (Desert, Arctic, Tropical, Mountain)
**Priority:** T5
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-009
**Source:** master_architecture.md, M6

## Description
Add map theme presets that configure terrain generation, biome distribution, and visual style. Desert: arid, flat, sand terrain, oasis water sources, extreme heat. Arctic: snowy, frozen water, permafrost (limits underground construction), high heating demand. Tropical: lush vegetation, high rainfall, jungle biome, flooding risk. Mountain: high elevation, steep terrain, limited flat building area, scenic value.

## Definition of Done
- [ ] Map theme enum: Temperate, Desert, Arctic, Tropical, Mountain
- [ ] Theme configures terrain generation parameters
- [ ] Theme configures biome distribution
- [ ] Theme affects gameplay (heating, water, vegetation)
- [ ] Visual palette changes per theme
- [ ] Theme selection on new game screen
- [ ] Tests pass

## Test Plan
- Unit: Desert theme generates < 10% water cells
- Unit: Arctic theme increases heating demand by 50%
- Integration: Each theme produces visually distinct and playably different maps

## Pitfalls
- Theme-specific gameplay effects need balance testing per theme
- Not all game mechanics apply in all themes (no snow in desert)
- Terrain colors need to match theme aesthetic

## Relevant Code
- `crates/simulation/src/terrain.rs` -- theme-based generation parameters
- `crates/rendering/src/terrain_render.rs` -- theme-based colors

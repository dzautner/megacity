# INFRA-009: Biome Classification via Whittaker Diagram
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 4.1-4.3

## Description
Implement temperature and moisture maps using noise + elevation, then classify each cell into a biome using a Whittaker diagram lookup. Temperature decreases with elevation (lapse rate ~6.5C/km). Moisture comes from separate noise layer + proximity to water. Biomes: tropical rainforest, temperate forest, grassland, desert, tundra, etc. Biome affects vegetation density, building styles, and terrain color.

## Definition of Done
- [ ] `BiomeType` enum with at least 6 biome variants
- [ ] Temperature map generated from noise + elevation
- [ ] Moisture map generated from noise + water proximity
- [ ] Whittaker diagram lookup classifies each cell
- [ ] Biome stored per-cell or in a separate `BiomeGrid`
- [ ] Tests pass

## Test Plan
- Unit: High elevation cells should have lower temperature
- Unit: Cells near water should have higher moisture
- Integration: Visual terrain rendering uses biome for color selection

## Pitfalls
- 256x256 map is only 4km; real biome variation happens at 100km+ scale. Use exaggerated parameters.
- Biome boundaries should be smooth transitions, not hard edges

## Relevant Code
- `crates/simulation/src/terrain.rs` -- biome generation
- `crates/simulation/src/grid.rs` -- new `BiomeType` field on `Cell`
- `crates/rendering/src/terrain_render.rs` -- biome-based vertex colors

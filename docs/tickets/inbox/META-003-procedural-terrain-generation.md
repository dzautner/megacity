# META-003: Procedural Terrain Generation

**Category:** Meta / Terrain
**Priority:** T2
**Source:** master_architecture.md, community_wishlists.md

## Summary

Replace flat terrain with procedural generation. fBm noise for terrain heightmap, erosion simulation, biome assignment. Multiple terrain types: flat plains, coastal, mountainous, island, desert, river valley. Heightmap-based rendering (currently flat y=0).

## Details

- fBm (fractal Brownian motion) for base terrain
- Hydraulic erosion simulation for realistic valleys
- Biome system: temperature + moisture -> biome type
- Water body generation (rivers, lakes, coast)
- Terrain slope affects buildability
- Multiple map presets for variety

## Acceptance Criteria

- [ ] Procedural terrain with height variation
- [ ] Rivers and water bodies generated
- [ ] Slope affects building placement
- [ ] Multiple terrain presets available
- [ ] Terrain rendering uses heightmap

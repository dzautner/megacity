# INFRA-016: 3D Terrain Rendering with Heightmap-Driven Vertices
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 8.2; master_architecture.md, M3

## Description
Replace the current flat terrain mesh (y=0.0 for all vertices) in `terrain_render.rs` with heightmap-driven vertex positions. Each vertex Y-coordinate should equal `cell.elevation * HEIGHT_SCALE`. Normals must be recalculated for proper lighting. Keep chunk-based mesh generation (CHUNK_SIZE=8). Add terrain LOD: full resolution for nearby chunks, simplified for distant.

## Definition of Done
- [ ] Terrain vertices use elevation for Y position
- [ ] HEIGHT_SCALE constant configurable (default ~50.0 for visible hills)
- [ ] Normals recalculated per-vertex from neighboring cell elevations
- [ ] Chunk meshes update when elevation changes
- [ ] Terrain LOD: 2 levels (full and simplified)
- [ ] Tests pass

## Test Plan
- Unit: Flat terrain produces y=0 vertices, hilly terrain produces non-zero Y
- Integration: Hills are visible in-game with proper lighting/shading

## Pitfalls
- Roads and buildings must also be elevated to match terrain height
- Camera near-clip may hide terrain at extreme angles
- LOD transitions between chunks must stitch seamlessly (T-junctions cause gaps)
- Water cells should remain at water surface elevation (not terrain elevation)

## Relevant Code
- `crates/rendering/src/terrain_render.rs` -- `spawn_terrain_chunks()`, vertex generation
- `crates/rendering/src/building_render.rs` -- building Y position must match terrain
- `crates/rendering/src/road_render.rs` -- road Y position must match terrain

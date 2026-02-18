# REND-001: 3D Terrain Rendering with Heightmap
**Priority:** T2
**Complexity:** L
**Dependencies:** none
**Source:** master_architecture.md, section M3

## Description
Implement 3D terrain rendering using heightmap-driven vertex positions. Currently terrain is flat. Hills, valleys, and water bodies create interesting city planning constraints and visual variety.

- Heightmap: per-vertex elevation from grid cell elevation values
- Terrain chunks: existing CHUNK_SIZE=8 chunks, each gets displaced vertices
- Water rendering: cells below water level rendered as water surface
- Terrain LOD: distant chunks use lower vertex density
- Slope affects: road cost (uphill expensive), building placement (steep = no building), runoff
- Procedural noise for elevation (fBm with octaves)

## Definition of Done
- [ ] Terrain vertices displaced by elevation
- [ ] Hills and valleys visible
- [ ] Water level renders as water surface
- [ ] Roads follow terrain elevation
- [ ] Buildings placed at correct elevation

## Test Plan
- Visual: Terrain with varied elevation renders correctly
- Integration: Buildings on slopes placed at correct height
- Integration: Water cells render as water

## Pitfalls
- Road rendering on slopes needs careful normal computation
- Building placement on steep slopes may clip through terrain
- Terrain editing (flatten for building) needs tool support

## Relevant Code
- `crates/rendering/src/terrain_render.rs` -- vertex displacement
- `crates/simulation/src/grid.rs:Cell::elevation` -- elevation data source
- `crates/simulation/src/terrain.rs` -- procedural elevation generation

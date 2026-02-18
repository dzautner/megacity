# REND-002: Procedural Terrain Generation (fBm + Erosion)
**Priority:** T2
**Complexity:** L
**Dependencies:** REND-001
**Source:** master_architecture.md, section M3

## Description
Generate terrain procedurally using fractional Brownian motion (fBm) noise with hydraulic erosion simulation. Add water bodies, rivers, and biomes. Seed-based generation for replayability.

- fBm noise: 6 octaves, persistence 0.5, lacunarity 2.0
- Hydraulic erosion: simulate rainfall and sediment transport (100K iterations)
- Water bodies: cells below threshold = water
- River generation: from high elevation to water body following steepest descent
- Biome assignment: based on elevation and moisture
- Seed parameter: deterministic generation from u64 seed
- Generate at game start or from map selection menu

## Definition of Done
- [ ] fBm noise generates varied elevation
- [ ] Hydraulic erosion creates realistic terrain features
- [ ] Water bodies and rivers placed naturally
- [ ] Seed produces identical terrain on replay
- [ ] At least 5 seeds produce distinct, playable maps

## Test Plan
- Unit: Same seed produces identical elevation grid
- Visual: Generated terrain has mountains, valleys, rivers, coastlines
- Integration: Generated terrain is playable (enough flat area for city)

## Pitfalls
- terrain.rs is currently a stub -- needs full implementation
- Erosion simulation is CPU-intensive (run during loading screen, not real-time)
- Must ensure enough flat/buildable area for city (terrain can't be all mountains)

## Relevant Code
- `crates/simulation/src/terrain.rs` -- terrain generation
- `crates/simulation/src/grid.rs:Cell::elevation` -- elevation storage
- `crates/rendering/src/terrain_render.rs` -- terrain visualization

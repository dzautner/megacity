# INFRA-002: Domain Warping for Organic Terrain Shapes
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 1.3

## Description
Add domain warping to the terrain heightmap generation using `fastnoise-lite`'s `FractalType::DomainWarpProgressive`. Use two independent warp noise generators (seed+1000, seed+2000) with `warp_strength=30.0` cells, 4 octaves, frequency 0.006. This transforms regular noise blobs into flowing, organic geological shapes.

## Definition of Done
- [ ] Domain warping applied before final terrain sampling
- [ ] Warp strength configurable (10-80 range)
- [ ] Visual comparison: terrain looks more natural with warping vs without
- [ ] Tests pass

## Test Plan
- Unit: Verify warped terrain still normalizes to [0,1]
- Integration: Visual inspection that terrain has elongated ridges instead of round blobs

## Pitfalls
- Warp strength > 60 cells can push samples off-grid edge, causing wrapping artifacts
- Multi-layer warping (warping the warp) adds compute cost; keep to 1 layer for now

## Relevant Code
- `crates/simulation/src/terrain.rs` -- generation pipeline

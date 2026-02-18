# CIT-074: Procedural Building Variety

**Priority:** T2 (Depth)
**Complexity:** High (4-6 person-weeks)
**Dependencies:** CIT-059 (five levels)
**Source:** master_architecture.md Section 1.4

## Description

Procedural building mesh variation. Currently all buildings of same type/level share identical mesh. Generate variation through: randomized height within level range, facade variation (window patterns, balcony placement), color palette per zone type with random selection, corner lot buildings (L-shaped or wider), roof type variation (flat, pitched, parapet). Seed per building for deterministic reproduction. Each building looks unique while maintaining zone type identity.

## Definition of Done

- [ ] Height randomization within level band
- [ ] 3-5 facade variations per zone type per level
- [ ] Color palette randomization (warm/cool/neutral per zone)
- [ ] Corner lot detection and special mesh
- [ ] Roof type variation
- [ ] Deterministic seed per building (same position = same appearance)
- [ ] Performance: mesh generation < 0.1ms per building

## Test Plan

- Unit test: same seed produces same mesh
- Unit test: different seeds produce visually different meshes
- Visual test: block of buildings shows variety
- Performance test: mesh generation for 1000 buildings < 100ms

## Pitfalls

- Procedural variety must be attractive, not random noise
- Mesh generation at city load for 5000+ buildings must be fast

## Relevant Code

- `crates/rendering/src/building_meshes.rs`
- `crates/rendering/src/building_render.rs`

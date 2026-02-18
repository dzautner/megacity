# INFRA-132: Edge Falloff for Island Maps
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 1.5

## Description
Add configurable edge falloff to create island-style maps surrounded by water. Multiply elevation by a radial falloff function: 1.0 at center, 0.0 at edges. Falloff shape: quadratic `(1 - t^2)` from `falloff_start` (50% distance) to `falloff_end` (95% distance). This creates a bounded island for building. Optional per-map setting (some maps are islands, others extend to edges).

## Definition of Done
- [ ] `edge_falloff()` function with configurable parameters
- [ ] Falloff applied as elevation multiplier during generation
- [ ] Map type option: island, coastal, continental
- [ ] Island maps have water border on all sides
- [ ] Tests pass

## Test Plan
- Unit: Center of map unaffected by falloff (factor = 1.0)
- Unit: Edge of map submerged (factor = 0.0)
- Integration: Island map provides natural water boundary

## Pitfalls
- Falloff at corners may be more aggressive than edges; consider rectangular falloff
- Island maps must still have enough flat area for a city

## Relevant Code
- `crates/simulation/src/terrain.rs` -- elevation post-processing

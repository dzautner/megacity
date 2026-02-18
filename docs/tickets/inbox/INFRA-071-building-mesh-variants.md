# INFRA-071: Building Mesh Variants (2-3 per Zone Type per Level)
**Priority:** T1
**Complexity:** L (3-5 days)
**Dependencies:** none
**Source:** master_architecture.md, M2

## Description
Add visual variety to buildings by creating at least 2-3 procedurally generated mesh variants per zone type per level. Currently each zone type + level combination has a single mesh. Variants should differ in roof style, window pattern, height variation (within level), and color accent. Variant selected from hash of cell position for deterministic but varied appearance.

## Definition of Done
- [ ] 2-3 mesh variants per zone type per level (minimum 30 total meshes)
- [ ] Variant selection deterministic from cell position (hash)
- [ ] Visual differences clear but not jarring
- [ ] All variants render correctly with lighting
- [ ] Tests pass

## Test Plan
- Unit: Same cell position always selects same variant
- Unit: Different positions select different variants (distribution roughly even)
- Integration: City block shows varied building appearances

## Pitfalls
- Procedural mesh generation can produce invalid geometry; validate normals
- Too much variation looks chaotic; maintain zone-type visual coherence
- Mesh memory usage increases 3x per type; monitor RAM

## Relevant Code
- `crates/rendering/src/building_meshes.rs` -- mesh generation
- `crates/rendering/src/building_render.rs` -- variant selection

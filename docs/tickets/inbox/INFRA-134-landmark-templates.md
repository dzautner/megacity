# INFRA-134: Landmark Templates in Map Generation
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 6.4

## Description
Add optional landmark templates that can be stamped onto generated terrain: mountain peak (cone heightmap), river valley (carved channel), harbor bay (semicircular depression at coast), plateau (flat elevated area), canyon (narrow deep channel). Templates enhance generated terrain with recognizable features. Selected based on map type or randomized from seed.

## Definition of Done
- [ ] At least 5 landmark template functions
- [ ] Templates blend smoothly with surrounding terrain
- [ ] Template selection from seed or map configuration
- [ ] Templates placed at appropriate locations (mountain at high elevation, bay at coast)
- [ ] Tests pass

## Test Plan
- Unit: Mountain peak template creates elevated cone at target location
- Unit: Template blends smoothly (no hard edges)
- Integration: Generated map with landmarks looks more interesting than pure noise

## Pitfalls
- Templates must respect playability (don't put mountain peak in only flat area)
- Blending edge artifacts when template elevation differs greatly from surroundings

## Relevant Code
- `crates/simulation/src/terrain.rs` -- template stamping

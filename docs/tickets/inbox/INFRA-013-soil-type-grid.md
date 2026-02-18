# INFRA-013: Soil Type Grid
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-001, INFRA-009
**Source:** procedural_terrain.md, Section 7.3

## Description
Add a soil type grid derived from elevation, slope, moisture, and biome. Soil types: Rock, Clay, Sand, Loam, Silt, Gravel. Soil affects: construction cost (rock = expensive to dig, clay = cheap), agricultural fertility (loam > clay > sand), groundwater permeability, foundation stability, and underground infrastructure tunneling cost. Store in `Cell.soil_type: SoilType` or separate grid.

## Definition of Done
- [ ] `SoilType` enum with at least 5 variants
- [ ] Soil type assigned per cell based on terrain properties
- [ ] Construction cost modifier per soil type
- [ ] Agricultural fertility modifier per soil type
- [ ] Tests pass

## Test Plan
- Unit: River valleys get silt/loam, mountains get rock/gravel
- Unit: Construction cost varies by soil type

## Pitfalls
- Soil type is generated once and should be deterministic from seed
- Need to decide if soil type is serialized (yes -- it affects gameplay)

## Relevant Code
- `crates/simulation/src/grid.rs` -- `Cell` struct, new `SoilType` enum
- `crates/simulation/src/terrain.rs` -- soil type generation
- `crates/simulation/src/buildings.rs` -- construction cost modifier

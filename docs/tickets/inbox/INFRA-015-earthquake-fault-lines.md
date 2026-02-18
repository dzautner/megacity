# INFRA-015: Earthquake Fault Lines in Terrain
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 7.5

## Description
Generate fault lines as linear features across the terrain using Voronoi cell boundaries or random walk paths. Buildings near fault lines have higher earthquake damage risk. Fault lines are hidden until geological survey reveals them. During earthquake events, damage is concentrated near fault lines rather than uniformly distributed. Fault line proximity modifies construction cost (seismic reinforcement).

## Definition of Done
- [ ] 1-3 fault lines generated per map from terrain seed
- [ ] Fault line proximity stored per cell (distance to nearest fault)
- [ ] Earthquake damage scales with fault proximity
- [ ] Fault lines visible in geological survey overlay
- [ ] Tests pass

## Test Plan
- Unit: Fault lines span significant portion of map
- Unit: Earthquake damage is higher near fault lines than far from them

## Pitfalls
- Fault lines through the center of the only flat area make the map nearly unplayable
- May need to push fault lines toward map edges for playability

## Relevant Code
- `crates/simulation/src/terrain.rs` -- fault line generation
- `crates/simulation/src/disasters.rs` -- earthquake damage scaling

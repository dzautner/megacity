# INFRA-005: Thermal Erosion for Cliff and Talus Slopes
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 2.3

## Description
Implement thermal erosion that creates natural cliff faces and talus slopes. When the slope between adjacent cells exceeds a `talus_angle` threshold, material moves from the higher cell to the lower cell. This softens sharp ridges and creates realistic scree slopes at cliff bases. Run 20-50 iterations after hydraulic erosion.

## Definition of Done
- [ ] `thermal_erosion()` processes heightmap in-place
- [ ] Talus angle threshold configurable (default ~0.05 in elevation units)
- [ ] Sharp ridges become smoother; material accumulates at bases
- [ ] Tests pass

## Test Plan
- Unit: Create a step function heightmap, verify thermal erosion creates a smooth slope
- Unit: Verify no elevation goes negative

## Pitfalls
- Iterating in fixed order creates directional bias; use checkerboard or random order
- Too many iterations flatten all terrain features

## Relevant Code
- `crates/simulation/src/terrain.rs` -- post-processing after hydraulic erosion

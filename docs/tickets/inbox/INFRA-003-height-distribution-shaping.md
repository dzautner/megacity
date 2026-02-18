# INFRA-003: Height Distribution Shaping (Plateau/Power Curves)
**Priority:** T2
**Complexity:** S (hours)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 1.4

## Description
Apply transfer functions to raw fBm output to create terrain suitable for city building: large flat buildable areas with some water and occasional hills. Implement `plateau_remap()` that flattens mid-range elevations (0.3-0.7) and `power_remap(h, exponent)` for adjustable land/water ratio. Exponent < 1.0 = more land, > 1.0 = more water.

## Definition of Done
- [ ] `plateau_remap()` and `power_remap()` functions implemented
- [ ] Remapping applied after noise generation, before cell assignment
- [ ] At least 40% of cells are in buildable elevation range
- [ ] Tests pass

## Test Plan
- Unit: Verify remap functions are monotonically increasing, preserve [0,1] range
- Unit: Generate terrain, check histogram has plateau in mid-range

## Pitfalls
- Non-monotonic remapping creates terrain inversions (valleys become peaks)
- Power curve with exponent 0 collapses all terrain to 1.0

## Relevant Code
- `crates/simulation/src/terrain.rs` -- post-processing step

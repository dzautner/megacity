# INFRA-023: Underground Infrastructure Collision Detection
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-019
**Source:** underground_infrastructure.md, Underground Collision Detection

## Description
Implement `UndergroundOccupancy` grid that tracks what infrastructure exists at each depth layer (Shallow/Medium/Deep) for each cell. Prevent placing two incompatible infrastructure elements at the same depth. Metro tunnels and utility tunnels cannot share space. Small pipes (water + power cable) can colocate. Display red highlight during placement preview when conflicts exist, with tooltip explaining the conflict.

## Definition of Done
- [ ] `UndergroundOccupancy` resource with per-cell, per-layer tracking
- [ ] `can_place()` checks occupancy before placement
- [ ] `can_colocate()` rules for compatible infrastructure pairs
- [ ] Red highlight on conflicting cells during placement preview
- [ ] Conflict tooltip explaining the issue
- [ ] Tests pass

## Test Plan
- Unit: Cannot place metro tunnel where another metro tunnel exists
- Unit: Can place water main + power cable in same cell at same depth
- Unit: Utility tunnel excludes individual pipes at same depth

## Pitfalls
- Occupancy grid must be serialized for save/load
- Removing infrastructure must clear occupancy
- Multiple infrastructure types need `UndergroundType` enum: WaterMain, SewerMain, StormDrain, PowerCable, MetroTunnel, MetroStation, UtilityTunnel

## Relevant Code
- `crates/simulation/src/utilities.rs` -- infrastructure placement
- `crates/rendering/src/input.rs` -- placement preview with conflict highlight

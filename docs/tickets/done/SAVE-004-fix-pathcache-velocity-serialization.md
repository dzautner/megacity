# SAVE-004: Fix PathCache/Velocity Serialization
**Priority:** T0
**Complexity:** S
**Dependencies:** none
**Source:** master_architecture.md, section M2 (known issue)

## Description
PathCache and Velocity components are not serialized. Commuting citizens lose their paths on save/load, causing them to teleport or stand still until recalculated.

- Serialize PathCache (current path waypoints, current waypoint index)
- Serialize Velocity (current movement direction and speed)
- On load, citizens resume movement along saved path
- If path becomes invalid on load (road changed), recalculate path

## Definition of Done
- [ ] PathCache serialized with waypoints
- [ ] Velocity serialized
- [ ] Citizens resume movement on load
- [ ] Invalid paths gracefully recalculated

## Test Plan
- Integration: Save while citizens are moving, load, verify they continue moving
- Integration: Save, modify road, load, verify path recalculation

## Pitfalls
- Path waypoints reference grid coordinates, which should be stable across save/load
- Vec<(usize, usize)> serialization for waypoints
- Velocity is just (f32, f32) -- simple to serialize

## Relevant Code
- `crates/simulation/src/movement.rs` -- PathCache, Velocity definitions
- `crates/save/src/serialization.rs` -- add to citizen save/load

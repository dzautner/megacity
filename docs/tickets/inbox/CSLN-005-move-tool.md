# CSLN-005: Move/Rotate Tool for Placed Objects
**Priority:** T4
**Complexity:** L
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 17.4

## Description
CS1's Move It! mod had ~5M subscribers -- the single most essential quality-of-life mod. Players need ability to nudge, rotate, and reposition placed buildings and objects after initial placement.

- Select any placed building, service, or prop
- Move with precision (grid-snap or free movement)
- Rotate to any angle
- Height adjustment (raise/lower from ground)
- Undo/redo for move operations
- Bulk selection (marquee select multiple objects)

## Definition of Done
- [ ] Select and move individual buildings
- [ ] Free rotation and grid-snap rotation
- [ ] Undo/redo for moves
- [ ] Bulk selection and movement
- [ ] Height adjustment

## Test Plan
- Integration: Place building, move it, verify it renders at new position
- Integration: Undo move, verify building returns to original position

## Pitfalls
- Moving a building must update grid cell references (building_id, zone state)
- Multi-cell buildings require moving all cells
- Road nodes cannot easily be moved (breaks pathfinding graph)

## Relevant Code
- `crates/rendering/src/input.rs` -- move tool
- `crates/simulation/src/buildings.rs` -- update grid position
- `crates/simulation/src/grid.rs` -- cell reference updates

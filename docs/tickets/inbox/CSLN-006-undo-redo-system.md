# CSLN-006: Undo/Redo System for All Player Actions
**Priority:** T4
**Complexity:** XL
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 17.4; master_architecture.md, section M5

## Description
Implement undo/redo for all player placement actions (roads, zones, buildings, services). CS1 lacked this entirely and it was a constant source of frustration. CS2 attempted it but it was buggy.

- Command pattern: every player action creates an undoable command
- Command types: PlaceRoad, RemoveRoad, PaintZone, ClearZone, PlaceBuilding, RemoveBuilding, PlaceService
- Undo stack: last 50 commands
- Redo stack: cleared on new action
- Ctrl+Z / Ctrl+Y keybindings
- Each command stores: action type, affected cells, previous state, cost/refund

## Definition of Done
- [ ] Command pattern for all placement actions
- [ ] Undo reverts last action and refunds cost
- [ ] Redo re-applies undone action
- [ ] Stack depth of 50 commands
- [ ] Keybindings work

## Test Plan
- Integration: Place road, undo, verify road removed and cost refunded
- Integration: Undo 5 actions, redo 3, verify correct state

## Pitfalls
- Some actions have side effects (building spawn triggered by zone) -- undo zone must also remove building
- Road removal may break pathfinding for moving citizens
- Cost refund on undo could be exploited (build, get land value boost, undo)

## Relevant Code
- `crates/rendering/src/input.rs` -- command pattern integration
- `crates/simulation/src/grid.rs` -- state snapshots for undo

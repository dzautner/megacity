# INFRA-092: Undo/Redo for All Player Actions
**Priority:** T4
**Complexity:** XL (1-2 weeks)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Implement undo/redo for all player actions: road placement/removal, zone painting, building demolition, service placement, policy changes. Command pattern: each action creates a reversible command object with `execute()` and `undo()`. Undo stack with configurable depth (50 actions). Redo stack cleared on new action. Ctrl+Z for undo, Ctrl+Shift+Z for redo.

## Definition of Done
- [ ] Command pattern for all player actions
- [ ] Undo stack (depth 50+)
- [ ] Redo stack
- [ ] Ctrl+Z / Ctrl+Shift+Z keyboard shortcuts
- [ ] Undo/redo buttons in UI
- [ ] All placement tools support undo
- [ ] Tests pass

## Test Plan
- Unit: Place road, undo, road is removed, money refunded
- Unit: Undo then redo restores the road
- Unit: New action after undo clears redo stack

## Pitfalls
- Simulation state changes between undo/redo make perfect reversal impossible
- Undoing a demolition must restore building, citizens, etc.
- Multi-step actions (drawing a road curve) should undo as one unit

## Relevant Code
- `crates/rendering/src/input.rs` -- player actions
- New module for command pattern and undo/redo stack

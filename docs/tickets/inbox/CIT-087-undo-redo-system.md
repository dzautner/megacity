# CIT-087: Undo/Redo System for Player Actions

**Priority:** T4 (Polish)
**Complexity:** High (4-5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.18

## Description

Command pattern undo/redo for all player actions. Each action (road placement, zone paint, building placement, bulldoze, policy change) creates a reversible command. Undo stack (50 actions max). Redo stack (cleared on new action). Ctrl+Z/Ctrl+Y keyboard shortcuts. Undo for road placement removes road and restores grid state. Undo for zone paint restores previous zone type. Critical for player confidence in experimentation.

## Definition of Done

- [ ] `Command` trait with execute() and undo()
- [ ] Command implementations for: road, zone, service, utility, bulldoze, policy
- [ ] Undo stack (50 commands max)
- [ ] Redo stack (cleared on new action)
- [ ] Ctrl+Z and Ctrl+Y shortcuts
- [ ] Undo restores full state (grid cells, entities, budget)
- [ ] Visual feedback ("Undo: Road placement")
- [ ] Undo disabled during simulation (only for placement actions)

## Test Plan

- Unit test: place road -> undo -> road removed
- Unit test: undo 5 actions in sequence
- Unit test: redo after undo restores action
- Unit test: new action clears redo stack

## Pitfalls

- Some actions have cascading effects (road connects two areas; undo may break paths)
- State capture for undo must be lightweight (diff, not full snapshot)

## Relevant Code

- `crates/rendering/src/input.rs` (handle_tool_input)
- `crates/simulation/src/grid.rs` (WorldGrid mutations)

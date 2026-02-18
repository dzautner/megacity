# UX-001: Undo/Redo System for Player Actions

## Priority: T4 (Polish)
## Effort: Large (1-2 weeks)
## Source: camera_controls_ux.md -- Section 11.4: Undo/Redo, master_architecture.md T4

## Description
Implement command pattern for all player actions. ActionHistory with undo_stack and redo_stack (100 action limit). Composite actions for drag operations (road drag = 1 undo). Ctrl+Z undo, Ctrl+Y redo.

## Acceptance Criteria
- [ ] `ActionHistory` resource with undo/redo stacks
- [ ] `CityAction` enum: PlaceRoad, PlaceZone, PlaceBuilding, Bulldoze, PlaceUtility, TerrainEdit, Composite
- [ ] Ctrl+Z undoes last action
- [ ] Ctrl+Y / Ctrl+Shift+Z redoes
- [ ] Drag operations grouped as single Composite action
- [ ] 100-action limit with oldest dropped
- [ ] Treasury restored on undo

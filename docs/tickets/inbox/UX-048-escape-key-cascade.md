# UX-048: Escape Key Cascade (Cancel / Close / Deselect)

## Priority: T1 (Core Polish)
## Effort: Small (0.5 day)
## Source: camera_controls_ux.md -- Section 14.2: Keybindings

## Description
Escape should cascade: first cancel current tool action (road drawing), then close open panels, then deselect current selection, then deselect active tool.

## Acceptance Criteria
- [ ] Escape cancels active road drawing first
- [ ] Then closes open panels (info, budget, etc.)
- [ ] Then deselects selected entity
- [ ] Then deselects active tool (returns to Inspect/None)
- [ ] Each press handles one level of cascade

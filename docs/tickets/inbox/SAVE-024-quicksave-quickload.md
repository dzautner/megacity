# SAVE-024: Quick Save / Quick Load (F5/F9)

## Priority: T1 (Medium-Term)
## Effort: Small (0.5 day)
## Source: camera_controls_ux.md -- Keybindings

## Description
Add F5 for quick save and F9 for quick load. Uses a dedicated `quicksave.bin` slot. Provides instant save/load without file dialog.

## Acceptance Criteria
- [ ] F5 saves to `quicksave.bin` with status message
- [ ] F9 loads from `quicksave.bin` with confirmation
- [ ] Status message: "Quick saved" / "Quick loaded"
- [ ] F9 warns if no quicksave exists

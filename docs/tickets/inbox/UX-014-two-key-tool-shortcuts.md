# UX-014: Two-Key Tool Shortcuts (Category + Sub-Number)

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: camera_controls_ux.md -- Section 11.2: Tool Activation UX

## Description
Press category key (R for roads), then sub-number (3 for boulevard). Enables keyboard-only tool selection without mouse clicking. CS1 approach.

## Acceptance Criteria
- [ ] Category keys: R (roads), Z (zones), etc.
- [ ] After category key, number keys select sub-tools
- [ ] Visual feedback: category popup appears, sub-tools numbered
- [ ] Timeout: if no sub-key in 2s, popup closes
- [ ] Consistent with existing toolbar layout

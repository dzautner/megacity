# CAM-014: Logarithmic Zoom Speed Curve

## Priority: T1 (Core Polish)
## Effort: Small (0.5 day)
## Source: camera_controls_ux.md -- Section 3: Zoom

## Description
Zoom speed should be proportional to current distance (logarithmic). At close zoom, each scroll step makes a small change. At far zoom, each step covers more distance. This prevents jarring fast zooms at close range.

## Acceptance Criteria
- [ ] Zoom delta proportional to current distance
- [ ] Small steps at close zoom
- [ ] Larger steps at far zoom
- [ ] Smooth and natural-feeling

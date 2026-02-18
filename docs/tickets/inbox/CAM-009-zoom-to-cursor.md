# CAM-009: Zoom-to-Cursor (Zoom Toward Mouse Position)

## Priority: T1 (Core Polish)
## Effort: Small (1 day)
## Source: camera_controls_ux.md -- Section 3.4: Zoom Toward Cursor

## Description
Currently zoom changes distance from focus point, not toward cursor. Implement zoom-toward-cursor: when scrolling, the camera moves toward the ground point under the cursor. This is standard in CS1/CS2/Google Maps.

## Acceptance Criteria
- [ ] Scroll zoom moves camera toward ground point under cursor
- [ ] Focus point shifts toward cursor proportional to zoom delta
- [ ] Zoom out moves away from cursor position
- [ ] Smooth and natural-feeling at all zoom levels

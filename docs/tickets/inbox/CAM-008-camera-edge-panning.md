# CAM-008: Edge-of-Screen Panning

## Priority: T2 (Depth)
## Effort: Small (0.5 day)
## Source: camera_controls_ux.md -- Section 2: Camera Controls

## Description
When the mouse cursor is within 20 pixels of the screen edge, auto-pan the camera in that direction. Speed proportional to how close to the edge. Toggleable in settings.

## Acceptance Criteria
- [ ] Edge panning triggers within 20px of screen border
- [ ] Pan speed increases closer to edge
- [ ] Option to enable/disable in settings
- [ ] Does not trigger during tool drag operations

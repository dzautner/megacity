# CAM-013: Camera Bounds Clamping

## Priority: T1 (Core Polish)
## Effort: Small (0.5 day)
## Source: camera_controls_ux.md -- Section 2: Camera Controls

## Description
Clamp camera focus point to valid grid bounds. Prevent panning beyond the map edge by more than a small margin.

## Acceptance Criteria
- [ ] Camera focus clamped to grid bounds + margin (50 units)
- [ ] Cannot pan to see beyond map edge significantly
- [ ] Smooth clamping (not hard stop)
- [ ] Zoom distance clamped (min/max distance)

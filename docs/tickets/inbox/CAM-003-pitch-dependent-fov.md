# CAM-003: Pitch-Dependent FOV

## Priority: T2 (Depth)
## Effort: Small (0.5 day)
## Source: camera_controls_ux.md -- Section 3.3: FOV Adjustment

## Description
At low pitch (street level), use telephoto FOV (~30 degrees) for realism. At high pitch (bird's eye), use wide FOV (~60 degrees) for overview. Smoothly interpolated.

## Acceptance Criteria
- [ ] FOV dynamically adjusts based on pitch
- [ ] Low pitch: FOV ~30 degrees
- [ ] High pitch: FOV ~60 degrees
- [ ] Smooth FOV transition with pitch changes

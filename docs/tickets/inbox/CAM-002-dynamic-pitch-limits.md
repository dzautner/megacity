# CAM-002: Dynamic Pitch Limits Based on Zoom Level

## Priority: T1 (Core Polish)
## Effort: Small (0.5 day)
## Source: camera_controls_ux.md -- Section 3.2: Pitch Range Based on Zoom

## Description
Currently pitch is clamped to [5, 80] degrees at all zoom levels. At maximum zoom-out, low pitch creates horizon-clipping. At close zoom, near-vertical pitch should be allowed for street-level views.

## Acceptance Criteria
- [ ] Pitch min/max varies with zoom distance
- [ ] Far zoom: pitch min 30 degrees (steep only)
- [ ] Close zoom: pitch min 5 degrees (street level)
- [ ] Smooth transition of limits as zoom changes

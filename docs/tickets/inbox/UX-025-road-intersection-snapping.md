# UX-025: Road Intersection Snapping

## Priority: T1 (Core Polish)
## Effort: Small (1 day)
## Source: camera_controls_ux.md -- Section 12.6: Road Snapping

## Description
When cursor is near an existing intersection, snap road endpoint to exact intersection position. Prevents near-miss connections that look aligned but are not connected.

## Acceptance Criteria
- [ ] Snap radius: 1 cell distance from intersection
- [ ] Visual indicator when snapping (highlight dot)
- [ ] Snapped endpoint connects to existing intersection
- [ ] Works for both start and end points of new road

# UX-019: Road Curve Drawing Mode

## Priority: T2 (Depth)
## Effort: Medium (3-5 days)
## Source: camera_controls_ux.md -- Section 12.2: Road Drawing Modes

## Description
New road drawing mode where curves maintain C1 continuity with previous segment. Control point computed from previous tangent reflection across junction.

## Acceptance Criteria
- [ ] Curve mode toggle in road tool options
- [ ] New segment P1 = reflection of previous P2 across junction
- [ ] Smooth tangent continuity at junctions
- [ ] Visual preview of curve during placement

# UX-009: Enhanced Click-to-Select (Buildings, Roads, Citizens, Cells)

## Priority: T1 (Core Polish)
## Effort: Medium (2-3 days)
## Source: camera_controls_ux.md -- Section 10.1: Click-to-Select

## Description
Expand selection to support buildings (any cell in footprint), road segments (highlight Bezier), citizens (raycast against meshes), districts, and empty cells. Selection priority: citizens > buildings > roads > cells.

## Acceptance Criteria
- [ ] Multi-cell building: clicking any cell selects the building
- [ ] Road click: selects road segment, shows segment info
- [ ] Citizen click: raycast against visible citizen meshes
- [ ] Empty cell click: shows cell info (elevation, land value, zone, pollution)
- [ ] Priority order: citizens > buildings > roads > cells

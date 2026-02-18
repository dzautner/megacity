# UX-006: Cell Tooltips on Hover

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: camera_controls_ux.md -- Section 13.3: Tooltips on Hover

## Description
When hovering over any cell for 500ms, show a brief tooltip with relevant info: cell type, zone, elevation, building level, traffic density. No tooltip during drag operations.

## Acceptance Criteria
- [ ] Tooltip appears after 500ms hover delay
- [ ] Shows cell type, zone, elevation
- [ ] For buildings: type, level, occupancy
- [ ] For roads: type, traffic density
- [ ] Hidden during drag operations
- [ ] Positioned 20px offset from cursor

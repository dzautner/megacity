# UX-060: Road One-Way Direction Toggle

## Priority: T2 (Depth)
## Effort: Small (1 day)
## Source: camera_controls_ux.md -- Section 10.4: Context Menu

## Description
Right-click context menu or tool option to toggle a road segment to one-way. Arrow overlay shows traffic direction. Affects pathfinding (CSR graph edges become directed).

## Acceptance Criteria
- [ ] One-way toggle in road context menu
- [ ] Direction arrows displayed on one-way roads
- [ ] CSR graph updated with directed edges
- [ ] Pathfinding respects one-way direction
- [ ] Reversible (can toggle back to two-way)

# UX-012: Right-Click Context Menu

## Priority: T2 (Depth)
## Effort: Medium (2-3 days)
## Source: camera_controls_ux.md -- Section 10.4: Right-Click Context Menu

## Description
Right-click release (without drag) shows context menu for entity under cursor. Building: [Inspect, Bulldoze, Upgrade, Set Policy]. Road: [Inspect, Upgrade, Bulldoze, One-Way]. Citizen: [Follow, Details]. Empty: [Zone, Place Service].

## Acceptance Criteria
- [ ] Right-click without drag (< 5px movement) shows context menu
- [ ] Right-click with drag > 5px still orbits camera
- [ ] Context menu items vary by entity type
- [ ] Menu items trigger appropriate tool/action
- [ ] Menu closes on click outside or Escape

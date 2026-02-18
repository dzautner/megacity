# UX-007: Minimap

## Priority: T4 (Polish)
## Effort: Medium (3-5 days)
## Source: camera_controls_ux.md -- Section 13.4: Minimap, master_architecture.md T4

## Description
Corner minimap showing terrain, roads, buildings (zone-colored dots), and camera viewport rectangle. Click on minimap to jump camera. Rendered to offscreen texture via orthographic camera at low frequency (every 2-5 seconds).

## Acceptance Criteria
- [ ] Minimap rendered in configurable corner (default: bottom-right)
- [ ] Size: 150-250px configurable
- [ ] Shows terrain base colors, road lines, building dots
- [ ] Camera viewport shown as white rectangle
- [ ] Click on minimap moves camera focus with smooth transition
- [ ] Updated every 2-5 seconds (not real-time)
- [ ] Toggle visibility with keybind

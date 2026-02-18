# CAM-004: Camera Bookmarks (Save/Recall Positions)

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: camera_controls_ux.md -- Section 4: Camera Bookmarks

## Description
Allow saving up to 8 camera positions. Ctrl+F1..F8 saves current camera state (focus, yaw, pitch, distance). F1..F8 recalls with smooth transition.

## Acceptance Criteria
- [ ] `CameraBookmarks` resource with 8 slots
- [ ] Ctrl+F1..F8 saves current camera state to slot
- [ ] F1..F8 smoothly transitions to saved position
- [ ] Visual feedback: "Bookmark 1 saved" / "Bookmark 1 loaded"
- [ ] Bookmarks saved with city save file

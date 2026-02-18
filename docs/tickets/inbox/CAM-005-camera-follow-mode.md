# CAM-005: Camera Follow Mode (Track Entity)

## Priority: T4 (Polish)
## Effort: Medium (2-3 days)
## Source: camera_controls_ux.md -- Section 5: Follow Mode, master_architecture.md T4

## Description
Follow a citizen, service vehicle, or transit vehicle. Camera focus locked to entity position. Player retains orbit and zoom control. Toggle with "Follow" button or context menu.

## Acceptance Criteria
- [ ] `CameraFollowTarget(Entity)` resource
- [ ] Camera focus tracks entity position each frame
- [ ] Player can still orbit and zoom while following
- [ ] Follow broken by WASD, middle-click pan, or Escape
- [ ] Smooth transition when entering and exiting follow mode
- [ ] Follow button in entity info panel

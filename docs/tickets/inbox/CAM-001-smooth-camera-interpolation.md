# CAM-001: Smooth Camera Interpolation (Exponential Lerp)

## Priority: T1 (Core Polish)
## Effort: Small (1-2 days)
## Source: camera_controls_ux.md -- Section 2.1: Exponential Lerp

## Description
Camera currently applies position/rotation changes directly (no smoothing). Implement exponential lerp (critically damped spring) for all camera parameters: focus, yaw, pitch, distance. Frame-rate independent via `1 - e^(-speed * dt)`.

## Acceptance Criteria
- [ ] Camera focus point smoothly interpolated
- [ ] Yaw, pitch, and distance smoothly interpolated
- [ ] Frame-rate independent (same behavior at 30fps and 144fps)
- [ ] Configurable smoothing speed (default: 10.0)
- [ ] No visible snapping or stuttering

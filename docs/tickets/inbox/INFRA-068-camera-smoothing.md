# INFRA-068: Camera Smoothing and Bookmarks
**Priority:** T1
**Complexity:** S (hours)
**Dependencies:** none
**Source:** master_architecture.md, M2

## Description
Add exponential lerp smoothing to camera movement for professional feel. Camera position interpolates toward target each frame: `position = lerp(position, target, 1.0 - exp(-speed * dt))`. Add camera bookmarks: save/restore camera positions with keyboard shortcuts (Ctrl+1-9 to save, 1-9 to restore).

## Definition of Done
- [ ] Camera movement uses exponential lerp smoothing
- [ ] Smooth zoom with lerp (not instant jump)
- [ ] Camera bookmarks (save: Ctrl+1-9, restore: 1-9)
- [ ] Smoothing speed configurable
- [ ] Tests pass

## Test Plan
- Unit: Camera lerps toward target over multiple frames
- Unit: Bookmark save/restore preserves position and zoom

## Pitfalls
- exp(-speed * dt) needs dt clamping for large frame times
- Smoothing should not add input lag; tune speed parameter

## Relevant Code
- `crates/rendering/src/camera.rs` -- camera system
- `crates/rendering/src/input.rs` -- keyboard shortcuts

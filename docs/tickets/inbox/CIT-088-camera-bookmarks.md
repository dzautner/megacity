# CIT-088: Camera Bookmarks and Follow Mode

**Priority:** T4 (Polish)
**Complexity:** Low (1-2 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.18

## Description

Camera bookmark system: save/recall camera positions with keyboard shortcuts (1-9). Follow mode: select a citizen or service vehicle, camera tracks their movement. Useful for observing individual citizen behavior and service vehicle dispatch. Cinematic value for screenshots and videos.

## Definition of Done

- [ ] Save camera position to bookmark (Ctrl+1-9)
- [ ] Recall bookmark (1-9)
- [ ] 9 bookmark slots
- [ ] Follow mode: click citizen -> camera follows
- [ ] Follow mode: click service vehicle -> camera follows
- [ ] Follow mode exit on any camera input
- [ ] Bookmarks persist within session (optional: save to file)

## Test Plan

- Unit test: bookmark saves and recalls position correctly
- Unit test: follow mode tracks moving citizen
- Unit test: follow mode exits on camera input

## Pitfalls

- Following LOD-distant citizen may cause visual artifacts (LOD transition near camera)
- Follow mode must handle citizen despawn gracefully

## Relevant Code

- `crates/rendering/src/camera.rs` (OrbitCamera)
- `crates/rendering/src/input.rs`

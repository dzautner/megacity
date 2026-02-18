# CIT-090: First-Person City Walk Mode

**Priority:** T4 (Polish)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.18

## Description

Street-level camera mode for walking through the city. WASD to walk, mouse to look. Collision with buildings. See citizens at eye level. Experience traffic, parks, services from ground perspective. Photo mode: hide UI, adjust FOV, depth of field, color grading for screenshots. Primarily for player enjoyment and screenshot sharing, not gameplay.

## Definition of Done

- [ ] First-person camera at street level (height ~1.8m)
- [ ] WASD movement along roads and paths
- [ ] Mouse look for free camera rotation
- [ ] Building collision (can't walk through walls)
- [ ] Citizens visible at eye level
- [ ] LOD must show full detail near first-person camera
- [ ] Exit back to orbital view (Esc)
- [ ] Photo mode: UI hide, FOV slider, DOF toggle

## Test Plan

- Unit test: camera position at correct height
- Unit test: collision prevents entering buildings
- Visual test: city looks good from street level

## Pitfalls

- Buildings are procedural meshes that may look bad up close
- LOD transition visible when walking; needs hysteresis

## Relevant Code

- `crates/rendering/src/camera.rs` (OrbitCamera)
- `crates/rendering/src/building_render.rs`

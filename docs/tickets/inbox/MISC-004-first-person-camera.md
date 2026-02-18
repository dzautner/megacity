# MISC-004: First-Person Camera Walk Mode
**Priority:** T4
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M5; cities_skylines_analysis.md, section 14.6

## Description
Allow player to walk through their city at ground level in first-person perspective. This is the most viral/shareable feature for screenshots and videos. CS1 had this via mod; CS2 had it natively.

- Toggle: switch from overhead to first-person mode
- WASD movement at walking speed
- Mouse look for camera rotation
- Collision with buildings and roads
- Can follow a citizen's daily routine (camera follow mode)
- Photo mode: freeze time, adjust camera, take screenshot

## Definition of Done
- [ ] First-person camera mode toggleable
- [ ] WASD movement at ground level
- [ ] Mouse look
- [ ] Follow citizen mode
- [ ] Photo/screenshot mode

## Test Plan
- Integration: Toggle first-person, walk around city
- Integration: Follow citizen, verify camera tracks movement

## Pitfalls
- Ground-level rendering exposes LOD issues (buildings need detail up close)
- Collision detection with buildings on grid
- Camera clipping through terrain at elevation changes

## Relevant Code
- `crates/rendering/src/camera.rs` -- first-person camera mode
- `crates/rendering/src/input.rs` -- mode toggle and FP controls

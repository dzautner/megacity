# INFRA-094: First-Person City Walk Camera Mode
**Priority:** T4
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-016
**Source:** master_architecture.md, M5

## Description
Add first-person camera mode where player walks through the city at street level. Camera at 1.7m height, WASD movement along roads, collision with buildings. Follow mode: attach camera to a citizen and follow their commute. Photo/cinematic mode: free camera with depth of field, time of day control, UI hidden. Creates shareable screenshot moments.

## Definition of Done
- [ ] First-person walking mode with WASD controls
- [ ] Camera at ground level following terrain height
- [ ] Follow mode: track a selected citizen
- [ ] Photo mode: free camera, DOF controls, hide UI
- [ ] Toggle between normal and first-person modes
- [ ] Tests pass

## Test Plan
- Unit: First-person camera height matches terrain + 1.7m
- Unit: Follow mode tracks citizen along their path
- Integration: Screenshots from first-person mode look compelling

## Pitfalls
- LOD system may show simplified meshes at ground level; need full LOD for nearby
- Building interiors don't exist; camera must stay outside
- First-person movement speed needs to feel right (~5 km/h walking)

## Relevant Code
- `crates/rendering/src/camera.rs` -- camera system

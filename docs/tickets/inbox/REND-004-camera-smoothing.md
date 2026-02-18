# REND-004: Camera Smoothing (Exponential Lerp)
**Priority:** T1
**Complexity:** S
**Dependencies:** none
**Source:** master_architecture.md, section M2

## Description
Add exponential interpolation smoothing to camera movement. Currently camera movement feels abrupt. Smooth panning, zooming, and rotation using exponential decay toward target position.

- Camera position lerps toward target: pos += (target - pos) * (1 - exp(-speed * dt))
- Zoom smoothing: same exponential lerp for zoom level
- Rotation smoothing: same for camera angle
- Speed parameter: configurable (default 8.0 for snappy but smooth)
- No smoothing when mouse button released (instant stop, not drift)

## Definition of Done
- [ ] Camera panning is smooth
- [ ] Zoom transitions are smooth
- [ ] Rotation is smooth
- [ ] Camera feels responsive but not jerky
- [ ] Smoothing speed configurable

## Test Plan
- Visual: Camera movement feels professional and polished
- Integration: Camera stops immediately when input stops (no drift)

## Pitfalls
- Too much smoothing = input lag feel; too little = no effect
- Exponential lerp is frame-rate independent (important for variable frame rates)
- camera.rs is in rendering crate

## Relevant Code
- `crates/rendering/src/camera.rs` -- add smoothing to camera update

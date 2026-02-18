# INFRA-143: Custom Asset Pipeline (Buildings, Vehicles)
**Priority:** T5
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-106
**Source:** master_architecture.md, M6

## Description
Enable modders to import custom 3D assets (building meshes, vehicle models) into the game. Asset pipeline: import (glTF/OBJ format), validate (poly count limits, texture size, correct scale), convert (to engine-internal format), load (at runtime). Asset hot-reload for development. Asset catalog with metadata (type, size, LOD variants).

## Definition of Done
- [ ] glTF/OBJ import pipeline
- [ ] Validation (poly budget, texture size, scale check)
- [ ] Runtime loading from mod folders
- [ ] Hot-reload for development
- [ ] Asset catalog with metadata
- [ ] Tests pass

## Test Plan
- Unit: Valid glTF asset loads and renders correctly
- Unit: Over-budget asset rejected with clear error message
- Integration: Custom building appears in-game when mod loaded

## Pitfalls
- Arbitrary mesh loading is a security concern (malformed files)
- Performance: high-poly custom assets can tank framerate
- LOD generation for custom assets may be needed

## Relevant Code
- `crates/rendering/src/building_meshes.rs` -- mesh loading patterns

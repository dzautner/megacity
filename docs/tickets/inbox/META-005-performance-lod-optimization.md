# META-005: Performance LOD and Rendering Optimization

**Category:** Meta / Performance
**Priority:** T1
**Source:** game_design_mechanics.md -- Section 9

## Summary

Per-mesh LOD with 3+ levels. Citizen LOD: full model -> simplified -> dots -> aggregate. Building LOD: full -> reduced -> box -> merged chunk. Proper culling (frustum, occlusion, distance). LOD budget: 70-90% triangle reduction. Avoid CS2 anti-patterns (rendering teeth at all distances).

## Acceptance Criteria

- [ ] 3-level mesh LOD for buildings
- [ ] Citizen LOD with aggregate at far zoom
- [ ] Frustum and distance culling
- [ ] 3-5x GPU performance from LOD alone

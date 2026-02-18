# INFRA-024: Underground Infrastructure Mesh Rendering
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-022
**Source:** underground_infrastructure.md, Visual Language section

## Description
Create mesh rendering for underground infrastructure types. Each type has distinct color and style: water mains (blue cylinders), sewer trunks (brown/olive cylinders, larger), storm drains (grey dashed), power cables (red/orange thin with glow), metro tunnels (dark grey rectangular), metro stations (white platform rectangles), utility tunnels (purple rectangular). Add `UndergroundMesh` component with `infrastructure_type` and `depth_layer`. Alpha-code by depth in AllUnderground view.

## Definition of Done
- [ ] `UndergroundMesh` component with type and depth layer
- [ ] Distinct mesh/color per infrastructure type
- [ ] Meshes spawned when player builds underground infrastructure
- [ ] Visibility controlled by `UndergroundViewState`
- [ ] Tests pass

## Test Plan
- Unit: Each infrastructure type produces mesh with correct color
- Integration: Underground view shows all types with distinct visual identity

## Pitfalls
- Generating cylinder/box meshes at runtime for every pipe segment is expensive; use instanced rendering
- Color coding must be colorblind-accessible (use pattern + color, not just color)

## Relevant Code
- `crates/rendering/src/building_meshes.rs` -- mesh generation patterns
- `crates/rendering/src/overlay.rs` -- color scheme reference

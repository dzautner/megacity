# INFRA-022: Underground View/Layer Rendering System
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-019
**Source:** underground_infrastructure.md, Underground View/Layer System

## Description
Implement depth-based underground view system with tab bar: [Surface] [Shallow] [Medium] [Deep] [All Underground]. Each tab adjusts rendering: surface terrain/buildings become transparent, underground infrastructure becomes visible. `ViewLayer` enum and `UndergroundViewState` resource control rendering. Terrain alpha: Surface=1.0, Shallow=0.30, Medium=0.15, Deep=0.10. Buildings alpha similarly reduced. Underground meshes visible based on depth layer match.

## Definition of Done
- [ ] `ViewLayer` enum: Surface, Shallow, Medium, Deep, AllUnderground
- [ ] `UndergroundViewState` resource tracks active layer
- [ ] Terrain material alpha changes with view layer
- [ ] Building material alpha changes with view layer
- [ ] Underground infrastructure meshes show/hide based on layer
- [ ] Tab bar UI for layer switching
- [ ] Keyboard shortcut for layer cycling
- [ ] Tests pass

## Test Plan
- Unit: ViewLayer::Surface hides all underground meshes
- Unit: ViewLayer::Shallow shows shallow infrastructure, hides medium/deep
- Integration: Switching layers is visually smooth, no flickering

## Pitfalls
- AlphaMode::Blend has Z-sorting issues in Bevy; may need separate render pass
- Switching layers every frame is expensive; only update on `is_changed()`
- Transparent terrain needs to still receive shadows for spatial reference

## Relevant Code
- `crates/rendering/src/terrain_render.rs` -- terrain material alpha
- `crates/rendering/src/building_render.rs` -- building material alpha
- `crates/ui/src/toolbar.rs` -- layer tab bar UI

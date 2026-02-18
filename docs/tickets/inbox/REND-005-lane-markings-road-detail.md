# REND-005: Road Lane Markings and Intersection Detail
**Priority:** T2
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section 5.2

## Description
Add lane markings to road rendering. Currently roads are flat-colored rectangles. Lane markings (white/yellow lines), crosswalks, and intersection detail make the city feel professional.

- Lane markings: white dashed center lines for 2-lane, solid lines for dividers
- Yellow markings: opposite direction dividers
- Crosswalks: at intersections where pedestrians cross
- Turn arrows: at intersection approach lanes
- Road edge lines: white solid at road edges
- Intersection detail: turn lane indicators, stop lines
- All rendered as additional geometry on road mesh (not decals)

## Definition of Done
- [ ] Lane markings visible on all road types
- [ ] Center line, edge line, divider line distinct
- [ ] Crosswalks at intersections
- [ ] Markings appropriate per road type

## Test Plan
- Visual: Roads look professional with lane markings
- Visual: Different road types have appropriate markings

## Pitfalls
- Road mesh generation in road_render.rs needs additional UV coordinates
- Curved roads (Bezier segments) need UV-mapped markings
- Too many markings = visual clutter at far zoom

## Relevant Code
- `crates/rendering/src/road_render.rs` -- add marking geometry
- `crates/simulation/src/grid.rs:RoadType` -- lane count for marking count

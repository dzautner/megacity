# INFRA-149: Road Lane Markings and Intersection Detail
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, Section 5.2 (Rendering Crate)

## Description
Add lane markings to road rendering: center line (yellow for two-way, no line for one-way), lane dividers (white dashed), edge lines (solid white), turn arrows at intersections. Intersection detail: crosswalks, stop lines, signal indicators. Markings improve visual clarity and help player understand road function. Markings scale with zoom level.

## Definition of Done
- [ ] Center line rendering (yellow double for two-way, none for one-way)
- [ ] Lane divider dashed lines
- [ ] Edge lines
- [ ] Crosswalks at intersections
- [ ] Markings visible at medium zoom, hidden at far zoom
- [ ] Tests pass

## Test Plan
- Unit: Two-way road has yellow center line
- Unit: Highway has more lane markings than local road
- Integration: Roads look professional with lane markings at close zoom

## Pitfalls
- Lane marking geometry generation is complex for curved Bezier roads
- Performance: too many line segments for large road networks
- Markings at intersections where roads meet need special handling

## Relevant Code
- `crates/rendering/src/road_render.rs` -- road rendering

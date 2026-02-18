# TRAF-002: Level of Service (LOS) A-F Traffic Grading
**Priority:** T1
**Complexity:** M
**Dependencies:** TRAF-001
**Source:** cities_skylines_analysis.md, section 6.2; master_architecture.md, section M2

## Description
Implement Level of Service grading (A-F) for road segments based on volume/capacity ratio. Display as color-coded overlay on roads. This gives players actionable feedback on traffic problems.

- LOS A: V/C < 0.35 (free flow, green)
- LOS B: V/C 0.35-0.55 (mostly free, light green)
- LOS C: V/C 0.55-0.75 (stable flow, yellow)
- LOS D: V/C 0.75-0.90 (approaching unstable, orange)
- LOS E: V/C 0.90-1.00 (at capacity, red)
- LOS F: V/C > 1.00 (breakdown, dark red/black)
- Color-code road cells in traffic overlay based on LOS
- City-wide average LOS as headline metric in traffic panel

## Definition of Done
- [ ] LOS grade computed per road cell
- [ ] Traffic overlay color-coded by LOS
- [ ] City-wide average LOS displayed
- [ ] LOS grade visible when hovering over road
- [ ] LOS D-F roads highlighted as problems

## Test Plan
- Unit: Road with 0 traffic = LOS A
- Unit: Road at capacity = LOS E
- Integration: Create traffic jam, verify LOS F coloring

## Pitfalls
- Volume measurement must be smoothed (instantaneous count is noisy)
- Capacity depends on road type and intersection spacing
- LOS should be computed on slow tick to avoid per-frame cost

## Relevant Code
- `crates/simulation/src/traffic.rs` -- compute LOS per cell
- `crates/rendering/src/overlay.rs` -- LOS overlay rendering
- `crates/rendering/src/road_render.rs` -- optional LOS coloring on roads

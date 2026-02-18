# INFRA-031: Level of Service (LOS) A-F Traffic Grading
**Priority:** T1
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-030
**Source:** transportation_simulation.md, Section 1.5; master_architecture.md, M2

## Description
Implement Highway Capacity Manual LOS grading for road segments based on V/C ratio. LOS A: V/C < 0.35 (free flow), LOS B: 0.35-0.55, LOS C: 0.55-0.77, LOS D: 0.77-0.93, LOS E: 0.93-1.00 (at capacity), LOS F: >1.00 (breakdown). Display LOS as road color in traffic overlay (green A through red F). Add LOS to road info panel. Compute per-road-segment, not per-cell.

## Definition of Done
- [ ] `LevelOfService` enum: A, B, C, D, E, F
- [ ] LOS computed per road segment from V/C ratio
- [ ] Traffic overlay colors roads by LOS (green to red gradient)
- [ ] Road info panel shows LOS grade
- [ ] City-wide LOS distribution shown in stats
- [ ] Tests pass

## Test Plan
- Unit: V/C=0.3 -> LOS A, V/C=0.9 -> LOS D, V/C=1.2 -> LOS F
- Integration: Visual traffic overlay shows congested corridors in red

## Pitfalls
- V/C must be computed from actual traffic volume vs capacity, not just density count
- LOS at intersections differs from mid-block LOS (intersection delay dominates)
- Need to decide if LOS includes intersection delay or just link delay

## Relevant Code
- `crates/simulation/src/traffic.rs` -- traffic density, add LOS computation
- `crates/rendering/src/overlay.rs` -- traffic overlay color coding
- `crates/ui/src/info_panel.rs` -- road info display

# TRAF-009: Barcelona Superblock District Policy
**Priority:** T3
**Complexity:** M
**Dependencies:** TRAF-002
**Source:** urban_planning_zoning.md, section 4.5

## Description
Implement superblock as a district policy. Converting interior roads to pedestrian paths within a 3x3 block area creates massive happiness/land value bonuses but reduces road capacity and increases perimeter traffic.

- Player designates superblock center and radius (10-12 cells)
- Interior roads converted to RoadType::Path (speed 5, no vehicles, zero noise)
- Perimeter roads remain unchanged
- Effects: +8-12 happiness inside, +15-25 land value, -10-20 noise, -5-10 pollution
- Penalty: +20-40% congestion on perimeter roads
- Green space gained: ~70% of interior road surface
- Best for: high residential density areas with good perimeter road capacity

## Definition of Done
- [ ] Superblock tool designates area
- [ ] Interior roads converted to pedestrian
- [ ] Happiness and land value bonuses applied
- [ ] Perimeter traffic increases visible
- [ ] Superblock can be reverted

## Test Plan
- Integration: Apply superblock to residential area, verify happiness increase
- Integration: Verify perimeter road traffic density increases

## Pitfalls
- Must not convert roads that are the ONLY connection between areas (connectivity check)
- Citizens inside superblock must still be reachable by emergency vehicles
- Reversing superblock must restore original road types

## Relevant Code
- `crates/simulation/src/policies.rs` -- superblock policy
- `crates/simulation/src/grid.rs:Cell` -- road type modification
- `crates/simulation/src/happiness.rs` -- superblock happiness bonus
- `crates/simulation/src/land_value.rs` -- superblock LV bonus

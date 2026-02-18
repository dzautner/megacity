# META-006: Simulation LOD and Tiered Detail

**Category:** Meta / Performance
**Priority:** T1
**Source:** game_design_mechanics.md -- Section 9.3

## Summary

Tiered simulation detail: individual (near camera), group (same building/vehicle), statistical (distant districts), growing agents (one represents many). Nearby chunks simulate every tick; distant every Nth tick. Spatial partitioning for O(1) neighbor queries.

## Acceptance Criteria

- [ ] Near-camera citizens fully simulated
- [ ] Distant districts use aggregate statistics
- [ ] Simulation tick rate varies by distance
- [ ] Performance maintained at 100K+ citizens

# WEATHER-009: Urban Heat Island Mitigation Measures

## Priority: T3 (Differentiation)

## Description
Implement UHI mitigation options: tree planting, green roofs, cool (white) roofs, cool pavement, parks, water features, permeable surfaces, building energy efficiency, and district cooling. Each reduces UHI by a specific amount per cell.

## Current State
- Trees reduce pollution but not temperature directly.
- No green roof, cool roof, or cool pavement building variants.
- No district cooling system.

## Definition of Done
- [ ] Tree planting: -1.5F UHI per tree cell.
- [ ] Green roofs: -2.0F, building upgrade costing $15K/building.
- [ ] Cool (white) roofs: -1.5F, $3K/building.
- [ ] Cool pavement: -1.0F, road upgrade $5K/cell.
- [ ] Parks: -3.0F in radius, $10K/cell.
- [ ] Water features (fountains): -2.0F, decorative placement $8K.
- [ ] Permeable surfaces: -0.5F, $4K/cell.
- [ ] District cooling: -1.0F, large facility $50K.
- [ ] Each mitigation reduces `UhiGrid` values in affected cells.

## Test Plan
- [ ] Unit test: green roof reduces UHI by 2.0F at building location.
- [ ] Unit test: park reduces UHI in a radius around placement.
- [ ] Integration test: comprehensive mitigation in downtown lowers temperature measurably.

## Pitfalls
- Depends on WEATHER-008 (UHI grid).
- Building upgrades require a building upgrade UI system.
- Cool pavement as a road variant needs integration with road system.

## Code References
- `crates/simulation/src/trees.rs`: tree effects (pattern to extend)
- Research: `environment_climate.md` section 4.5.4

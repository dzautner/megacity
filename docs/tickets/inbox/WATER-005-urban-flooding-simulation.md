# WATER-005: Urban Flooding Simulation and Depth-Damage Curves

## Priority: T2 (Depth)

## Description
Implement the shallow-water flooding simulation that spreads excess stormwater across the terrain when drainage capacity is exceeded. Includes depth-damage curves that translate flood depth to building damage percentages, and drainage infrastructure (storm drains, retention ponds).

## Current State
- `DisasterType::Flood` exists but uses a simple radius + elevation threshold model.
- No dynamic water level simulation.
- No depth-damage curves.
- No drainage capacity concept.

## Definition of Done
- [ ] `FloodGrid` resource (f32 per cell, depth in feet).
- [ ] Shallow-water flood simulation: 5 iterations per tick, water flows from high to low elevation.
- [ ] Drainage capacity: each cell has a drain rate (0 without storm drains, configurable with infrastructure).
- [ ] Depth-damage curves by building type: residential 0%/10%/35%/65%/90% at 0/1/3/6/10 ft.
- [ ] Commercial and industrial curves with different values.
- [ ] Flood damage applied to buildings when depth > 0.5 ft for residential.
- [ ] Flood depth overlay for rendering.
- [ ] FloodGrid activated only during active flooding events.

## Test Plan
- [ ] Unit test: depth-damage curve returns correct percentages at boundary values.
- [ ] Unit test: water flows from higher to lower cells.
- [ ] Integration test: heavy rain in a valley causes flooding that damages buildings.
- [ ] Integration test: storm drains prevent flooding at moderate rainfall.

## Pitfalls
- Shallow-water simulation is computationally expensive (~2ms per iteration at 5 iterations).
- Must only allocate FloodGrid during active events to save memory.
- Existing `DisasterType::Flood` needs refactoring or replacement.

## Code References
- `crates/simulation/src/disasters.rs`: existing `DisasterType::Flood`
- `crates/simulation/src/grid.rs`: `Cell.elevation`
- Research: `environment_climate.md` sections 2.4.1-2.4.3

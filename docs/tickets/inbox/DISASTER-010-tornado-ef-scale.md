# DISASTER-010: Tornado Enhanced Fujita (EF) Scale and Path Simulation

## Priority: T2 (Depth)

## Description
Replace the current simple radius-based tornado with a path-following tornado using the Enhanced Fujita (EF) scale. Tornadoes have a starting point, direction, path length, width, and wobble, with damage determined by EF rating and building resilience.

## Current State
- `DisasterType::Tornado` uses a fixed radius (5 cells) and 30% destruction chance.
- No path simulation (damage is circular around center).
- No EF rating system.
- No building resilience factor.

## Definition of Done
- [ ] EF rating distribution: EF0(40%), EF1(25%), EF2(20%), EF3(10%), EF4(4%), EF5(1%).
- [ ] Path generation: start cell (typically west edge), direction ± 30 degrees, length = 5 + EF*10 + random(20), width = 1 + EF cells.
- [ ] Forward speed: 2-5 cells per tick.
- [ ] Path wobble: direction += random ± 10 degrees per cell.
- [ ] Damage by EF: EF0=5%, EF1=20%, EF2=45%, EF3=70%, EF4=90%, EF5=99%.
- [ ] Building resilience: WoodFrame=0.5, Masonry=0.7, ReinforcedConc=0.85, SteelFrame=0.9.
- [ ] Edge-of-path intensity reduction: damage scales with distance from center.
- [ ] Trees destroyed in path.
- [ ] Power lines downed in path.

## Test Plan
- [ ] Unit test: EF0 tornado destroys only 5% of buildings in path.
- [ ] Unit test: EF5 tornado destroys 99% of all building types except underground.
- [ ] Unit test: path length increases with EF rating.
- [ ] Integration test: tornado leaves a visible path of destruction across the map.

## Pitfalls
- Must replace existing `DisasterType::Tornado` logic or heavily refactor.
- Path simulation requires multi-tick processing (tornado advances each tick).
- Wobble can cause tornado to double back, hitting cells twice.

## Code References
- `crates/simulation/src/disasters.rs`: `DisasterType::Tornado`, `TORNADO_RADIUS`
- Research: `environment_climate.md` sections 5.4.1-5.4.3

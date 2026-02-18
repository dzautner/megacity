# WEATHER-015: Fog and Visibility System

## Priority: T3 (Differentiation)

## Description
Implement fog as a weather condition that reduces visibility, slows traffic, and affects airport operations. Fog forms when humidity is high and temperature drops near the dew point, especially in early morning near water bodies.

## Current State
- No fog weather condition.
- No visibility concept.
- No humidity tracking.

## Definition of Done
- [ ] Fog condition when humidity > 90% and temperature within 2C of dew point.
- [ ] More likely near water cells and in early morning (hours 4-8).
- [ ] Traffic speed reduction: -20% in fog.
- [ ] Airport operations: heavy fog suspends flights.
- [ ] Visual rendering: fog particle effect or distance fog shader.
- [ ] Duration: typically 2-4 game-hours, burns off by midday.

## Test Plan
- [ ] Unit test: fog forms at high humidity + near dew point.
- [ ] Unit test: fog dissipates as temperature rises.
- [ ] Integration test: early morning fog near rivers.

## Pitfalls
- Requires humidity and dew point tracking (WEATHER-001 dependency).
- Airport suspension requires airport operations system.
- Rendering fog is a shader feature, not simulation.

## Code References
- `crates/simulation/src/weather.rs`: weather conditions
- `crates/rendering/src/terrain_render.rs`: visual effects
- Research: `environment_climate.md` section 4.1

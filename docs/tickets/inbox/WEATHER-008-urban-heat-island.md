# WEATHER-008: Urban Heat Island (UHI) Effect Grid

## Priority: T2 (Depth)

## Description
Implement the Urban Heat Island effect as a per-cell temperature increment grid. Dense urban areas with dark surfaces and reduced vegetation are 2-8C warmer than surrounding rural areas. UHI is amplified at night (2x) and mitigated by trees, parks, green roofs, and cool pavement.

## Current State
- No UHI system exists.
- Temperature is uniform across the entire map.
- Trees reduce pollution but not temperature.

## Definition of Done
- [ ] `UhiGrid` resource (f32 per cell, temperature increment in Fahrenheit).
- [ ] Surface heat factor: asphalt/dark roof=+2F, concrete=+1.5F, light roof=+0.5F, green roof=-1F, water=-2F, vegetation=-1.5F.
- [ ] Vegetation deficit: compare local green fraction to rural baseline (0.6), UHI += deficit * 8.0F.
- [ ] Waste heat: proportional to energy demand density.
- [ ] Canyon effect: buildings > 4 stories add height-to-width ratio * 1.5F.
- [ ] Nighttime amplification: UHI *= 2.0 at night.
- [ ] Smooth grid (3x3 average) to prevent sharp boundaries.
- [ ] Final cell temperature = base_weather_temperature + UHI_grid[x][y].
- [ ] Update frequency: every 30 ticks.

## Test Plan
- [ ] Unit test: dense urban area has +5-10F UHI increment.
- [ ] Unit test: park area has negative UHI (cooling effect).
- [ ] Unit test: nighttime UHI is double daytime.
- [ ] Integration test: downtown area is measurably warmer than suburbs.
- [ ] Integration test: planting trees reduces UHI.

## Pitfalls
- UHI affects all temperature-dependent systems (heating demand, heat wave mortality, etc.).
- Per-cell temperature becomes more expensive to compute than global temperature.
- Must balance UHI magnitude so it is noticeable but not overwhelming.

## Code References
- `crates/simulation/src/trees.rs`: `TreeGrid`
- `crates/simulation/src/weather.rs`: `Weather.temperature`
- Research: `environment_climate.md` sections 4.5.1-4.5.4

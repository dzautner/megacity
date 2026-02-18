# POLL-007: Water Pollution Non-Point Source Runoff

## Priority: T3 (Differentiation)

## Description
Implement diffuse non-point source (NPS) water pollution that activates during rainfall events. Agricultural land, construction sites, paved roads, parking lots, and industrial yards all contribute pollution proportional to rainfall intensity and impervious surface percentage.

## Current State
- No rain-activated pollution sources exist.
- Water pollution is continuous, not weather-dependent.
- No impervious surface calculation per cell.
- No construction site pollution.

## Definition of Done
- [ ] NPS load formula: `base_pollution * rainfall_intensity * (0.3 + 0.7 * imperviousness)`.
- [ ] NPS source table: agricultural=3.0, construction=5.0, paved roads=1.5, parking=2.0, lawns=1.0, industrial yard=4.0, landfill unlined=6.0, landfill lined=0.5.
- [ ] NPS only activates during Rain and Storm weather events.
- [ ] Imperviousness derived from cell land use type.
- [ ] Continuous leaching from unlined landfills (not rain-dependent).
- [ ] NPS pollution fed into downstream flow model (POLL-005).

## Test Plan
- [ ] Unit test: NPS load is zero during clear weather.
- [ ] Unit test: NPS load scales with rainfall intensity.
- [ ] Integration test: rainstorm causes pollution spike in downstream water bodies.
- [ ] Integration test: unlined landfill creates continuous water pollution.

## Pitfalls
- Depends on POLL-005 (downstream flow model) for proper routing.
- Construction sites are not currently tracked as a cell state.
- Agricultural zones do not exist in the current `ZoneType` enum.

## Code References
- `crates/simulation/src/water_pollution.rs`
- `crates/simulation/src/weather.rs`: `WeatherEvent::Rain`, `WeatherEvent::Storm`
- Research: `environment_climate.md` section 1.2.2

# WEATHER-002: Climate Zone Map Presets

## Priority: T2 (Depth)

## Description
Implement climate zone presets that shift all seasonal parameters for different map types. The research doc defines 7 climate zones (Temperate, Tropical, Arid, Mediterranean, Continental, Subarctic, Oceanic) with distinct temperature ranges, rain patterns, and snow behavior.

## Current State
- Only one climate zone exists (temperate-like with Spring 15C, Summer 28C, etc.).
- No map-specific climate settings.
- No climate zone resource or configuration.

## Definition of Done
- [ ] `ClimateZone` enum with 7 variants.
- [ ] Each zone defines: winter_low, summer_high, rain_pattern, snow_enabled, base_precipitation_chance per season.
- [ ] `ClimateZone` resource set at map generation time.
- [ ] All weather calculations reference climate zone parameters instead of hardcoded values.
- [ ] Map selection screen shows climate zone information.
- [ ] Temperate as default (backward-compatible).

## Test Plan
- [ ] Unit test: Tropical zone has winter_low=65F, no snow.
- [ ] Unit test: Subarctic zone has winter_low=-30F, heavy snow.
- [ ] Unit test: Arid zone has very low precipitation chance.
- [ ] Integration test: changing climate zone changes weather patterns dramatically.

## Pitfalls
- Extreme zones (Subarctic, Arid) may require rebalancing of heating/water systems.
- Some zones disable snow entirely; must handle snow-dependent systems gracefully.
- Climate zone affects optimal building strategies, which may frustrate unprepared players.

## Code References
- `crates/simulation/src/weather.rs`: `Weather`, `Season`
- Research: `environment_climate.md` section 4.1.2

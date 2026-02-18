# WEATHER-004: Seasonal and Weather Tourism Modifiers

## Priority: T2 (Depth)

## Description
Connect the tourism system to weather and seasons with specific multipliers. Summer is peak tourism (1.5x), storms suppress tourism (0.2x), sunny weather boosts it (1.2x). This makes tourism revenue seasonal and weather-dependent.

## Current State
- `tourism.rs` exists with tourism systems.
- `Weather::park_multiplier()` provides a weather-driven park effectiveness modifier.
- No explicit tourism seasonal/weather modifier function.

## Definition of Done
- [ ] `tourism_seasonal_modifier(season, weather)` function per research doc.
- [ ] Base seasonal: Summer=1.5, Spring=1.2, Autumn=1.1, Winter=0.6.
- [ ] Weather modifier: Sunny=1.2, PartlyCloudy=1.0, Overcast=0.8, Rain=0.5, Storm=0.2, Snow=0.7, Extreme=0.1.
- [ ] Combined modifier applied to daily tourist arrival rate.
- [ ] Winter tourism bonus for ski resort amenities (if applicable).
- [ ] Weather-related tourism events (festivals in good weather, closures in bad).

## Test Plan
- [ ] Unit test: summer + sunny = 1.5 * 1.2 = 1.8x tourism.
- [ ] Unit test: winter + storm = 0.6 * 0.2 = 0.12x tourism.
- [ ] Integration test: tourism revenue peaks in summer, drops in winter.

## Pitfalls
- Current `WeatherEvent` enum may not have all required conditions; depends on WEATHER-001.
- Snow can be positive for winter tourism (ski resorts); need conditional logic.
- Tourism system must exist and be connected.

## Code References
- `crates/simulation/src/tourism.rs`: tourism systems
- `crates/simulation/src/weather.rs`: `Weather`, `Season`
- Research: `environment_climate.md` section 4.3.2

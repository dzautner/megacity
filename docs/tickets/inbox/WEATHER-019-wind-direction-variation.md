# WEATHER-019: Improved Wind Direction and Speed Variation

## Priority: T2 (Depth)

## Description
Enhance the wind system with more realistic variation. Wind direction should follow prevailing patterns with random perturbation, and speed should vary with weather events. Currently wind changes randomly and uniformly.

## Current State
- `WindState` in `wind.rs` has direction (0-360) and speed (0-1).
- `update_wind` changes direction by random +-10 degrees and speed by +-0.05 every 30 ticks.
- No prevailing wind direction.
- No weather-event-driven wind.

## Definition of Done
- [ ] Prevailing wind direction: defined per climate zone (default: westerly, 270 degrees).
- [ ] Direction reverts toward prevailing with `(prevailing - current) * 0.1` per update.
- [ ] Random perturbation: +-5 degrees per slow tick (calmer than current +-10).
- [ ] Storm wind: speed boosted to 0.6-0.9 during Storm events.
- [ ] Calm periods: speed drops to 0.0-0.1 during Clear + high pressure.
- [ ] Wind gust events: temporary speed spikes (1-2 ticks) during transitions.
- [ ] Diurnal variation: afternoon winds (12-18) are 20% stronger than morning.

## Test Plan
- [ ] Unit test: wind direction trends toward prevailing over time.
- [ ] Unit test: storm event increases wind speed to 0.6+.
- [ ] Unit test: calm clear weather produces low wind speed.
- [ ] Integration test: pollution plumes shift direction based on prevailing wind.

## Pitfalls
- Prevailing wind direction differs by climate zone; must coordinate with WEATHER-002.
- Wind speed boosts during storms must coordinate with WEATHER-011 (storm damage).
- Diurnal wind variation adds realism but may complicate solar/wind power predictions.

## Code References
- `crates/simulation/src/wind.rs`: `WindState`, `update_wind`
- `crates/simulation/src/weather.rs`: `WeatherEvent`

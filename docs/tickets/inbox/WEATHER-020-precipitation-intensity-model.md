# WEATHER-020: Precipitation Intensity Variation

## Priority: T2 (Depth)

## Description
Replace the binary rain on/off system with variable precipitation intensity. Rainfall varies from drizzle (0.1 in/hr) to torrential (4+ in/hr). Intensity affects stormwater runoff, flood risk, fire suppression, agriculture, and solar power output.

## Current State
- `WeatherEvent::Rain` and `WeatherEvent::Storm` are binary (on/off).
- No rainfall intensity value.
- No precipitation accumulation tracking.

## Definition of Done
- [ ] `Weather.precipitation_intensity: f32` (inches per hour, 0.0 to 4.0).
- [ ] Intensity categories: None(0), Drizzle(0.01-0.1), Light(0.1-0.25), Moderate(0.25-1.0), Heavy(1.0-2.0), Torrential(2.0-4.0), Extreme(4.0+).
- [ ] Rain events set intensity based on season and storm type.
- [ ] Accumulation: `daily_rainfall += intensity * hours_of_rain`.
- [ ] Rolling 30-day rainfall total for drought calculation (WEATHER-010).
- [ ] Intensity affects: stormwater runoff volume, fire suppression rate, solar output reduction, traffic speed.

## Test Plan
- [ ] Unit test: storm event produces higher intensity than rain event.
- [ ] Unit test: daily accumulation matches intensity * duration.
- [ ] Integration test: heavy rain causes more stormwater than drizzle.
- [ ] Integration test: 30-day rolling average tracks drought conditions.

## Pitfalls
- Must not break existing rain-dependent systems that check `WeatherEvent::Rain`.
- Intensity needs to be deterministic (based on day hash) for reproducibility.
- Extreme rainfall (4+ in/hr) is very rare; used for flash flood events.

## Code References
- `crates/simulation/src/weather.rs`: `Weather`, `WeatherEvent`
- Research: `environment_climate.md` section 4.1

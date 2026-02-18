# DISASTER-023: Fire Weather Index (Red Flag Days)

## Priority: T2 (Depth)

## Description
Implement a Fire Weather Index (FWI) combining temperature, humidity, wind speed, and drought index to predict wildfire risk. High FWI triggers "Red Flag" warnings that increase fire ignition chance. Fire weather monitoring ($20K) provides early warning.

## Current State
- `forest_fire.rs` checks `is_hot` (temperature > 30C) and `is_storm` for lightning.
- No composite fire weather index.
- No red flag warning system.

## Definition of Done
- [ ] `fire_weather_index = temperature_factor * (1.0 - humidity_factor) * wind_factor * drought_factor`.
- [ ] Temperature factor: (T - 20) / 20, clamped 0-1.
- [ ] Humidity factor: humidity / 100, clamped 0-1 (high humidity = low risk).
- [ ] Wind factor: wind_speed * 2.0, clamped 0-2.
- [ ] Drought factor: 1.0 / drought_index, clamped 1-5.
- [ ] FWI > 0.5: moderate risk (Yellow), > 1.0: high risk (Orange), > 2.0: extreme (Red Flag).
- [ ] Red Flag: fire ignition probability multiplied by 5x.
- [ ] Fire weather monitoring building: $20K, provides FWI forecast and red flag warnings.
- [ ] UI: FWI indicator and red flag banner when extreme.

## Test Plan
- [ ] Unit test: hot, dry, windy day produces FWI > 2.0.
- [ ] Unit test: cool, humid, calm day produces FWI < 0.2.
- [ ] Integration test: red flag day increases fire ignition rate.
- [ ] Integration test: fire weather monitoring provides advance warning.

## Pitfalls
- Depends on humidity tracking (WEATHER-001) and drought index (WEATHER-010).
- FWI scaling must match actual fire ignition probability in forest_fire.rs.
- Red flag warning is only useful if player can take preventive action (close campgrounds, pre-position firefighters).

## Code References
- `crates/simulation/src/forest_fire.rs`: ignition probability
- `crates/simulation/src/weather.rs`: temperature, weather conditions
- Research: `environment_climate.md` section 5.3.8 (Fire weather monitoring)

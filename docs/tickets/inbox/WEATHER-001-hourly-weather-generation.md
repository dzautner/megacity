# WEATHER-001: Hourly Weather Generation with Diurnal Temperature Curve

## Priority: T1 (Core)

## Description
Replace the current daily weather update with an hourly system that generates realistic diurnal temperature curves. The research doc specifies temperature varying hour-by-hour with a diurnal factor (peak at 15:00, minimum at 06:00), smooth transitions between conditions, and separate humidity/cloud_cover tracking.

## Current State
- `update_weather` runs once per game-day, not hourly.
- Temperature is base + sin variation (not a realistic diurnal curve).
- No humidity, cloud_cover, or precipitation_intensity fields.
- Weather events are binary (on/off), no smooth transitions.
- 360-day year with 90 days per season.

## Definition of Done
- [ ] `Weather` resource gains: `humidity: f32`, `cloud_cover: f32`, `precipitation_intensity: f32`.
- [ ] Weather system runs on game-hour boundary (via `GameClock.hour`).
- [ ] Diurnal temperature curve: `T(hour) = T_min + (T_max - T_min) * diurnal_factor(hour)`.
- [ ] `diurnal_factor(hour)` peaks at 15:00 (1.0), minimum at 06:00 (0.0).
- [ ] Smooth transitions: `temperature += (target - temperature) * 0.3` per hour.
- [ ] Weather event planning at midnight: roll for rain/snow, extreme events.
- [ ] Visual weather condition derived from cloud_cover, precipitation, and temperature.
- [ ] Condition enum expanded: Sunny, PartlyCloudy, Overcast, Rain, HeavyRain, Snow, Storm.

## Test Plan
- [ ] Unit test: diurnal factor peaks at hour 15 and troughs at hour 6.
- [ ] Unit test: temperature transitions smoothly between hours.
- [ ] Unit test: precipitation conditions map correctly (rain above 32F, snow below 32F).
- [ ] Integration test: temperature rises through morning, peaks afternoon, drops overnight.

## Pitfalls
- Changing from daily to hourly requires `GameClock` to track hours reliably.
- 360-day year with seasons already works; must not break existing seasonal logic.
- Smooth transitions may cause weather to "lag" -- ensure events start promptly.

## Code References
- `crates/simulation/src/weather.rs`: `Weather`, `update_weather`, `WeatherEvent`
- `crates/simulation/src/time_of_day.rs`: `GameClock`
- Research: `environment_climate.md` sections 4.1, 4.5.5

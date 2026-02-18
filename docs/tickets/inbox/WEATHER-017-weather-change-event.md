# WEATHER-017: Weather Change Event for Cross-System Communication

## Priority: T2 (Depth)

## Description
Implement a `WeatherChangeEvent` Bevy event that fires whenever weather conditions change. Other systems (solar output, tourism, construction, fire risk) can listen to this event instead of polling the weather state every tick.

## Current State
- Systems read `Weather` resource directly every tick to check conditions.
- No event-driven notification when weather changes.
- Unnecessary work when weather hasn't changed.

## Definition of Done
- [ ] `WeatherChangeEvent` with `old_condition`, `new_condition`, `is_extreme` flag.
- [ ] Fired when: weather event starts/ends, season changes, temperature crosses threshold.
- [ ] Solar/wind systems listen for this event to recalculate output.
- [ ] Fire risk system listens for rain/storm events.
- [ ] Construction system listens for storm halts.
- [ ] UI notification for extreme weather changes.

## Test Plan
- [ ] Unit test: event fired when weather transitions from Clear to Rain.
- [ ] Unit test: event includes `is_extreme = true` for HeatWave, ColdSnap, Storm.
- [ ] Integration test: solar system recalculates output on weather change.

## Pitfalls
- Event must fire on the same tick as the weather change (not delayed).
- Must not fire spuriously when weather state doesn't actually change.
- Too many listeners may cause performance issues; keep event lightweight.

## Code References
- `crates/simulation/src/weather.rs`: `update_weather`
- Research: `environment_climate.md` section 8.4

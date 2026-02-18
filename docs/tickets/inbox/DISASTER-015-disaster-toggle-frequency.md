# DISASTER-015: Disaster Toggle and Frequency Settings

## Priority: T1 (Core)

## Description
Expand the disaster enable/disable setting to per-disaster-type toggles and frequency multipliers. Players should be able to customize which disasters are active and how frequently they occur.

## Current State
- `Weather.disasters_enabled` is a single boolean toggle for all disasters.
- `DISASTER_CHANCE = 0.0005` is a constant.
- No per-type frequency adjustment.
- No settings UI for disaster configuration.

## Definition of Done
- [ ] `DisasterSettings` resource with per-type enable/disable flags.
- [ ] Frequency multiplier per disaster type: 0x (off), 0.5x (rare), 1x (normal), 2x (frequent), 5x (extreme).
- [ ] Default: all enabled at 1x frequency.
- [ ] Settings accessible from game options / disaster preparedness panel.
- [ ] Serialize/deserialize disaster settings in save files.
- [ ] Frequency affects roll threshold: `DISASTER_CHANCE * frequency_multiplier`.

## Test Plan
- [ ] Unit test: disabled disaster type never triggers.
- [ ] Unit test: 2x frequency doubles the trigger rate.
- [ ] Integration test: settings persist through save/load.

## Pitfalls
- Must not break existing single-toggle behavior during transition.
- Extreme frequency (5x) may overwhelm unprepared cities.
- Settings UI needs to be discoverable but not cluttering normal gameplay.

## Code References
- `crates/simulation/src/disasters.rs`: `DISASTER_CHANCE`, `trigger_random_disaster`
- `crates/simulation/src/weather.rs`: `Weather.disasters_enabled`
- `crates/save/src/serialization.rs`: save/load

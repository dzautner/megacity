# POLL-022: Pollution Alert Event System

## Priority: T2 (Depth)

## Description
Implement pollution alert events that notify the player when pollution levels exceed health thresholds. Alerts for air quality (AQI > 150), water quality (tier > Polluted), noise complaints (>80 dB residential), and soil contamination discovery.

## Current State
- No pollution alert system.
- No event notifications for environmental thresholds.

## Definition of Done
- [ ] `PollutionAlertEvent` Bevy event with `pollution_type`, `severity`, `affected_area`.
- [ ] Air quality alert: triggered when any residential cell exceeds AQI 150 for 3+ ticks.
- [ ] Water quality alert: triggered when drinking water source quality drops below Clean tier.
- [ ] Noise complaint: triggered when residential cells exceed 80 dB for extended period.
- [ ] Soil contamination alert: triggered when soil contamination discovered near residential.
- [ ] Alert levels: Advisory, Warning, Emergency.
- [ ] UI notification with action suggestions (build treatment plant, plant trees, etc.).
- [ ] Event log records all pollution alerts with timestamps.

## Test Plan
- [ ] Unit test: air quality alert triggers at AQI 150.
- [ ] Unit test: alert severity escalates with pollution level.
- [ ] Integration test: new factory near residential triggers air quality advisory.

## Pitfalls
- Alert fatigue: too many alerts will be ignored by players.
- Threshold values need tuning so alerts are meaningful but not constant.
- Must not alert for transient spikes (require sustained exceedance).

## Code References
- Research: `environment_climate.md` section 8.4 (Event System)

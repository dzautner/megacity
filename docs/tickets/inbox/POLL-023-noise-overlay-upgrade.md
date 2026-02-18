# POLL-023: Noise Pollution Overlay with dB Labels

## Priority: T2 (Depth)

## Description
Update the noise pollution overlay to display decibel values and use a color scheme matching the 7-tier noise classification. Tooltip shows dB level, tier name (Quiet/Normal/Noticeable/Loud/Very Loud/Painful/Dangerous), and health advisory.

## Current State
- Noise overlay exists but uses arbitrary 0-100 scale.
- No dB labels or tier classification in the overlay.
- No tooltip with noise level details.

## Definition of Done
- [ ] Color ramp: 7 tiers from green (Quiet, <45 dB) to red/black (Dangerous, >110 dB).
- [ ] Tooltip: dB level, tier name, land value effect, health advisory.
- [ ] Legend showing tier colors and dB ranges.
- [ ] Consistent with POLL-020 (AQI overlay style).

## Test Plan
- [ ] Visual test: quiet residential areas show green, highways show orange/red.
- [ ] Visual test: tooltip shows correct dB and tier.

## Pitfalls
- Depends on POLL-010 (logarithmic noise model) for meaningful dB values.
- Until POLL-010 is implemented, overlay must handle the current 0-100 scale.

## Code References
- `crates/rendering/src/overlay.rs`: noise overlay
- Research: `environment_climate.md` section 1.3.7

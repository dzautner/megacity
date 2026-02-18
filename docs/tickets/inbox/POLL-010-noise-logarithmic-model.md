# POLL-010: Noise Pollution Logarithmic Attenuation Model

## Priority: T2 (Depth)

## Description
Replace the current linear Manhattan-distance decay noise model with a proper logarithmic attenuation model (6 dB per doubling of distance + atmospheric absorption). The research doc specifies `L(d) = L_source - 6.0 * log2(distance) - 0.5 * distance` with decibel-based units.

## Current State
- `NoisePollutionGrid` uses u8 (0-100) with linear Manhattan distance decay.
- Road noise: flat values per road type (Highway=25, Boulevard=15, etc.).
- Industrial noise: 20 base, decays by `3 * distance`.
- Airport noise: radius-based, decays by `3 * distance`.
- No logarithmic attenuation.
- No barrier attenuation (buildings blocking noise).
- Sources combine additively instead of logarithmically.

## Definition of Done
- [ ] Noise levels stored in actual dB scale (35-105 range).
- [ ] Source levels per research doc table (17 source types from 35-105 dB).
- [ ] Distance attenuation: `L = L_source - 6.0 * log2(dist) - 0.5 * dist`.
- [ ] Dominant source approximation (use max instead of log addition for performance).
- [ ] Ambient baseline: 35 dB everywhere (quiet natural level).
- [ ] Max propagation radius precomputed per source level.
- [ ] Time-of-day activity patterns (daytime only, nighttime only, 24h).

## Test Plan
- [ ] Unit test: noise drops ~6 dB per doubling of distance from a point source.
- [ ] Unit test: two equal sources combine to +3 dB.
- [ ] Unit test: dominant source (10+ dB louder) effectively masks smaller source.
- [ ] Integration test: highway produces higher noise than local road.
- [ ] Performance test: full grid noise calculation with 500 sources completes within 5ms.

## Pitfalls
- Changing from linear 0-100 to dB scale changes overlay color mapping.
- `log2` operations per cell per source can be expensive; consider lookup tables.
- The dominant-source approximation introduces up to 3 dB error near multiple sources.

## Code References
- `crates/simulation/src/noise.rs`: `NoisePollutionGrid`, `update_noise_pollution`
- Research: `environment_climate.md` sections 1.3.1-1.3.6

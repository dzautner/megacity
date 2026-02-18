# POLL-019: Complete Noise Source Type Table (17 Sources)

## Priority: T2 (Depth)

## Description
Implement the full 17-source noise emission table from the research doc. Currently only 4 source types are implemented (road types, industrial, airport). The doc specifies source levels from 35 dB (park) to 105 dB (jet takeoff), with different activity patterns (daytime, nighttime, 24h).

## Current State
- `NoisePollutionGrid` tracks 4 source types with linear decay.
- Roads: Highway=25, Boulevard=15, Avenue=10, Local=5 (arbitrary units, not dB).
- Industrial: 20 base.
- Airport: 25 base.
- No construction, nightclub, train, power plant, or stadium noise.

## Definition of Done
- [ ] Source level table (dB): Highway=75, Arterial=70, Local Road=55, Rail Corridor=80, Airport=105, Construction=90, Heavy Industry=85, Light Industry=70, Commercial HVAC=60, Nightclub=95, Fire Station=80, Power Plant=75, Stadium=95, School=70, Park=35, Parking Structure=65, Train Station=75.
- [ ] Activity patterns: Daytime(06-22), Nighttime(22-06), 24h, Event-driven.
- [ ] Each source tagged with its activity pattern.
- [ ] Inactive sources produce no noise outside their activity window.

## Test Plan
- [ ] Unit test: highway noise is 75 dB at source.
- [ ] Unit test: nightclub noise only active during nighttime.
- [ ] Unit test: park contributes 35 dB ambient baseline.
- [ ] Integration test: construction site is noisy during daytime, quiet at night.

## Pitfalls
- Many source types require corresponding building types that may not exist yet.
- Stadium and nightclub as event-driven noise sources need event scheduling.
- 17 source types with dB levels requires the logarithmic model (POLL-010) to be meaningful.

## Code References
- `crates/simulation/src/noise.rs`: `update_noise_pollution`
- Research: `environment_climate.md` section 1.3.2

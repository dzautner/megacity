# POLL-016: Airport and Seaport Air Pollution Sources

## Priority: T3 (Differentiation)

## Description
Implement airport and seaport as significant area air pollution sources. Airports emit Q=25 for jet exhaust and ground operations; seaports emit Q=20 for ship diesel and cargo handling. Both also produce major noise pollution.

## Current State
- Airport exists as a building type.
- Airport noise exists (radius-based, 25 base, decays by distance*3).
- No airport air pollution.
- Seaport may not exist yet.

## Definition of Done
- [ ] Airport air pollution: Q=25.0, area source covering airport footprint.
- [ ] Airport noise: already exists but should use dB model (POLL-010).
- [ ] Seaport air pollution: Q=20.0, area source.
- [ ] Seaport noise: 85 dB (24h operation).
- [ ] Both contribute to AQI in surrounding areas.

## Test Plan
- [ ] Unit test: airport emits Q=25 area pollution.
- [ ] Integration test: area around airport has elevated AQI.

## Pitfalls
- Airport and seaport are large area sources, not point sources.
- Must integrate with the dispersion kernel differently (area vs point).

## Code References
- `crates/simulation/src/noise.rs`: airport noise
- `crates/simulation/src/airport.rs`: airport systems
- Research: `environment_climate.md` section 1.1.2

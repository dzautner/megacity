# DISASTER-001: Earthquake Magnitude-to-MMI Intensity System

## Priority: T2 (Depth)

## Description
Replace the current simple radius-based earthquake damage with a proper magnitude-to-Modified Mercalli Intensity (MMI) system. MMI decreases with distance from the epicenter, is amplified by soft soil, and determines damage probability per building. The research doc provides Gutenberg-Richter magnitude distribution and MMI attenuation formulas.

## Current State
- `DisasterType::Earthquake` uses a simple radius (10 cells) and 10% destroy chance.
- No magnitude concept (all earthquakes are equal).
- No distance-based intensity attenuation.
- No soil amplification.
- No building construction type vulnerability.

## Definition of Done
- [ ] Magnitude generated from inverse power law: `magnitude = -log10(random()) + 3.0`, clamped 3.0-9.0.
- [ ] Depth: random 5-50 km, shallow = more damage.
- [ ] MMI per cell: `base_mmi = 1.5 * magnitude - 1.0`, attenuated by `2.5 * log10(distance_km / depth_km)`.
- [ ] Distance conversion: `distance_km = cell_distance * 0.05`.
- [ ] Soil amplification: bedrock=+0, firm=+0.5, soft=+1.0, fill=+1.5, liquefaction=+2.0.
- [ ] MMI grid calculated once per earthquake event.
- [ ] Damage probability table by MMI level (simplified 4-column version for 2-3 construction types).

## Test Plan
- [ ] Unit test: M7 earthquake at epicenter = MMI IX.
- [ ] Unit test: MMI drops by ~1 per doubling of distance.
- [ ] Unit test: soft soil amplifies MMI by 1.0.
- [ ] Integration test: M5 earthquake causes minimal damage; M7 causes widespread destruction.

## Pitfalls
- Soil type is not currently tracked per cell; needs to be added or derived from elevation.
- log10 operations per cell during earthquake are fine (one-time event).
- Must replace or wrap existing `DisasterType::Earthquake` logic.

## Code References
- `crates/simulation/src/disasters.rs`: `DisasterType::Earthquake`, `process_active_disaster`
- Research: `environment_climate.md` sections 5.1.1-5.1.3

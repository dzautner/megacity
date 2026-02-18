# DISASTER-012: Volcanic Eruption Event System

## Priority: T3 (Differentiation)

## Description
Implement volcanic events as a map-specific disaster type. Ranges from fumaroles (constant) to minor eruptions (ash fall) to major eruptions (pyroclastic flow, lava flow). Only available on maps with volcanic terrain features.

## Current State
- No volcanic disaster type.
- No volcanic terrain feature.
- No ash fall or lava flow simulation.

## Definition of Done
- [ ] `DisasterType::VolcanicEruption` with magnitude (minor, moderate, major).
- [ ] Ash fall: `ash_depth = magnitude * 10.0 / (1.0 + distance^1.5)`, wind-carried downwind.
- [ ] Ash effects by depth: 0-1cm nuisance, 1-5cm road closures, 5-15cm structural risk, 15-30cm building collapse, 30cm+ destruction zone.
- [ ] Lava flow: follows D8 downhill at 0.1 cells/tick, destroys everything in path, creates new terrain after solidifying.
- [ ] Fumaroles: constant minor effect, creates tourism attraction.
- [ ] Map-specific: only on maps with volcanic terrain feature flag.
- [ ] Air quality: hazardous AQI during ash fall events.

## Test Plan
- [ ] Unit test: ash depth decreases with distance from volcano.
- [ ] Unit test: lava follows downhill path.
- [ ] Integration test: minor eruption causes ash damage in radius.
- [ ] Integration test: major eruption destroys buildings in lava path.

## Pitfalls
- Volcanic terrain must be pre-defined in map generation.
- Lava changing terrain permanently is a major feature.
- Very rare event; may never trigger in a normal game.

## Code References
- `crates/simulation/src/disasters.rs`: disaster framework
- Research: `environment_climate.md` sections 5.5.1-5.5.3

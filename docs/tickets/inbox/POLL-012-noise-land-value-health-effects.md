# POLL-012: Noise Pollution Land Value and Health Effects

## Priority: T2 (Depth)

## Description
Implement the 7-tier noise effects system from the research doc. Noise levels map to land value modifiers (from +10% for quiet to -80% for dangerous), health effects (stress, hearing risk), and sleep disruption effects with a nighttime penalty multiplier.

## Current State
- Noise pollution affects happiness through the service coverage system but not directly via tiered effects.
- No land value modifier from noise.
- No health damage from noise.
- No nighttime noise penalty.

## Definition of Done
- [ ] 7-tier noise classification: Quiet/Normal/Noticeable/Loud/Very Loud/Painful/Dangerous.
- [ ] Land value modifiers: +10% (quiet) to -80% (dangerous) applied in `land_value.rs`.
- [ ] Health effects: stress and hearing risk at loud+ levels.
- [ ] Sleep effect: noise in residential cells 50% worse between 22:00-06:00 game time.
- [ ] Noise mitigation measures: noise barrier walls, speed limits, truck route restrictions, curfews, quiet asphalt, depressed highways.
- [ ] Each mitigation measure as a placeable item or policy toggle.

## Test Plan
- [ ] Unit test: quiet area gets +10% land value bonus.
- [ ] Unit test: nighttime noise penalty correctly doubles effect.
- [ ] Integration test: building near highway has reduced land value.
- [ ] Integration test: noise barrier wall reduces land value penalty.

## Pitfalls
- Nighttime penalty requires checking game clock hour, which is in `GameClock` resource.
- Land value is u8 (0-255); percentage modifiers must work within that range.
- Multiple noise mitigation policies may stack; define interaction rules.

## Code References
- `crates/simulation/src/noise.rs`: `NoisePollutionGrid`
- `crates/simulation/src/land_value.rs`: `LandValueGrid`
- `crates/simulation/src/time_of_day.rs`: `GameClock`
- Research: `environment_climate.md` sections 1.3.7, 1.3.8

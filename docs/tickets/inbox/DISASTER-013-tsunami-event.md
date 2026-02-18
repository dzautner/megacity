# DISASTER-013: Tsunami Event System

## Priority: T3 (Differentiation)

## Description
Implement tsunami events triggered by offshore earthquakes (M > 7.0). Wave height depends on magnitude with coastal amplification. Inundation penetrates inland based on terrain slope. Warning systems can reduce casualties.

## Current State
- No tsunami disaster type.
- No coastal inundation simulation.

## Definition of Done
- [ ] Tsunami triggered by offshore earthquake M > 7.0.
- [ ] Wave height: `(magnitude - 6.0) * 8.0` ft base, amplified in shallow water.
- [ ] Inundation: wave penetrates inland, height decreasing with terrain slope * 2.0.
- [ ] Momentum damage: 50% more damage than equivalent static flooding.
- [ ] Warning time: near-field 10-30 min, far-field 2-8 hours.
- [ ] Warning system: reduces casualties by 80-95%.
- [ ] Depth-damage curves from flood system applied with momentum multiplier.

## Test Plan
- [ ] Unit test: M7 tsunami = 8 ft wave height.
- [ ] Unit test: wave attenuates with uphill terrain.
- [ ] Integration test: tsunami damages coastal buildings.
- [ ] Integration test: warning system allows evacuation.

## Pitfalls
- Requires coastal map terrain (not all maps have coasts).
- Offshore earthquake source needs to be modeled (outside map bounds).
- Wave height amplification in shallow water is complex physics.

## Code References
- Research: `environment_climate.md` section 5.6

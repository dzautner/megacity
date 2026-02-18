# CIT-029: Happiness Factor -- Environmental Quality

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 5.2

## Description

Environmental quality happiness component combining: air pollution (negative, weight 0.35), noise pollution (negative, weight 0.25), water pollution (negative, weight 0.15), greenery/trees (positive, weight 0.15), weather (seasonal, weight 0.10). Formula: env_quality = 1.0 - air_pollution*0.35 - noise*0.25 - water_pollution*0.15 + greenery*0.15 + weather_comfort*0.10. All inputs normalized 0-1 from their grid values. Weight in overall happiness: 0.10.

## Definition of Done

- [ ] `compute_environmental_quality()` combining 5 factors
- [ ] Air pollution from PollutionGrid, normalized 0-1
- [ ] Noise from NoisePollutionGrid, normalized 0-1
- [ ] Water pollution from WaterPollutionGrid, normalized 0-1
- [ ] Greenery from TreeGrid (count nearby trees / max)
- [ ] Weather comfort from temperature delta from ideal (22C)
- [ ] Weight of 0.10 in overall happiness

## Test Plan

- Unit test: zero pollution + full greenery + good weather = ~0.85 quality
- Unit test: max pollution = ~0.0 quality
- Integration test: planting trees near homes improves environmental happiness

## Pitfalls

- Current happiness already penalizes pollution separately; consolidate to avoid double-counting

## Relevant Code

- `crates/simulation/src/happiness.rs` (update_happiness, pollution penalty)
- `crates/simulation/src/pollution.rs` (PollutionGrid)
- `crates/simulation/src/noise.rs` (NoisePollutionGrid)
- `crates/simulation/src/trees.rs` (TreeGrid)

# POLL-021: City Environmental Score Aggregate Metric

## Priority: T2 (Depth)

## Description
Implement a single aggregate "Environmental Score" metric (0-100) that summarizes the city's environmental health across all pollution types, green coverage, energy mix, waste management, and water quality. Displayed prominently on the city dashboard.

## Current State
- No aggregate environmental metric.
- Individual pollution overlays exist but no combined score.
- No green coverage percentage.
- No energy mix analysis.

## Definition of Done
- [ ] Environmental Score = weighted average of: air quality (25%), water quality (20%), noise (15%), soil health (10%), green coverage (15%), energy cleanliness (15%).
- [ ] Each sub-score normalized to 0-100 scale.
- [ ] Air quality sub-score based on average city AQI (0 AQI = 100 score, 500 AQI = 0 score).
- [ ] Green coverage = percentage of cells with trees/parks.
- [ ] Energy cleanliness = fraction of power from renewables.
- [ ] Score visible on main dashboard and in yearly statistics.
- [ ] Achievement triggers: "Green City" at score > 80, "Eco Champion" at score > 95.

## Test Plan
- [ ] Unit test: all-clean city scores near 100.
- [ ] Unit test: all-polluted city scores near 0.
- [ ] Integration test: planting trees increases environmental score.
- [ ] Integration test: switching from coal to solar increases score.

## Pitfalls
- Weighting of sub-scores is a game design decision; may need tuning.
- Score computation should be infrequent (yearly) to avoid performance impact.
- Must not double-count effects (e.g., air quality already affects health).

## Code References
- Research: `environment_climate.md` section 7.2 (Anno 2070 ecobalance reference)

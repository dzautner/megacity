# WASTE-005: Landfill Capacity and Environmental Effects

## Priority: T1 (Core)

## Description
Implement landfill as the default waste destination with finite capacity, environmental effects, and post-closure requirements. Each landfill cell holds ~50,000 tons. An 8x8 landfill holds 3.2M tons and lasts 16-78 years depending on diversion.

## Current State
- No landfill building type.
- No waste capacity tracking.
- No landfill environmental effects.

## Definition of Done
- [ ] Landfill building: 8x8 footprint, 1,000 tons/day capacity, $5M build, $3K/day operating.
- [ ] Capacity: 50,000 tons per cell = 3,200,000 tons total for 8x8.
- [ ] `Landfill` struct with `current_fill_tons`, `total_capacity_tons`, fill tracking.
- [ ] `years_remaining()` estimate based on current daily input.
- [ ] Environmental effects by type: Unlined (high groundwater pollution, large odor radius), Lined (low pollution, moderate odor), Lined+Collection (minimal pollution, small odor).
- [ ] Landfill gas: ~1 MW per 1,000 tons/day if gas collection enabled.
- [ ] Land value penalty: -40% (unlined) to -15% (lined + collection) in radius.
- [ ] Odor radius: 15 cells (unlined) to 5 cells (lined + collection).
- [ ] When full: must cap ($10K/cell), 30-year monitoring ($50K/year), site becomes park after 30+ years.

## Test Plan
- [ ] Unit test: years_remaining calculation correct for given input rate.
- [ ] Unit test: fill_fraction increases daily.
- [ ] Integration test: landfill fills up over time and triggers capacity warning.
- [ ] Integration test: unlined landfill contaminates nearby groundwater.

## Pitfalls
- 8x8 footprint is very large (64 cells); may need smaller options.
- Post-closure 30-year tracking is a very long game-time commitment.
- Landfill gas capture creates energy (interaction with POWER system).

## Code References
- `crates/simulation/src/groundwater.rs`: groundwater contamination
- Research: `environment_climate.md` sections 6.3.1-6.3.3

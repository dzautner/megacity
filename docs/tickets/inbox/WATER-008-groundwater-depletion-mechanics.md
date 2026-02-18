# WATER-008: Groundwater Depletion and Sustainability Mechanics

## Priority: T2 (Depth)

## Description
Enhance the existing groundwater system with depletion warnings, well yield reduction when water table drops, land subsidence from over-extraction, and sustainability metrics. Currently wells draw down groundwater but there are no consequences beyond level changes.

## Current State
- `GroundwaterGrid` tracks levels (0-255) and quality (0-255).
- Well pumps draw down groundwater in radius 4.
- Rain replenishes groundwater.
- `GroundwaterStats` tracks averages but no depletion warnings.

## Definition of Done
- [ ] Well yield reduction: wells produce less water when groundwater level < 50 (diminishing returns).
- [ ] Critical depletion warning when average groundwater < 20% (GROUNDWATER_CRITICAL_LEVEL).
- [ ] Land subsidence: cells with groundwater < 20 for extended periods lose elevation permanently.
- [ ] Sustainability metric: extraction rate vs recharge rate ratio displayed.
- [ ] Over-extraction indicator in groundwater overlay.
- [ ] Recharge boost from constructed recharge basins (new building type).

## Test Plan
- [ ] Unit test: well yield at 50% = half normal output.
- [ ] Unit test: subsidence triggers when groundwater < 20 for 50+ slow ticks.
- [ ] Integration test: heavy pumping without rain depletes groundwater.
- [ ] Integration test: recharge basin increases groundwater recovery rate.

## Pitfalls
- Permanent elevation change from subsidence requires modifying `Cell.elevation`, which affects rendering.
- Subsidence is a one-way process (cannot un-subside), which may confuse players.
- Must balance recharge rates so groundwater is manageable but not trivial.

## Code References
- `crates/simulation/src/groundwater.rs`: `GroundwaterGrid`, `update_groundwater`, `GroundwaterStats`
- `crates/simulation/src/grid.rs`: `Cell.elevation`
- Research: `environment_climate.md` section 2.2

# POWER-009: Energy Dispatch (Merit Order) System

## Priority: T1 (Core)

## Description
Implement the merit order dispatch algorithm that determines which generators run to meet demand. Generators are dispatched in order of marginal cost: cheapest first, most expensive last. This determines the electricity price and which plants are profitable.

## Current State
- No dispatch system exists.
- No generation order or marginal cost concept.

## Definition of Done
- [ ] `dispatch_energy()` system that runs every 4 ticks.
- [ ] Merit order: sort generators by marginal cost (fuel + variable O&M).
- [ ] Dispatch order: renewables ($0) -> nuclear ($10) -> coal ($30) -> gas ($40) -> gas peaker ($80) -> battery discharge.
- [ ] Each generator's `current_output_mw` set by dispatch.
- [ ] Reserve margin calculation: `(supply - demand) / demand`.
- [ ] Blackout logic: when demand > available supply, shed load by priority tier.
- [ ] Rolling blackout: rotate affected cells each tick during deficit.
- [ ] Electricity price = marginal cost of the last dispatched unit * scarcity multiplier.

## Test Plan
- [ ] Unit test: cheapest generators dispatched first.
- [ ] Unit test: expensive generators only run when demand exceeds cheaper supply.
- [ ] Unit test: reserve margin < 0 triggers blackout.
- [ ] Integration test: city with only solar at night triggers gas plant dispatch.
- [ ] Integration test: rolling blackout rotates affected areas.

## Pitfalls
- Must handle the case where no generators exist (new city).
- Rolling blackout cell rotation needs to be deterministic.
- Reserve margin calculation must consider battery storage availability.

## Code References
- Research: `environment_climate.md` sections 3.3, 3.4

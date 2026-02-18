# WASTE-010: Uncollected Waste Health, Happiness, and Pest Effects

## Priority: T2 (Depth)

## Description
Implement penalties for uncollected waste including health risks, happiness reduction, vermin/pest attraction, land value decline, and visual indicators. Uncollected waste accumulates at buildings and in streets, creating a visible decline in city quality.

## Current State
- No uncollected waste concept.
- No waste-related health or happiness penalties.

## Definition of Done
- [ ] Uncollected waste tracked per cell/building as accumulated lbs.
- [ ] Health penalty: -0.5 health/slow_tick per 100 lbs accumulated nearby.
- [ ] Happiness penalty: -5 per building with uncollected waste.
- [ ] Vermin/pest risk: cells with > 500 lbs uncollected have pest events.
- [ ] Land value penalty: -20% in radius of accumulated waste.
- [ ] Visual indicator: garbage pile render at buildings with excess waste.
- [ ] Waste decays very slowly (0.5% per day) representing natural breakdown.
- [ ] Public health crisis event triggered if > 20% of buildings have uncollected waste.

## Test Plan
- [ ] Unit test: health penalty scales with waste accumulation.
- [ ] Unit test: land value drops near waste accumulation.
- [ ] Integration test: removing waste collection causes progressive deterioration.
- [ ] Integration test: restoring collection clears accumulated waste over time.

## Pitfalls
- Visual garbage piles need art/rendering support.
- Pest events are a new event type requiring implementation.
- Must not punish players too harshly for temporary collection gaps.

## Code References
- `crates/simulation/src/happiness.rs`: happiness calculations
- `crates/simulation/src/land_value.rs`: land value effects
- Research: `environment_climate.md` sections 6.2, 6.3.2

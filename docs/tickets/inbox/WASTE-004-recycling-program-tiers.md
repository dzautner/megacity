# WASTE-004: Recycling Program Tiers and Economics

## Priority: T2 (Depth)

## Description
Implement tiered recycling programs from "No program" (5% diversion) to "Zero waste goal" (60% diversion). Each tier has different participation rates, costs per household, and revenue from selling recyclables. Recycling commodity prices fluctuate.

## Current State
- No recycling system.
- No diversion tracking.
- No recyclable material revenue.

## Definition of Done
- [ ] Recycling program tiers: None(5%), VoluntaryDropoff(15%), CurbsideBasic(30%), CurbsideSort(45%), SingleStream(40%), PayAsYouThrow(50%), ZeroWaste(60%).
- [ ] Each tier has: diversion_rate, participation_rate, cost_per_household_year, revenue_potential.
- [ ] Recycling center building: 100 tons/day, 3x3 footprint, $2M build, $4K/day operating.
- [ ] `RecyclingEconomics` resource with commodity prices per material type.
- [ ] Net value per ton: revenue from materials minus collection + processing costs.
- [ ] Contamination rate (15-30%): waste in recycling stream goes to landfill.
- [ ] Market price cycles: ~5 game-year period, 0.3x bust to 1.5x boom.
- [ ] Player selects recycling program tier as a policy.

## Test Plan
- [ ] Unit test: CurbsideBasic diverts 30% of recyclable waste.
- [ ] Unit test: contamination reduces effective recycling by 15-30%.
- [ ] Unit test: net value can be negative during bust cycle.
- [ ] Integration test: recycling program reduces landfill input.
- [ ] Integration test: switching to higher tier increases diversion.

## Pitfalls
- Recycling economics can go negative (costs city money); must communicate this clearly.
- Market cycle timing needs a game-year counter.
- Pay-as-you-throw is unusual (variable fee per trash bag); may simplify.

## Code References
- Research: `environment_climate.md` sections 6.4.1-6.4.3

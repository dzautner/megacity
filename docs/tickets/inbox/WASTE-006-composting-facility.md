# WASTE-006: Composting Facility and Organic Waste Diversion

## Priority: T2 (Depth)

## Description
Implement composting facilities that divert organic waste (food + yard waste = ~34% of MSW) from landfill. Multiple composting methods with different capacities, timeframes, and co-benefits. Compost can be sold or used in city parks and agriculture.

## Current State
- No composting system.
- No organic waste tracking.

## Definition of Done
- [ ] Composting methods: Windrow (50 tons/day, 6x6, $30/ton), Aerated Static Pile (100 tons/day, 4x4, $45/ton), In-vessel (200 tons/day, 3x3, $60/ton), Anaerobic Digestion (100 tons/day, 3x3, $50/ton).
- [ ] Compostable waste = food_waste + yard_waste = ~34% of total MSW.
- [ ] Participation rate affects actual diversion (70% default).
- [ ] Anaerobic digestion bonus: produces biogas = 0.1-0.2 MWh/ton electricity.
- [ ] Compost products: sold at $20-40/ton, used in parks (-25% irrigation need), agriculture (+15% crop yield).
- [ ] Composting policy toggle to activate collection program.

## Test Plan
- [ ] Unit test: in-vessel processes 200 tons/day max.
- [ ] Unit test: anaerobic digestion generates electricity.
- [ ] Integration test: composting diverts 34% * participation from landfill.
- [ ] Integration test: compost applied to parks improves park quality.

## Pitfalls
- 4 composting methods may be too many for player choice; could consolidate to 2.
- Anaerobic digestion biogas electricity needs to feed into POWER system.
- Composting facility has odor radius that affects nearby buildings.

## Code References
- Research: `environment_climate.md` sections 6.6.1-6.6.2

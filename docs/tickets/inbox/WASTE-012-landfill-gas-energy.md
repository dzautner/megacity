# WASTE-012: Landfill Gas (LFG) Collection and Energy Generation

## Priority: T3 (Differentiation)

## Description
Implement landfill gas collection as a power source. Decomposing organic waste in landfills generates methane. With gas collection infrastructure, this methane can generate ~1 MW per 1,000 tons/day landfill. Without collection, methane is a greenhouse gas and fire/explosion risk.

## Current State
- No landfill gas concept.
- No gas collection building.

## Definition of Done
- [ ] Landfill gas generation: `gas_rate = daily_input * 100 cubic_ft / ton / year`.
- [ ] Gas composition: 50% methane, 50% CO2.
- [ ] With gas collection: generates electricity at 1 MW per 1,000 tons/day capacity.
- [ ] Without collection: contributes to greenhouse gas (WEATHER-016 CO2 tracking).
- [ ] Gas collection infrastructure: $500K upgrade per landfill, $20K/year maintenance.
- [ ] Fire/explosion risk: 0.1% per year without collection.
- [ ] Gas collection continues 15-30 years after landfill closure.
- [ ] Feeds into power grid as small renewable source.

## Test Plan
- [ ] Unit test: 500 tons/day landfill with collection = 0.5 MW.
- [ ] Unit test: without collection, methane counted as CO2 emissions.
- [ ] Integration test: landfill gas appears in energy dashboard.

## Pitfalls
- Small power output (0.5-2 MW) may not feel significant.
- Post-closure gas collection needs tracking after landfill is decommissioned.
- Fire/explosion risk without collection needs disaster system integration.

## Code References
- Research: `environment_climate.md` section 6.3.2 (LFG)

# WATER-011: Water Conservation Policies

## Priority: T3 (Differentiation)

## Description
Implement water conservation policies that reduce demand without building new supply: low-flow fixtures, xeriscaping mandates, water pricing tiers, greywater recycling, and rainwater harvesting.

## Current State
- No water conservation policies.
- No demand-side management for water.

## Definition of Done
- [ ] Low-flow fixture mandate: -20% residential demand, $500/building retrofit cost.
- [ ] Xeriscaping (drought-tolerant landscaping): -40% irrigation demand.
- [ ] Tiered water pricing: higher rates for high usage, reduces demand 15%.
- [ ] Greywater recycling: -30% sewage generation, $3K/building.
- [ ] Rainwater harvesting: -10% demand during rainy seasons, $1K/building.
- [ ] Each policy toggleable with cost/benefit display.

## Test Plan
- [ ] Unit test: low-flow fixtures reduce residential demand by 20%.
- [ ] Unit test: multiple policies stack correctly.
- [ ] Integration test: conservation policies prevent water deficit.

## Pitfalls
- Multiple stacking policies could reduce demand to unrealistically low levels; cap at -60% total.
- Retrofit costs per building are expensive for large cities.
- Greywater system needs to reduce both water demand and sewage generation.

## Code References
- Research: `environment_climate.md` section 2.1 (conservation)

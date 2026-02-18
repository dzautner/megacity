# CIT-053: Economic Boom/Bust Cycles

**Priority:** T2 (Depth)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.8

## Description

7-10 year economic cycles with boom and bust phases. Boom: +20% commercial demand, +15% immigration, +10% land value, low unemployment. Bust: -20% commercial demand, -10% emigration, -5% land value, high unemployment, business closures. Cycle driven by sine wave + random perturbation. Economic indicators: GDP growth rate, unemployment rate, inflation, business confidence. Player can dampen cycles with counter-cyclical policy (stimulus during bust, reserves during boom).

## Definition of Done

- [ ] `EconomicCycle` resource with phase, amplitude, period
- [ ] Sine wave base with random perturbation
- [ ] Boom effects: demand, immigration, land value modifiers
- [ ] Bust effects: demand reduction, unemployment, closures
- [ ] GDP growth rate, unemployment rate metrics
- [ ] Business confidence index
- [ ] Counter-cyclical policy tools
- [ ] Economic cycle graph in UI

## Test Plan

- Unit test: cycle period in 7-10 year range
- Unit test: boom phase increases demand
- Unit test: bust phase increases unemployment
- Integration test: city experiences visible boom/bust over game time

## Pitfalls

- Bust phase must not be city-ending; player should be able to survive with preparation
- Random perturbation should not override player's economic management

## Relevant Code

- `crates/simulation/src/economy.rs` (CityBudget)
- `crates/simulation/src/market.rs` (MarketPrices)

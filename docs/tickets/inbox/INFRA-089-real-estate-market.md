# INFRA-089: Real Estate Market Simulation
**Priority:** T3
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-060, INFRA-084
**Source:** master_architecture.md, M4

## Description
Simulate real estate market with property values, speculation, and market cycles. Property values based on land value + building quality + demand pressure. Speculation: rapid value increase attracts investors, creating price bubbles. Crash: overvalued properties correct when demand drops. Market affects construction (developers build when profitable) and citizen wealth.

## Definition of Done
- [ ] Per-building property value computation
- [ ] Market demand pressure affecting values
- [ ] Speculation/bubble detection
- [ ] Market correction/crash events
- [ ] Developer construction driven by profit potential
- [ ] Real estate market index in stats
- [ ] Tests pass

## Test Plan
- Unit: High-demand area has higher property values than low-demand
- Unit: Rapid appreciation triggers speculation flag
- Integration: Real estate bubble forms and corrects over game years

## Pitfalls
- Market simulation interacts with everything (economy, happiness, construction)
- Crashes can bankrupt player if property tax revenue drops suddenly
- Current `market.rs` has some functionality; extend

## Relevant Code
- `crates/simulation/src/market.rs` -- market simulation
- `crates/simulation/src/land_value.rs` -- base for property value
- `crates/simulation/src/economy.rs` -- tax revenue from property

# INFRA-115: Non-Revenue Water and Pipe Leak Simulation
**Priority:** T3
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-019
**Source:** infrastructure_engineering.md, Section 3 (NRW)

## Description
Model pipe aging and water leaks. Non-Revenue Water (NRW) = water produced minus water billed. Aging pipes increase leak rate (15-30% typical, up to 60% for very old pipes). Pipe age tracked per segment. Leak detection program reduces NRW. Higher NRW = higher operating costs. District Metering Areas (DMAs) for monitoring. Pipe age overlay showing infrastructure age.

## Definition of Done
- [ ] Per-pipe-segment age tracking
- [ ] Leak rate increases with pipe age
- [ ] NRW percentage computed and displayed
- [ ] Leak detection program building reduces NRW
- [ ] Pipe replacement option for old segments
- [ ] Pipe age overlay
- [ ] Tests pass

## Test Plan
- Unit: New pipe has 5% NRW; 50-year-old pipe has 30% NRW
- Unit: Leak detection reduces NRW by 10%
- Integration: Water bills increase as pipes age (higher operating costs)

## Pitfalls
- Pipe age must be serialized
- Pipe burst events from very old pipes (emergency repair)
- NRW affects water supply/demand balance

## Relevant Code
- `crates/simulation/src/utilities.rs` -- water pipe network
- `crates/simulation/src/economy.rs` -- water operating costs

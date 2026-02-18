# INFRA-041: Transit Transfer Penalties and Free Transfers
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-037, INFRA-028
**Source:** transportation_simulation.md, Section 4.6

## Description
Model transfer penalties in transit ridership calculation. Each transfer reduces probability of choosing transit by 10-20%. Penalty values: cross-platform (5-8 min equivalent), same-stop (8-12 min), walk transfer (12-18 min), long walk (15-25 min). Implement free transfers within 60-90 minutes of first boarding. Hub stations with short transfer distances get lower penalty. Timed transfers at key junctions (all routes arrive simultaneously, 2-5 min transfer window).

## Definition of Done
- [ ] Transfer penalty values per transfer type in mode choice model
- [ ] Free transfer window (configurable, default 90 min)
- [ ] Hub station designation with reduced transfer penalty
- [ ] Timed transfer option at designated junctions
- [ ] Multi-leg transit paths include transfer penalties in total cost
- [ ] Tests pass

## Test Plan
- Unit: Trip with 1 transfer has 10-20% lower transit mode share than direct trip
- Unit: Cross-platform transfer adds 5-8 min equivalent to trip time
- Integration: Hub stations attract more transfers than non-hub stops

## Pitfalls
- Timed transfers require very reliable operations; delays cascade
- Free transfer tracking needs per-citizen fare state
- Too many transfers makes transit uncompetitive; warn player

## Relevant Code
- `crates/simulation/src/movement.rs` -- multi-modal pathfinding

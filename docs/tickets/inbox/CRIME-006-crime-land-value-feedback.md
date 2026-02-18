# CRIME-006: Crime-Land Value Feedback Loop

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** CRIME-001 (crime types)
**Source:** social_agent_simulation.md Section 8.4

## Description

Create bidirectional feedback between crime and land value. High crime reduces land value (already partially implemented: crime inversely proportional to land value). Low land value increases crime motivation. This feedback loop should be dampened to prevent runaway spirals. Crime reduction per unit of land value increase should be smaller than land value reduction per unit of crime increase. Introduce "recovery" mechanic where sustained low crime allows land value to recover over months.

## Definition of Done

- [ ] Crime -> land value penalty: -1 land value per 5 crime units
- [ ] Land value -> crime motivation: +1 crime base per 10 land value below 50
- [ ] Recovery timer: 90 game-days of low crime = land value starts recovering
- [ ] Damping factor prevents runaway spiral (crime can't exceed 200, land value can't go below 5)
- [ ] Feedback loop convergence tested (doesn't oscillate indefinitely)

## Test Plan

- Unit test: high crime area has lower land value
- Unit test: low crime area recovers land value after 90 days
- Integration test: vicious cycle is bounded (doesn't crash to zero everywhere)
- Integration test: police investment breaks the negative cycle

## Pitfalls

- Current crime grid resets each tick; feedback requires persistent state
- Damping must be strong enough to allow player intervention to work

## Relevant Code

- `crates/simulation/src/crime.rs` (update_crime, lines 44-58)
- `crates/simulation/src/land_value.rs` (update_land_value)

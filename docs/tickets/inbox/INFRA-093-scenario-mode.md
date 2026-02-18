# INFRA-093: Scenario/Challenge Modes (5+ Scenarios)
**Priority:** T4
**Complexity:** L (3-5 days)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Create at least 5 scenario/challenge modes with specific objectives and constraints. Example scenarios: 1) Traffic emergency (fix gridlocked city), 2) Debt crisis (balance budget in 5 years), 3) Natural disaster recovery (rebuild after earthquake), 4) Green city (achieve zero emissions), 5) Population boom (handle rapid growth). Each scenario has win/loss conditions, preset map, and time limit.

## Definition of Done
- [ ] Scenario framework: preset map, objectives, constraints, time limit
- [ ] At least 5 scenarios implemented
- [ ] Win/loss condition evaluation
- [ ] Scenario selection screen
- [ ] Scenario-specific advisor dialogue
- [ ] Tests pass

## Test Plan
- Unit: Win condition triggers when objectives met
- Unit: Loss condition triggers when time expires or failure state reached
- Integration: Each scenario is completable and fun

## Pitfalls
- Scenario balance: must be challenging but not impossible
- Preset maps need to be carefully designed for each scenario
- Scenario state must be separate from sandbox saves

## Relevant Code
- `crates/simulation/src/events.rs` -- scenario event system
- `crates/ui/src/lib.rs` -- scenario selection UI

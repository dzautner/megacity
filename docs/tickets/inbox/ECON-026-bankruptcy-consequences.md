# ECON-026: Budget Deficit and Bankruptcy Consequences
**Priority:** T1
**Complexity:** M
**Dependencies:** ECON-001
**Source:** master_architecture.md, section M2; cities_skylines_analysis.md, section 3.5

## Description
Implement consequences when treasury drops below zero. Currently budget can go negative without effect. Deficit should trigger service reductions, credit rating downgrade, and eventually game over.

- Warning at treasury < $2000 ("Budget deficit imminent")
- At treasury < 0: emergency loan auto-offered
- If declined or max debt reached: service effectiveness reduced by 50%
- Extended deficit (12+ months): service buildings close, buildings abandon
- Game over: if treasury < -$50,000 and no recovery for 24 months
- Player always has option to take loan or raise taxes to recover

## Definition of Done
- [ ] Budget warning notifications
- [ ] Emergency loan offer at deficit
- [ ] Service reduction during extended deficit
- [ ] Eventual game over condition
- [ ] Recovery always possible via player action

## Test Plan
- Integration: Set tax to 0, verify deficit occurs and warnings appear
- Integration: Take emergency loan, verify recovery possible

## Pitfalls
- Game over should be avoidable (not surprising or unfair)
- Service reduction creates spiral -- need damping
- Must not make loans so easy that deficit has no consequence

## Relevant Code
- `crates/simulation/src/economy.rs:CityBudget` -- deficit tracking
- `crates/simulation/src/budget.rs:ExtendedBudget` -- emergency loan
- `crates/simulation/src/events.rs` -- budget warning events

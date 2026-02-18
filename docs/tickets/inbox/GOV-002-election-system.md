# GOV-002: Election System

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** GOV-001 (factions)
**Source:** social_agent_simulation.md Section 10

## Description

Regular elections every 4 game-years. Mayor approval rating = weighted sum of faction satisfactions. Approval > 50% = re-election. Approval < 50% = election challenge. Campaign promises mechanic: player makes 2-3 binding promises (build X, reduce crime, lower taxes) that become mandatory objectives. Breaking promises = -20% approval. Election loss = game over (or handicap mode). Early election triggered when civic trust < 20% and grievance > 80%.

## Definition of Done

- [ ] Election cycle every 4 game-years
- [ ] Approval rating from faction satisfactions
- [ ] Campaign promise selection UI (3 promises from pool)
- [ ] Promise tracking (objective completion checker)
- [ ] Broken promise penalty (-20% approval)
- [ ] Election result based on approval + faction support
- [ ] Election loss consequence (game over or continued with handicap)
- [ ] Early election trigger conditions

## Test Plan

- Unit test: approval above 50% wins election
- Unit test: broken promise reduces approval by 20%
- Unit test: early election triggered at trust < 20% and grievance > 80%
- Integration test: full election cycle plays through

## Pitfalls

- Game over from election loss may frustrate players; offer graceful continuation
- Campaign promises must be feasible (don't offer impossible goals)

## Relevant Code

- `crates/simulation/src/events.rs` (MilestoneTracker)
- `crates/simulation/src/policies.rs`

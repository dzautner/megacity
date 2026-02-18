# CIT-095: Scenario/Challenge Mode

**Priority:** T4 (Polish)
**Complexity:** High (4-5 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.20

## Description

Structured challenge scenarios with specific objectives and constraints. Example scenarios: "Grow to 100K in 50 years", "Survive 3 disasters", "Achieve zero pollution city", "Housing crisis: reduce homelessness from 10% to 1%", "Traffic challenge: reduce average commute below 20 min". Each scenario has: starting conditions, objectives, time limit, scoring criteria. Leaderboard for scores. Scenario editor for community content.

## Definition of Done

- [ ] Scenario definition format (objectives, constraints, starting state)
- [ ] 5-10 built-in scenarios
- [ ] Objective tracking and completion checking
- [ ] Time limit countdown
- [ ] Scoring system
- [ ] Victory/defeat conditions
- [ ] Scenario selection menu
- [ ] Score display and comparison

## Test Plan

- Unit test: objective tracking detects completion
- Unit test: time limit enforced
- Unit test: scoring formula produces expected results
- Integration test: full scenario playthrough

## Pitfalls

- Scenarios must use existing systems, not require new mechanics
- Difficulty must be calibrated through playtesting

## Relevant Code

- `crates/simulation/src/achievements.rs`
- `crates/simulation/src/events.rs`

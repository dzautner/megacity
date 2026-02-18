# GOV-007: Referendum Mechanic for Controversial Decisions

**Priority:** T3 (Differentiation)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** GOV-001 (factions)
**Source:** social_agent_simulation.md Section 10, master_architecture.md Section 1.15

## Description

Certain actions trigger a referendum: highway through residential area, factory near school, airport expansion, casino zone. Referendum popup shows: proposal, support by faction, projected vote outcome. Player can accept referendum result, override with political cost (-15 trust, faction anger), or modify proposal. Referendum outcome determined by faction votes weighted by support.

## Definition of Done

- [ ] `ReferendumEvent` struct with proposal, affected factions, vote projection
- [ ] Trigger conditions for controversial actions
- [ ] Referendum UI with support breakdown by faction
- [ ] Player choices: accept, override (trust cost), modify
- [ ] Override penalty: -15 civic trust, affected faction -20 satisfaction
- [ ] Passed referendum boosts trust +5
- [ ] Event journal records referendum and outcome

## Test Plan

- Unit test: highway near homes triggers referendum
- Unit test: override applies trust penalty
- Unit test: vote outcome matches faction projections
- Integration test: referendum popup appears during gameplay

## Pitfalls

- Referendums should be rare (2-3 per game-decade) to avoid being tedious
- Must not block essential infrastructure (fire station placement shouldn't require referendum)

## Relevant Code

- `crates/simulation/src/events.rs`
- `crates/simulation/src/policies.rs`

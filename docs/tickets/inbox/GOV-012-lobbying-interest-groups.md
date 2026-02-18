# GOV-012: Lobbying and Interest Group Pressure

**Priority:** T3 (Differentiation)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** GOV-001 (factions)
**Source:** social_agent_simulation.md Section 10

## Description

Interest groups (tied to factions) lobby for specific policies. Lobbying power proportional to faction wealth * support. Powerful lobbies offer deals: "approve this zoning change and we'll fund a new park." Accepting deals gives immediate benefit but creates obligation. Rejecting deals angers the faction. Lobbying events create interesting political dilemmas.

## Definition of Done

- [ ] Interest group lobbying events (periodic, based on faction clout)
- [ ] Deal proposals with benefit and obligation
- [ ] Accept/reject choice for player
- [ ] Accept: immediate benefit + faction satisfaction + obligation
- [ ] Reject: faction anger, no obligation
- [ ] Lobby power proportional to wealth * support
- [ ] Lobbying visible in governance panel

## Test Plan

- Unit test: wealthy faction has stronger lobbying
- Unit test: accepted deal provides benefit and creates obligation
- Unit test: rejected deal reduces faction satisfaction

## Pitfalls

- Lobbying events should be infrequent (1-2 per game year)
- Deals must be genuinely tempting (not obviously bad)

## Relevant Code

- `crates/simulation/src/events.rs`
- GOV-001 infrastructure

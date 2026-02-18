# CRIME-010: Substance Abuse and Drug-Related Crime

**Priority:** T3 (Differentiation)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** CRIME-001 (crime types), HEALTH-010 (mental health)
**Source:** social_agent_simulation.md Section 8

## Description

Substance abuse as health condition that drives crime. Abuse rate = f(unemployment, mental_health, income, availability). Drug crime includes dealing (organized crime driven) and possession (individual). Substance abuse treatment centers reduce abuse rate and associated crime. Legalization policy: reduces drug crime but may increase usage. Needle exchange/harm reduction: reduces health costs but controversial (faction reactions).

## Definition of Done

- [ ] Substance abuse rate per chunk
- [ ] Abuse driven by unemployment, mental health, poverty
- [ ] Drug crime from dealing and possession
- [ ] Treatment center service building reduces abuse
- [ ] Legalization policy option with tradeoffs
- [ ] Harm reduction policy option with faction reactions
- [ ] Substance abuse health penalty
- [ ] Associated crime stats

## Test Plan

- Unit test: high unemployment increases abuse rate
- Unit test: treatment center reduces local abuse
- Unit test: legalization reduces drug crime but increases usage

## Pitfalls

- Sensitive topic; keep mechanics game-focused, not preachy

## Relevant Code

- `crates/simulation/src/crime.rs`
- `crates/simulation/src/policies.rs`

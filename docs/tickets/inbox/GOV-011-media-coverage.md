# GOV-011: Media Coverage and Public Opinion

**Priority:** T3 (Differentiation)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** GOV-001 (factions)
**Source:** social_agent_simulation.md Section 10

## Description

Media system amplifies events and affects public opinion. TVStation building generates media coverage. Media coverage doubles the happiness/trust impact of events (positive and negative). Scandal events get media amplification (-20 trust instead of -10). Good news events amplified (+10 trust instead of +5). Free press policy: media reports all events honestly. State media policy: filters negative events but reduces civic trust long-term.

## Definition of Done

- [ ] Media coverage metric from TVStation presence
- [ ] Event impact amplification with media coverage
- [ ] Scandal amplification
- [ ] Good news amplification
- [ ] Free press vs state media policy
- [ ] State media reduces negative impact short-term, trust long-term
- [ ] Media coverage affects immigration (city visibility)

## Test Plan

- Unit test: media doubles event impact
- Unit test: state media filters negative events
- Unit test: free press maintains trust

## Pitfalls

- Media system should add flavor, not be a dominant mechanic

## Relevant Code

- `crates/simulation/src/events.rs`
- `crates/simulation/src/services.rs` (TVStation)

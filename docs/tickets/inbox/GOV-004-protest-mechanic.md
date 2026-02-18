# GOV-004: Protest Mechanic

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** GOV-001 (factions), GOV-003 (civic trust)
**Source:** social_agent_simulation.md Section 10

## Description

Citizens protest when faction satisfaction drops below 30% for a faction with >15% support. Protests are visible events: citizens gather at city hall or problematic location. Protest effects: -10 happiness city-wide, -20% productivity in protest area, media attention (doubles grievance gain). Protests resolve when the underlying complaint is addressed. Multiple simultaneous protests = crisis. Protest escalation: peaceful -> disruptive -> violent (if ignored for 30+ days).

## Definition of Done

- [ ] Protest trigger: faction satisfaction < 30% AND faction support > 15%
- [ ] Protest event with location (city hall or issue location)
- [ ] Visual: citizen sprites gather at protest location
- [ ] Effects: -10 happiness, -20% productivity in area
- [ ] Grievance increase doubled during protest
- [ ] Protest resolution when complaint addressed
- [ ] Escalation timeline: peaceful (0-10 days) -> disruptive (10-20) -> violent (20-30+)
- [ ] Event journal entry for protests

## Test Plan

- Unit test: protest triggered at correct faction satisfaction threshold
- Unit test: protest applies happiness and productivity penalties
- Unit test: addressing complaint resolves protest
- Integration test: ignored protest escalates over time

## Pitfalls

- Protests shouldn't be constant (too annoying); cooldown between same faction's protests
- Violent protests need careful handling (property damage, arrests)

## Relevant Code

- `crates/simulation/src/events.rs` (EventJournal)
- GOV-001 infrastructure

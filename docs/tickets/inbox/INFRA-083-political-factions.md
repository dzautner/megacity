# INFRA-083: Political Faction System
**Priority:** T3
**Complexity:** XL (1-2 weeks)
**Dependencies:** none
**Source:** master_architecture.md, M4

## Description
Implement political factions with citizen opinions and elections. At least 3 factions: Progressives (transit, parks, density), Conservatives (low taxes, roads, suburbs), Greens (environment, renewable energy). Citizens align with factions based on their demographics, location, and experiences. Elections every N game-years where citizens vote. Winning faction influences available policies and advisor recommendations. NIMBY/YIMBY mechanics for controversial developments.

## Definition of Done
- [ ] At least 3 political factions with distinct priorities
- [ ] Citizen faction alignment from demographics and satisfaction
- [ ] Election system with voting
- [ ] Winning faction affects policy options
- [ ] NIMBY/YIMBY reactions to developments near residential areas
- [ ] Election results affect approval rating
- [ ] Tests pass

## Test Plan
- Unit: Low-income citizens lean toward faction offering welfare
- Unit: Suburbanites lean toward faction favoring low taxes
- Integration: Election outcomes vary based on city composition

## Pitfalls
- Political simulation is sensitive; avoid real-world party names
- Faction balance: no faction should be objectively "correct"
- Citizen alignment should shift gradually based on gameplay, not be fixed

## Relevant Code
- `crates/simulation/src/citizen.rs` -- citizen demographics
- `crates/simulation/src/events.rs` -- election events
- `crates/simulation/src/advisors.rs` -- faction-aligned advisors

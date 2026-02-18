# POL-002: Political Faction System
**Priority:** T3
**Complexity:** XL
**Dependencies:** ECON-012
**Source:** master_architecture.md, section M4; cities_skylines_analysis.md, section 16.7

## Description
Implement political factions that represent citizen interest groups. Factions have opinions on policies, development decisions, and city management. Elections force player to balance competing interests.

- Factions (5 minimum): Developers (pro-growth), Environmentalists (anti-pollution), Traditionalists (anti-change), Progressives (pro-transit/density), Business (pro-low-tax)
- Citizens belong to faction based on personality, income, education, neighborhood
- Faction approval rating affected by policy decisions
- Elections every 4 game-years: player's approval must be >50% aggregate
- Low approval consequences: reduced cooperation, protests, emigration
- Faction-aligned advisors give biased recommendations

## Definition of Done
- [ ] 5 political factions with distinct priorities
- [ ] Citizens assigned to factions
- [ ] Faction approval ratings computed from policies/decisions
- [ ] Elections with consequences
- [ ] Faction-aligned advisor recommendations

## Test Plan
- Integration: Enact pro-environment policy, verify Environmentalist approval rises
- Integration: Hold election with low approval, verify consequences

## Pitfalls
- advisors.rs already exists -- integrate faction alignment
- Politics must enhance gameplay, not gatekeep (player always retains control)
- Faction distribution should emerge organically from demographics

## Relevant Code
- `crates/simulation/src/advisors.rs` -- faction-aligned advice
- `crates/simulation/src/citizen.rs:Personality` -- faction assignment
- `crates/simulation/src/events.rs` -- election events

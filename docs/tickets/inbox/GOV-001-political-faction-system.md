# GOV-001: Political Faction System

**Priority:** T3 (Differentiation)
**Complexity:** High (5-7 person-weeks)
**Dependencies:** CIT-001 (demographics)
**Source:** social_agent_simulation.md Section 10, master_architecture.md Section 1.15

## Description

Implement Tropico-style faction system. 6 factions: Environmentalists (want parks, clean energy, nature), Business (want low taxes, industry, commerce), Labor (want worker rights, social programs, fair wages), NIMBY (want low density, historic preservation, no change), Progressive (want transit, density, innovation), Conservative (want traditional values, safety, fiscal prudence). Each citizen has affinity to factions based on demographics. Faction support = count of citizens with dominant faction alignment / total. Faction clout influences policy effectiveness.

## Definition of Done

- [ ] `Faction` enum with 6 variants
- [ ] `FactionAffinities` component with f32 per faction per citizen
- [ ] Affinity determined by: income, education, age, occupation, personality
- [ ] `FactionSupport` resource tracking city-wide faction strength
- [ ] Dominant faction per citizen
- [ ] Faction satisfaction based on current policies and city conditions
- [ ] Faction clout = sum(member_wealth * member_count) / total
- [ ] Faction UI panel showing support levels and demands

## Test Plan

- Unit test: wealthy educated citizen has Business/Progressive affinity
- Unit test: working-class citizen has Labor affinity
- Unit test: elderly homeowner has NIMBY/Conservative affinity
- Integration test: faction support shifts with city demographics changes

## Pitfalls

- 6 factions * citizen count = significant computation; use aggregate per chunk
- Factions must create genuine impossible tradeoffs (Tropico's design insight)

## Relevant Code

- `crates/simulation/src/policies.rs` (Policies)
- `crates/simulation/src/citizen.rs` (CitizenDetails, Personality)

# CIT-032: Happiness Factor -- Social Belonging

**Priority:** T3 (Differentiation)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** CIT-019 (segregation), CIT-001 (demographics)
**Source:** social_agent_simulation.md Section 5.2

## Description

Social belonging happiness component: measures citizen's integration into community. Factors: neighborhood_compatibility (Schelling satisfaction), family_nearby (partner + children in city), friend_network_size, community_facilities (parks, libraries, religious buildings within radius). Formula: belonging = compatibility*0.3 + family*0.25 + friends*0.25 + community*0.2. Weight in overall happiness: 0.10.

## Definition of Done

- [ ] Neighborhood compatibility from Schelling model
- [ ] Family nearby check (partner and children entities exist and in city)
- [ ] Friend network size (simplified: same-chunk same-income citizens count)
- [ ] Community facilities count within walkable radius
- [ ] Combined belonging formula
- [ ] Weight of 0.10 in overall happiness

## Test Plan

- Unit test: citizen with family, compatible neighborhood, friends = high belonging
- Unit test: isolated citizen in incompatible neighborhood = low belonging
- Integration test: citizens near parks and libraries report higher belonging

## Pitfalls

- Friend network is expensive to compute; use statistical approximation
- Must not create feedback loop: low belonging -> emigration -> fewer friends -> lower belonging

## Relevant Code

- `crates/simulation/src/happiness.rs` (update_happiness)
- `crates/simulation/src/citizen.rs` (Family component)

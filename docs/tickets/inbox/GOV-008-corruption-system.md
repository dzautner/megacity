# GOV-008: Corruption System

**Priority:** T3 (Differentiation)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** GOV-001 (factions)
**Source:** social_agent_simulation.md Section 10

## Description

Corruption level (0-100) increases with: low transparency (no city hall), rapid growth (10% per doubling), low oversight, certain policies. Corruption effects: -15% tax revenue (leakage), +20% construction costs (kickbacks), -10% service quality, random scandal events (-20 trust). Anti-corruption measures: transparency policy, audit office, free press. Corruption visible as metric in governance panel.

## Definition of Done

- [ ] Corruption resource (0-100, starts 10)
- [ ] Growth factors: city size, oversight level, policy choices
- [ ] Effects: tax leakage, construction cost increase, service quality decrease
- [ ] Scandal events at high corruption (random triggers)
- [ ] Anti-corruption policies and buildings
- [ ] Corruption metric in governance panel
- [ ] Historical tracking

## Test Plan

- Unit test: rapid growth increases corruption
- Unit test: audit office reduces corruption
- Unit test: high corruption reduces tax revenue by 15%

## Pitfalls

- Corruption should be a slow-moving background threat, not an immediate crisis
- Must not make corruption unavoidable (player should be able to fight it)

## Relevant Code

- `crates/simulation/src/policies.rs`
- `crates/simulation/src/economy.rs`

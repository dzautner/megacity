# CIT-033: Happiness Factor -- Governance Satisfaction

**Priority:** T3 (Differentiation)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** GOV-001 (political factions)
**Source:** social_agent_simulation.md Section 5.2

## Description

Governance satisfaction happiness component: measures citizen's approval of city management. Factors: tax_fairness (1.0 - tax_rate * 2, clamped), service_responsiveness (are complaints addressed?), political_representation (faction in power alignment), corruption_level. Formula: governance = tax_fairness*0.3 + responsiveness*0.3 + representation*0.2 + (1-corruption)*0.2. Weight in overall happiness: 0.05.

## Definition of Done

- [ ] Tax fairness from effective tax rate
- [ ] Service responsiveness (proxy: average service coverage quality)
- [ ] Political representation (faction alignment placeholder until GOV-001)
- [ ] Corruption level (placeholder, default 0.0)
- [ ] Combined governance formula
- [ ] Weight of 0.05 in overall happiness
- [ ] Replace flat HIGH_TAX_PENALTY

## Test Plan

- Unit test: low taxes + good services = high governance satisfaction
- Unit test: high taxes + poor services = low governance satisfaction

## Pitfalls

- Political representation requires faction system that doesn't exist yet; use placeholder

## Relevant Code

- `crates/simulation/src/happiness.rs` (HIGH_TAX_PENALTY, line 166)
- `crates/simulation/src/policies.rs` (Policies)

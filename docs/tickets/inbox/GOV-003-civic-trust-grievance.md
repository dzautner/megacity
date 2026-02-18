# GOV-003: Civic Trust and Grievance Dual Track

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** GOV-001 (factions)
**Source:** social_agent_simulation.md Section 14.5 (Frostpunk model)

## Description

Implement Frostpunk-inspired dual-track system. Civic Trust (0-100): long-term confidence. Built by: infrastructure investment (+2/month), consistent governance (+1/month), economic growth (+3/month). Destroyed by: scandals (-10), broken promises (-15), decline (-5/month). Grievance (0-100): short-term frustration. Caused by: traffic (+2/month if avg commute > 30min), crime (+3/month if crime high), pollution (+2/month), service failures (+5 per incident). Reduced by: responsive action (-3/month), good services (-2/month). Trust < 20 AND Grievance > 80 = early election / crisis event.

## Definition of Done

- [ ] `CivicTrust` resource (0-100, starts 60)
- [ ] `Grievance` resource (0-100, starts 20)
- [ ] Trust builders and destroyers with monthly rates
- [ ] Grievance causes and reducers with monthly rates
- [ ] Crisis event when trust < 20 AND grievance > 80
- [ ] Both metrics displayed in UI prominently
- [ ] Historical graphs for trust and grievance

## Test Plan

- Unit test: infrastructure investment raises trust
- Unit test: crime increases grievance
- Unit test: crisis triggered at correct thresholds
- Integration test: well-managed city maintains high trust, low grievance

## Pitfalls

- Grievance should be "always rising unless managed" (Frostpunk insight)
- Some actions raise both trust and grievance (authoritarian laws)

## Relevant Code

- `crates/simulation/src/events.rs` (ActiveCityEffects)
- `crates/simulation/src/happiness.rs`

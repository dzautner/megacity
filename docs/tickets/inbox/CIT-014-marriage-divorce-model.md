# CIT-014: Marriage and Divorce Model

**Priority:** T3 (Differentiation)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** CIT-001 (demographics), CIT-004 (household)
**Source:** social_agent_simulation.md Section 2.3

## Description

Enhance existing marriage/divorce life events with realistic probability models. Marriage probability peaks at 25-34 (5-8% annually), modified by education, income, and personality sociability. Divorce probability follows U-shaped curve peaking at years 5-8 of marriage (~2-4%/year), higher in first marriage than subsequent. Marriage creates/merges households. Divorce splits households, creates housing demand, reduces happiness for both parties and children.

## Definition of Done

- [ ] Marriage probability by age band with modifiers
- [ ] Partner matching by proximity (same neighborhood), education similarity, age range
- [ ] Marriage merges households
- [ ] Divorce probability by marriage duration (U-shaped)
- [ ] Divorce splits household, distributes children
- [ ] Divorce happiness penalty (-15 for 1 game-year)
- [ ] Children of divorce get happiness penalty (-10 for 1 game-year)
- [ ] Event journal entries for marriages and divorces

## Test Plan

- Unit test: marriage probability peaks at 25-34 age band
- Unit test: divorce probability peaks at years 5-8
- Unit test: household correctly split on divorce
- Integration test: marriage rate produces ~7 per 1000 pop/year (real-world rate)

## Pitfalls

- Entity references between partners must be bidirectional and consistent
- Save/load must preserve marriage relationships via entity remapping

## Relevant Code

- `crates/simulation/src/life_simulation.rs` (life_events)
- `crates/simulation/src/citizen.rs` (Family component, line 282)

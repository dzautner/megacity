# HEALTH-007: Disease -- Food Poisoning

**Priority:** T3 (Differentiation)
**Complexity:** Low (included in HEALTH-002)
**Dependencies:** HEALTH-002 (SIR model)
**Source:** social_agent_simulation.md Section 9.1

## Description

Food poisoning: non-contagious, triggered by low-quality commercial zones (food establishments). Rate = base * (1 - food_safety_inspection_rate). Recovery 1-3 days, mortality 0.01% (higher for elderly). Health inspection policy reduces incidence. Commercial buildings in industrial zones have higher food contamination risk.

## Definition of Done

- [ ] Food poisoning disease variant (non-contagious)
- [ ] Triggered by commercial building quality and food safety
- [ ] Health inspection policy toggle (reduces incidence by 80%)
- [ ] Short recovery (1-3 days)
- [ ] Mild health penalty (-10)
- [ ] Higher risk near industrial zones

## Test Plan

- Unit test: food poisoning rate higher near industry
- Unit test: health inspection policy reduces incidence by 80%
- Integration test: commercial areas without inspection have periodic outbreaks

## Pitfalls

- Food poisoning is individual, not epidemic; don't use SIR model, use per-building probability

## Relevant Code

- HEALTH-002 infrastructure

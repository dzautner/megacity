# HEALTH-003: Disease -- Common Cold

**Priority:** T3 (Differentiation)
**Complexity:** Low (included in HEALTH-002)
**Dependencies:** HEALTH-002 (SIR model)
**Source:** social_agent_simulation.md Section 9.1

## Description

Common Cold: transmission_rate 0.3, recovery_rate 0.2 (5-day average), mortality_rate 0.0001, incubation 1-3 days. High transmission, negligible severity. Seasonal: 3x more common in winter. Causes minor productivity loss (-10% work output) and slight health reduction (-5 health). Spreads through workplace and school contact.

## Definition of Done

- [ ] CommonCold disease variant with correct parameters
- [ ] Seasonal modifier (3x in winter)
- [ ] Productivity penalty (-10%)
- [ ] Health penalty (-5)
- [ ] Spreads in workplaces and schools
- [ ] Self-resolving (no hospital needed)

## Test Plan

- Unit test: cold resolves in ~5 game-days average
- Unit test: winter has 3x more cold cases
- Integration test: cold season visible in health stats

## Pitfalls

- Very common disease shouldn't spam notifications; aggregate reporting only

## Relevant Code

- HEALTH-002 infrastructure

# HEALTH-004: Disease -- Flu / Influenza

**Priority:** T3 (Differentiation)
**Complexity:** Low (included in HEALTH-002)
**Dependencies:** HEALTH-002 (SIR model)
**Source:** social_agent_simulation.md Section 9.1

## Description

Flu: transmission_rate 0.15, recovery_rate 0.1 (10-day average), mortality_rate 0.002 (0.2%, higher for elderly), incubation 1-4 days. Moderate transmission and severity. Annual flu season in winter. Causes moderate productivity loss (-25%) and health reduction (-15). Hospitalization needed for severe cases (5% of infected). Elderly (65+) mortality 5x higher.

## Definition of Done

- [ ] Flu disease variant with correct parameters
- [ ] Seasonal modifier (winter peak)
- [ ] Productivity penalty (-25%)
- [ ] Health penalty (-15)
- [ ] 5% hospitalization rate
- [ ] Elderly mortality multiplier (5x)
- [ ] Hospital surge capacity pressure during epidemic

## Test Plan

- Unit test: flu lasts ~10 game-days average
- Unit test: elderly citizen has 5x mortality
- Unit test: 5% of infected require hospitalization
- Integration test: flu season increases hospital demand

## Pitfalls

- Flu epidemic overwhelming hospitals is a gameplay event, not a bug
- Must not kill too many citizens (balance severity)

## Relevant Code

- HEALTH-002 infrastructure

# HEALTH-006: Disease -- Measles (Vaccine-Preventable)

**Priority:** T3 (Differentiation)
**Complexity:** Low (included in HEALTH-002)
**Dependencies:** HEALTH-002 (SIR model)
**Source:** social_agent_simulation.md Section 9.1

## Description

Measles: highly contagious, transmission_rate 0.4, recovery_rate 0.1 (10-14 days), mortality_rate 0.003 (higher in children), incubation 10-14 days. Vaccine-preventable: vaccination program (health policy) moves citizens from S to R directly. Herd immunity threshold: 95% vaccination needed. Schools are primary spread vector. Outbreaks in under-vaccinated populations.

## Definition of Done

- [ ] Measles disease variant
- [ ] Vaccination policy (per-city toggle)
- [ ] Vaccinated citizens move to Recovered (immune)
- [ ] Herd immunity at 95% vaccination rate
- [ ] Schools amplify transmission
- [ ] Children mortality higher than adults
- [ ] Outbreak notification when cases exceed threshold

## Test Plan

- Unit test: 95% vaccination prevents outbreak
- Unit test: 80% vaccination allows outbreak
- Unit test: schools amplify spread
- Integration test: vaccination policy eliminates measles over time

## Pitfalls

- Vaccination as policy choice creates interesting gameplay (cost vs benefit)
- Anti-vax citizens could add flavor but may be controversial

## Relevant Code

- HEALTH-002 infrastructure
- `crates/simulation/src/policies.rs`

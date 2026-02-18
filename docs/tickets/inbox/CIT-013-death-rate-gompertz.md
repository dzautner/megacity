# CIT-013: Gompertz Mortality Curve

**Priority:** T1 (Core -- bugfix/improvement)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** social_agent_simulation.md Section 2.3, historical_demographics_services.md Section 2.2

## Description

Replace the current linear death probability (starts at age 70, linear ramp) with a Gompertz mortality curve: P(death|age) = 0.0001 * exp(0.085 * age). This produces realistic age-specific death rates: <0.1% at 30, ~0.5% at 60, ~3% at 75, ~15% at 90. Health modifiers scale the base rate: poor health (health < 20) doubles death probability. Infant mortality at 0.3% for age 0. This replaces the current death check in lifecycle.rs lines 43-66.

## Definition of Done

- [ ] `gompertz_death_probability(age, health)` function
- [ ] Replaces linear death check in `age_citizens`
- [ ] Infant mortality rate for age 0
- [ ] Health modifier multiplies base probability
- [ ] MAX_AGE death at 120 (raised from 100 for rare centenarians)

## Test Plan

- Unit test: probability at age 30 < 0.001
- Unit test: probability at age 75 ~ 0.03
- Unit test: probability at age 90 ~ 0.15
- Unit test: poor health doubles probability
- Integration test: average lifespan of citizens ~78-82

## Pitfalls

- exp(0.085 * age) grows fast; clamp to 1.0 to avoid > 100% probability
- Current MAX_AGE=100 may be too low; Gompertz naturally handles this

## Relevant Code

- `crates/simulation/src/lifecycle.rs` lines 43-66 (current death check)

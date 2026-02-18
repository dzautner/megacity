# HEALTH-010: Mental Health System

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** CIT-016 (needs enhancement)
**Source:** social_agent_simulation.md Section 9, historical_demographics_services.md Section 5.4

## Description

Mental health as a separate dimension from physical health. Stress accumulates from: long commutes (+5/month), unemployment (+10/month), crime exposure (+8/month), overcrowding (+5/month), isolation (+7/month). Stress reduced by: parks (-3/month within radius), community centers (-5), mental health clinics (-10), entertainment (-3). Chronic high stress triggers: reduced productivity (-20%), increased crime propensity (+50%), substance abuse risk, emigration. Mental health services needed: 10-20 counselors per 100K pop.

## Definition of Done

- [ ] `mental_health: f32` field on CitizenDetails (0-100)
- [ ] Stress accumulation from environmental factors
- [ ] Stress reduction from services and facilities
- [ ] Chronic stress (>80 for 90+ days) triggers negative outcomes
- [ ] Mental health clinic service type
- [ ] Mental health metric in city stats
- [ ] Substance abuse as mental health outcome

## Test Plan

- Unit test: long commute increases stress by 5/month
- Unit test: park proximity reduces stress by 3/month
- Unit test: chronic stress reduces productivity by 20%
- Integration test: city with no mental health services has rising stress levels

## Pitfalls

- Mental health is sensitive topic; keep mechanics game-like, not clinical
- Must not create doom spiral (stress -> unemployment -> more stress)

## Relevant Code

- `crates/simulation/src/citizen.rs` (CitizenDetails)
- `crates/simulation/src/happiness.rs`
- `crates/simulation/src/services.rs`

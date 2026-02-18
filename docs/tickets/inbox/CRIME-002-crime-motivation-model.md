# CRIME-002: Crime Motivation Model (Individual Level)

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** CRIME-001 (crime types), CIT-001 (demographics)
**Source:** social_agent_simulation.md Section 8.2

## Description

Individual-level crime probability for Full LOD citizens. Crime motivation = rational choice (expected gain vs risk): P(crime) = f(poverty, opportunity, social_control, individual_factors). Formula: base_propensity * poverty_multiplier * opportunity * (1 - social_control) * personality_factor. Poverty multiplier: 5x at poverty vs 0.2x at wealthy. Opportunity from target density (commercial buildings nearby). Social control from police + community engagement. Personality: low resilience and high materialism increase propensity.

## Definition of Done

- [ ] `crime_propensity()` function for Full LOD citizens
- [ ] Poverty multiplier from income class
- [ ] Opportunity from nearby commercial building density
- [ ] Social control from police coverage + neighborhood quality
- [ ] Personality factors (resilience, materialism) modify propensity
- [ ] Crime event generated when random < propensity
- [ ] Crime event records location, type, perpetrator entity

## Test Plan

- Unit test: poor citizen near commercial area with no police = high propensity
- Unit test: wealthy citizen in policed area = near-zero propensity
- Unit test: personality factors correctly modify propensity
- Integration test: crime events cluster in expected areas

## Pitfalls

- Individual crime computation only for Full LOD (~2000 citizens); Statistical tier uses aggregate
- Must not create deterministic crime (same citizen always commits crime)

## Relevant Code

- `crates/simulation/src/crime.rs`
- `crates/simulation/src/citizen.rs` (Personality)

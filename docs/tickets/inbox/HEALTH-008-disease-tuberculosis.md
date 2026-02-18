# HEALTH-008: Disease -- Tuberculosis (Density-Dependent)

**Priority:** T3 (Differentiation)
**Complexity:** Low (included in HEALTH-002)
**Dependencies:** HEALTH-002 (SIR model)
**Source:** social_agent_simulation.md Section 9.1

## Description

Tuberculosis: density-dependent transmission. Transmission_rate 0.05 * population_density_factor, recovery_rate 0.005 (6 months treatment), mortality_rate 0.05 without treatment, 0.01 with treatment. Incubation weeks to months. Spreads in overcrowded housing and homeless shelters. Treatment requires consistent healthcare access over months. Latent TB can reactivate years later.

## Definition of Done

- [ ] TB disease variant with density-dependent transmission
- [ ] Overcrowded housing amplifies transmission (2x)
- [ ] Homeless population has 5x transmission rate
- [ ] Long treatment duration (6 months game-time)
- [ ] Treatment requires sustained hospital access
- [ ] Without treatment: 5% mortality
- [ ] With treatment: 1% mortality

## Test Plan

- Unit test: TB transmission higher in dense areas
- Unit test: homeless population has elevated TB rates
- Unit test: treatment duration is ~6 months
- Integration test: overcrowded city with no healthcare sees TB epidemic

## Pitfalls

- Long treatment duration requires tracking treatment progress per citizen
- TB is a slow disease; visible effects take many game-months

## Relevant Code

- HEALTH-002 infrastructure
- `crates/simulation/src/homelessness.rs`

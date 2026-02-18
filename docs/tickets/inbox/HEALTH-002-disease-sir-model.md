# HEALTH-002: Disease and Epidemic SIR Model

**Priority:** T3 (Differentiation)
**Complexity:** High (3-4 person-weeks)
**Dependencies:** HEALTH-001 (healthcare tiers)
**Source:** social_agent_simulation.md Section 9, historical_demographics_services.md

## Description

Implement SIR (Susceptible-Infected-Recovered) model for disease spread. Diseases: CommonCold (high transmission, low severity), Flu (moderate/moderate), Measles (high/moderate, vaccine-preventable), Cholera (water contamination), FoodPoisoning (restaurant sanitation), Tuberculosis (density-dependent). Each disease has transmission_rate, recovery_rate, mortality_rate, and incubation_period. Spatial spread via citizen proximity and shared buildings. Hospitals reduce mortality.

## Definition of Done

- [ ] `Disease` enum with 6 types
- [ ] SIR model per chunk: dS/dt = -beta*S*I/N, dI/dt = beta*S*I/N - gamma*I, dR/dt = gamma*I
- [ ] Transmission rate modified by: population density, sanitation, healthcare
- [ ] Infected citizens have health penalty and reduced productivity
- [ ] Hospital treatment reduces mortality rate
- [ ] Epidemic threshold: when I > 5% of local population
- [ ] Epidemic event notification
- [ ] Vaccination mechanic (reduce susceptible pool)

## Test Plan

- Unit test: SIR model converges to expected equilibrium
- Unit test: high density increases transmission rate
- Unit test: hospital coverage reduces mortality
- Integration test: flu season creates visible illness wave
- Integration test: cholera outbreak near contaminated water

## Pitfalls

- SIR model per-citizen is O(n^2) for contact tracing; use spatial grid approximation
- Multiple simultaneous diseases need separate SIR states
- Epidemics shouldn't be city-ending; balance severity

## Relevant Code

- `crates/simulation/src/health.rs`
- `crates/simulation/src/citizen.rs` (CitizenDetails.health)
- `crates/simulation/src/life_simulation.rs` (update_health)

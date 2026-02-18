# INFRA-112: Disease/Epidemic Simulation
**Priority:** T5
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-057
**Source:** master_architecture.md, M6

## Description
Model disease spread and epidemics. SIR (Susceptible-Infected-Recovered) contagion model. Diseases spread through proximity (population density), transit (crowded buses/metro), and water (contaminated supply). Hospital capacity determines recovery rate. Quarantine policy reduces spread. Vaccination reduces susceptibility. Epidemic events trigger when infection rate exceeds threshold.

## Definition of Done
- [ ] SIR contagion model per citizen
- [ ] Density-based transmission
- [ ] Transit transmission multiplier
- [ ] Hospital capacity for treatment
- [ ] Quarantine and vaccination policies
- [ ] Disease overlay showing infection spread
- [ ] Tests pass

## Test Plan
- Unit: Dense area has higher transmission rate
- Unit: Hospital at capacity -> slower recovery
- Unit: Quarantine reduces R0
- Integration: Epidemic emerges and resolves over game time

## Pitfalls
- COVID-19 sensitivity; keep diseases fictional/abstracted
- SIR model can be expensive for 100K+ citizens; use statistical model
- Disease affecting worker productivity -> economic cascade

## Relevant Code
- `crates/simulation/src/health.rs` -- health system
- `crates/simulation/src/citizen.rs` -- citizen health state

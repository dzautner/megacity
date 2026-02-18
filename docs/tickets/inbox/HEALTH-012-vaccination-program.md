# HEALTH-012: Vaccination Program

**Priority:** T3 (Differentiation)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** HEALTH-002 (SIR model)
**Source:** social_agent_simulation.md Section 9

## Description

Vaccination as health policy. Vaccination coverage = vaccinated / total population. Herd immunity threshold varies by disease (measles 95%, flu 70%). Vaccination program cost = pop * cost_per_dose. Vaccinated citizens move from Susceptible to Recovered in SIR model. Anti-vax sentiment as random modifier (5-15% refusal rate). Vaccination campaign event for new disease outbreaks.

## Definition of Done

- [ ] Vaccination policy toggle per disease
- [ ] Vaccination coverage tracking
- [ ] Cost per vaccination dose
- [ ] Herd immunity threshold per disease
- [ ] Vaccinated citizens immune (S -> R)
- [ ] Refusal rate modifier (5-15%)
- [ ] Vaccination campaign event

## Test Plan

- Unit test: 95% vaccination prevents measles outbreak
- Unit test: vaccination cost deducted from budget
- Unit test: refusal rate limits max coverage

## Pitfalls

- Keep vaccination mechanics game-like, not political

## Relevant Code

- HEALTH-002 infrastructure
- `crates/simulation/src/policies.rs`

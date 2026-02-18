# HEALTH-011: Hospital Surge Capacity During Epidemics

**Priority:** T3 (Differentiation)
**Complexity:** Medium (1-2 person-weeks)
**Dependencies:** HEALTH-002 (SIR model), HEALTH-001 (healthcare tiers)
**Source:** social_agent_simulation.md Section 9

## Description

During epidemics, hospital demand surges beyond normal capacity. Surge capacity = normal_beds * 1.5 (using overflow areas). At surge, quality drops to 60%. Beyond surge, patients turned away (mortality increases). Temporary field hospitals as policy response (fast construction, moderate capacity, low quality). Pandemic declaration triggers: school closures, event cancellations, mask mandate option, quarantine zones.

## Definition of Done

- [ ] Surge capacity at 150% of normal beds
- [ ] Quality degradation during surge
- [ ] Beyond surge: patients untreated (increased mortality)
- [ ] Temporary field hospital building (fast deploy)
- [ ] Pandemic declaration event
- [ ] School closure policy during pandemic
- [ ] Quarantine zone designation
- [ ] Pandemic stats (infections, hospitalizations, deaths)

## Test Plan

- Unit test: surge at 150% capacity
- Unit test: quality drops during surge
- Unit test: field hospital provides emergency capacity
- Integration test: epidemic overwhelms unprepared city hospitals

## Pitfalls

- Pandemic events must be rare but impactful; tune carefully

## Relevant Code

- HEALTH-002 infrastructure
- `crates/simulation/src/services.rs`

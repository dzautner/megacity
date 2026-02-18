# CRIME-005: Prison System and Recidivism

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** CRIME-004 (court system)
**Source:** historical_demographics_services.md Section 5.3

## Description

Prison system with two philosophical approaches. Punitive model: $35K-80K/inmate/year, 68% recidivism (3-year re-arrest). Rehabilitation model: $90K-130K/inmate/year, 20% recidivism. Player chooses approach as policy. Prison capacity by type: MinSecurity (200-1000 beds, $50K-100K/bed build), Medium (500-2000, $100-200K), Maximum (500-1500, $150-300K). Overcrowded prisons (>100% capacity) generate unrest/escape events. Released prisoners need housing and employment; without them, recidivism guaranteed.

## Definition of Done

- [ ] Prison capacity tracking (beds vs inmates)
- [ ] `CorrectionModel` policy (Punitive vs Rehabilitation)
- [ ] Punitive: lower cost, higher recidivism (68%)
- [ ] Rehabilitation: higher cost, lower recidivism (20%)
- [ ] Inmates serve sentences (property: 6-24 months, violent: 24-120 months)
- [ ] Released prisoners seek housing and employment
- [ ] Recidivism check after release based on model and post-release conditions
- [ ] Overcrowding events at >100% capacity
- [ ] Prison stats displayed in service panel

## Test Plan

- Unit test: punitive model produces ~68% re-arrest rate
- Unit test: rehabilitation model produces ~20% re-arrest rate
- Unit test: overcrowded prison triggers event
- Integration test: rehabilitation investment reduces long-term crime

## Pitfalls

- Incarcerated citizens removed from workforce; large prison population = labor shortage
- Released prisoners competing for limited unskilled jobs

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceType::Prison)
- `crates/simulation/src/crime.rs`

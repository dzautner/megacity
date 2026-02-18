# CRIME-004: Court System and Case Processing

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2 person-weeks)
**Dependencies:** CRIME-001 (crime types), CRIME-003 (policing)
**Source:** historical_demographics_services.md Section 5.2

## Description

Court system that processes arrested criminals. Courthouse building type with capacity (cases per judge per year: 300-500). Court metrics: case backlog, time to trial, conviction rate. Overburdened courts (>800 cases/judge) create backlogs leading to: jail overcrowding, reduced deterrence (criminals see no consequences), rights violations. Well-functioning courts (<6 month backlog) increase deterrence. Court buildings needed per 100K population.

## Definition of Done

- [ ] `Courthouse` service type added
- [ ] `CourtStats` resource: backlog, processing rate, conviction rate
- [ ] Cases generated from crime events (arrests)
- [ ] Processing rate = judges * cases_per_judge_per_year
- [ ] Backlog grows when crimes > processing capacity
- [ ] High backlog reduces crime deterrence (increases crime rate)
- [ ] Low backlog increases deterrence (-15% crime)
- [ ] Court building capacity scales with level

## Test Plan

- Unit test: 500 crimes/year with 1 judge = manageable backlog
- Unit test: 2000 crimes/year with 1 judge = severe backlog
- Integration test: adding courthouse reduces crime over time via deterrence

## Pitfalls

- Court system is abstract (no individual trial simulation); aggregate processing only
- Must connect to prison system (convicted criminals need beds)

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceType enum)
- `crates/simulation/src/crime.rs`
